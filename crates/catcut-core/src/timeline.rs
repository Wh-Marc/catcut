use crate::model::*;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSummary {
    pub project_name: String,
    pub resolution: String,
    pub fps: f64,
    pub total_duration_seconds: f64,
    pub materials_count: usize,
    pub tracks_count: usize,
    pub clips_count: usize,
    pub markdown: String,
}

#[derive(Debug, Clone)]
pub struct TimelineEngine {
    pub project: Project,
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
    max_history: usize,
}

impl TimelineEngine {
    pub fn new(name: &str, width: u32, height: u32, fps: f64) -> Self {
        Self {
            project: Project::new(name.to_string(), width, height, fps),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }

    pub fn from_project(project: Project) -> Self {
        Self {
            project,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.project.clone());
        self.redo_stack.clear();
        self.project.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn undo(&mut self) -> Result<bool> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.project.clone());
            self.project = prev;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self) -> Result<bool> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.project.clone());
            self.project = next;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Add a media material to the project bin
    pub fn add_material(&mut self, material: MediaMaterial) -> Result<MediaMaterial> {
        self.push_undo();
        let clone = material.clone();
        self.project.materials.push(material);
        Ok(clone)
    }

    /// Add a track to the project
    pub fn add_track(&mut self, track_type: TrackType, name: &str) -> Result<Track> {
        self.push_undo();
        let track = Track::new(track_type, name.to_string());
        let clone = track.clone();
        self.project.tracks.push(track);
        Ok(clone)
    }

    /// Insert a clip onto a track
    pub fn add_clip(
        &mut self,
        track_id: Option<&str>,
        material_id: &str,
        target_start: i64,
        source_start: Option<i64>,
        source_duration: Option<i64>,
        speed: Option<f64>,
    ) -> Result<ClipSegment> {
        let material = self
            .project
            .find_material(material_id)
            .ok_or_else(|| anyhow!("Material not found: {}", material_id))?
            .clone();

        let s_start = source_start.unwrap_or(0);
        let s_dur = source_duration.unwrap_or(material.duration.saturating_sub(s_start));
        if s_dur <= 0 {
            bail!("Invalid source duration: {}", s_dur);
        }

        let spd = speed.unwrap_or(1.0);
        let segment = ClipSegment::new(
            material.id.clone(),
            TimeRange::new(s_start, s_dur),
            target_start,
            spd,
        );

        self.push_undo();

        // Determine destination track
        let track = if let Some(tid) = track_id {
            self.project
                .tracks
                .iter_mut()
                .find(|t| t.id == tid)
                .ok_or_else(|| anyhow!("Track not found: {}", tid))?
        } else {
            // Pick or create matching track
            let needed_type = match material.material_type {
                MaterialType::Video | MaterialType::Image => TrackType::Video,
                MaterialType::Audio => TrackType::Audio,
                MaterialType::Text => TrackType::Text,
            };
            self.project
                .tracks
                .iter_mut()
                .find(|t| t.track_type == needed_type)
                .ok_or_else(|| anyhow!("No track available for type {:?}", needed_type))?
        };

        let result = segment.clone();
        track.segments.push(segment);
        track.sort_segments();
        Ok(result)
    }

    /// Split a clip into two segments at a timeline timestamp (microseconds)
    pub fn split_clip(&mut self, clip_id: &str, split_time: i64) -> Result<(ClipSegment, ClipSegment)> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        let orig = &self.project.tracks[t_idx].segments[s_idx];
        if !orig.target_timerange.contains(split_time) {
            bail!(
                "Split time {} is outside clip range [{} - {}]",
                split_time,
                orig.target_timerange.start,
                orig.target_timerange.end()
            );
        }

        let dt_target = split_time - orig.target_timerange.start;
        if dt_target <= 1000 || (orig.target_timerange.duration - dt_target) <= 1000 {
            bail!("Split point is too close to the clip boundaries");
        }

        let dt_source = (dt_target as f64 * orig.speed) as i64;

        let mut seg1 = orig.clone();
        seg1.target_timerange.duration = dt_target;
        seg1.source_timerange.duration = dt_source;

        let mut seg2 = orig.clone();
        seg2.id = uuid::Uuid::new_v4().to_string();
        seg2.target_timerange.start = split_time;
        seg2.target_timerange.duration = orig.target_timerange.duration - dt_target;
        seg2.source_timerange.start = orig.source_timerange.start + dt_source;
        seg2.source_timerange.duration = orig.source_timerange.duration - dt_source;

        self.push_undo();

        let track = &mut self.project.tracks[t_idx];
        track.segments[s_idx] = seg1.clone();
        track.segments.push(seg2.clone());
        track.sort_segments();

        Ok((seg1, seg2))
    }

    /// Trim in/out of a clip
    pub fn trim_clip(
        &mut self,
        clip_id: &str,
        new_source_start: Option<i64>,
        new_duration: Option<i64>,
    ) -> Result<ClipSegment> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();
        let track = &mut self.project.tracks[t_idx];
        let clip = &mut track.segments[s_idx];

        if let Some(src_start) = new_source_start {
            clip.source_timerange.start = src_start.max(0);
        }
        if let Some(dur) = new_duration {
            if dur > 0 {
                clip.source_timerange.duration = dur;
                clip.target_timerange.duration = (dur as f64 / clip.speed) as i64;
            }
        }

        let res = clip.clone();
        track.sort_segments();
        Ok(res)
    }

    /// Remove a clip from timeline, optionally rippling subsequent clips
    pub fn remove_clip(&mut self, clip_id: &str, ripple: bool) -> Result<ClipSegment> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();
        let track = &mut self.project.tracks[t_idx];
        let removed = track.segments.remove(s_idx);

        if ripple {
            let shift = removed.target_timerange.duration;
            for seg in &mut track.segments {
                if seg.target_timerange.start >= removed.target_timerange.end() {
                    seg.target_timerange.start = (seg.target_timerange.start - shift).max(0);
                }
            }
        }

        Ok(removed)
    }

    /// Move a clip to a new position and/or different track
    pub fn move_clip(
        &mut self,
        clip_id: &str,
        new_track_id: Option<&str>,
        new_target_start: i64,
    ) -> Result<ClipSegment> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();

        let mut clip = self.project.tracks[t_idx].segments.remove(s_idx);
        clip.target_timerange.start = new_target_start.max(0);

        let target_track_idx = if let Some(target_tid) = new_track_id {
            self.project
                .tracks
                .iter()
                .position(|t| t.id == target_tid)
                .ok_or_else(|| anyhow!("Target track not found: {}", target_tid))?
        } else {
            t_idx
        };

        let result = clip.clone();
        let dest_track = &mut self.project.tracks[target_track_idx];
        dest_track.segments.push(clip);
        dest_track.sort_segments();

        Ok(result)
    }

    /// Set playback speed of a clip
    pub fn set_speed(&mut self, clip_id: &str, speed: f64) -> Result<ClipSegment> {
        if speed <= 0.05 || speed > 100.0 {
            bail!("Speed must be between 0.05 and 100.0");
        }

        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();
        let track = &mut self.project.tracks[t_idx];
        let clip = &mut track.segments[s_idx];

        clip.speed = speed;
        clip.target_timerange.duration = (clip.source_timerange.duration as f64 / speed) as i64;

        let result = clip.clone();
        track.sort_segments();
        Ok(result)
    }

    /// Adjust audio properties (volume, fade in/out, mute)
    pub fn adjust_audio(
        &mut self,
        clip_id: &str,
        volume: Option<f64>,
        fade_in_us: Option<i64>,
        fade_out_us: Option<i64>,
        is_muted: Option<bool>,
    ) -> Result<ClipAudio> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();
        let track = &mut self.project.tracks[t_idx];
        let clip = &mut track.segments[s_idx];

        if let Some(v) = volume {
            clip.audio.volume = v.clamp(0.0, 5.0);
        }
        if let Some(fi) = fade_in_us {
            clip.audio.fade_in = fi.max(0);
        }
        if let Some(fo) = fade_out_us {
            clip.audio.fade_out = fo.max(0);
        }
        if let Some(m) = is_muted {
            clip.audio.is_muted = m;
        }

        Ok(clip.audio.clone())
    }

    /// Adjust visual transforms (scale, position, rotation, opacity)
    pub fn set_transform(
        &mut self,
        clip_id: &str,
        scale_x: Option<f64>,
        scale_y: Option<f64>,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        rotation: Option<f64>,
        opacity: Option<f64>,
    ) -> Result<ClipTransform> {
        let (t_idx, s_idx) = self
            .project
            .find_segment_indices(clip_id)
            .ok_or_else(|| anyhow!("Clip not found: {}", clip_id))?;

        self.push_undo();
        let track = &mut self.project.tracks[t_idx];
        let clip = &mut track.segments[s_idx];

        if let Some(sx) = scale_x {
            clip.transform.scale_x = sx;
        }
        if let Some(sy) = scale_y {
            clip.transform.scale_y = sy;
        }
        if let Some(px) = pos_x {
            clip.transform.position_x = px;
        }
        if let Some(py) = pos_y {
            clip.transform.position_y = py;
        }
        if let Some(rot) = rotation {
            clip.transform.rotation = rot;
        }
        if let Some(op) = opacity {
            clip.transform.opacity = op.clamp(0.0, 1.0);
        }

        Ok(clip.transform.clone())
    }

    /// Add a text / title clip
    pub fn add_text(
        &mut self,
        track_id: Option<&str>,
        text: &str,
        target_start: i64,
        duration: i64,
        style: Option<TextStyle>,
    ) -> Result<ClipSegment> {
        self.push_undo();
        let segment = ClipSegment::new_text(text.to_string(), target_start, duration, style);

        let track = if let Some(tid) = track_id {
            self.project
                .tracks
                .iter_mut()
                .find(|t| t.id == tid)
                .ok_or_else(|| anyhow!("Track not found: {}", tid))?
        } else {
            // Find or create Text track
            if let Some(pos) = self.project.tracks.iter().position(|t| t.track_type == TrackType::Text) {
                &mut self.project.tracks[pos]
            } else {
                let text_track = Track::new(TrackType::Text, "Text 1".to_string());
                self.project.tracks.push(text_track);
                self.project.tracks.last_mut().unwrap()
            }
        };

        let res = segment.clone();
        track.segments.push(segment);
        track.sort_segments();
        Ok(res)
    }

    /// Format timeline as Markdown overview for LLM chat
    pub fn summary(&self) -> TimelineSummary {
        let total_us = self.project.duration();
        let total_sec = total_us as f64 / MICROSECONDS_PER_SECOND as f64;
        let clips_count: usize = self.project.tracks.iter().map(|t| t.segments.len()).sum();

        let mut md = String::new();
        md.push_str(&format!(
            "### 🎬 Project: **{}**\n- **Resolution**: {}x{} @ {:.1} fps\n- **Total Duration**: {:.2}s ({:.1} min)\n- **Clips**: {} across {} tracks\n\n",
            self.project.name, self.project.width, self.project.height, self.project.fps, total_sec, total_sec / 60.0, clips_count, self.project.tracks.len()
        ));

        md.push_str("#### 📂 Media Library:\n");
        if self.project.materials.is_empty() {
            md.push_str("*(No media imported)*\n\n");
        } else {
            for m in &self.project.materials {
                let m_sec = m.duration as f64 / MICROSECONDS_PER_SECOND as f64;
                md.push_str(&format!(
                    "- `[{}]` **{}** ({:?}) - {:.2}s, {}x{}, ID: `{}`\n",
                    &m.id[..8], m.name, m.material_type, m_sec, m.width, m.height, m.id
                ));
            }
            md.push('\n');
        }

        md.push_str("#### 🎞️ Tracks & Clips:\n");
        for (i, track) in self.project.tracks.iter().enumerate() {
            md.push_str(&format!(
                "**Track {} ({} - {:?})** [ID: `{}`]:\n",
                i + 1, track.name, track.track_type, track.id
            ));
            if track.segments.is_empty() {
                md.push_str("  *(Empty track)*\n");
            } else {
                for seg in &track.segments {
                    let (t_start, t_dur) = seg.target_timerange.to_seconds();
                    let t_end = t_start + t_dur;
                    let label = if let Some(text) = &seg.text_content {
                        format!("Text: \"{}\"", text)
                    } else if let Some(m) = self.project.find_material(&seg.material_id) {
                        m.name.clone()
                    } else {
                        "Unknown".to_string()
                    };

                    let speed_str = if (seg.speed - 1.0).abs() > 0.001 {
                        format!(" @ {:.1}x", seg.speed)
                    } else {
                        "".to_string()
                    };

                    let vol_str = if (seg.audio.volume - 1.0).abs() > 0.001 {
                        format!(" (vol {:.0}%)", seg.audio.volume * 100.0)
                    } else {
                        "".to_string()
                    };

                    md.push_str(&format!(
                        "  - `[{:05.2}s - {:05.2}s]` **{}**{}{} [ID: `{}`]\n",
                        t_start, t_end, label, speed_str, vol_str, seg.id
                    ));
                }
            }
        }

        TimelineSummary {
            project_name: self.project.name.clone(),
            resolution: format!("{}x{}", self.project.width, self.project.height),
            fps: self.project.fps,
            total_duration_seconds: total_sec,
            materials_count: self.project.materials.len(),
            tracks_count: self.project.tracks.len(),
            clips_count,
            markdown: md,
        }
    }

    /// Automatically cut out silent/hesitation intervals and ripple-stitch voice clips onto the timeline
    pub fn auto_rough_cut(
        &mut self,
        material_id: &str,
        noise_db: Option<f64>,
        min_silence_sec: Option<f64>,
        padding_sec: Option<f64>,
    ) -> Result<Vec<ClipSegment>> {
        let material = self
            .project
            .find_material(material_id)
            .ok_or_else(|| anyhow!("Material not found: {}", material_id))?
            .clone();

        let total_dur_sec = material.duration as f64 / MICROSECONDS_PER_SECOND as f64;
        let db = noise_db.unwrap_or(-30.0);
        let min_sil = min_silence_sec.unwrap_or(0.4);
        let pad = padding_sec.unwrap_or(0.08);

        let voices = crate::media::extract_voice_intervals(&material.path, total_dur_sec, db, min_sil, pad)?;
        if voices.is_empty() {
            bail!("No speech segments detected in media");
        }

        self.push_undo();

        let track_idx = if let Some(idx) = self.project.tracks.iter().position(|t| t.track_type == TrackType::Video) {
            self.project.tracks[idx].segments.clear();
            idx
        } else {
            let t = Track::new(TrackType::Video, "Auto Rough Cut".to_string());
            self.project.tracks.push(t);
            self.project.tracks.len() - 1
        };

        let mut timeline_pos_us = 0i64;
        let mut created_segments = Vec::new();

        for voice in voices {
            let src_start_us = (voice.start_sec * MICROSECONDS_PER_SECOND as f64) as i64;
            let src_dur_us = (voice.duration_sec * MICROSECONDS_PER_SECOND as f64) as i64;

            let seg = ClipSegment::new(
                material.id.clone(),
                TimeRange::new(src_start_us, src_dur_us),
                timeline_pos_us,
                1.0,
            );

            timeline_pos_us += seg.target_timerange.duration;
            created_segments.push(seg);
        }

        self.project.tracks[track_idx].segments = created_segments.clone();
        Ok(created_segments)
    }
}
