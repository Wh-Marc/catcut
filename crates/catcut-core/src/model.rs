use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MICROSECONDS_PER_SECOND: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time in microseconds
    pub start: i64,
    /// Duration in microseconds
    pub duration: i64,
}

impl TimeRange {
    pub fn new(start: i64, duration: i64) -> Self {
        Self {
            start: start.max(0),
            duration: duration.max(0),
        }
    }

    pub fn from_seconds(start_sec: f64, duration_sec: f64) -> Self {
        Self::new(
            (start_sec * MICROSECONDS_PER_SECOND as f64) as i64,
            (duration_sec * MICROSECONDS_PER_SECOND as f64) as i64,
        )
    }

    pub fn to_seconds(&self) -> (f64, f64) {
        (
            self.start as f64 / MICROSECONDS_PER_SECOND as f64,
            self.duration as f64 / MICROSECONDS_PER_SECOND as f64,
        )
    }

    pub fn end(&self) -> i64 {
        self.start + self.duration
    }

    pub fn contains(&self, time: i64) -> bool {
        time >= self.start && time < self.end()
    }

    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start < other.end() && other.start < self.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    Video,
    Audio,
    Image,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMaterial {
    pub id: String,
    pub material_type: MaterialType,
    pub path: String,
    pub name: String,
    /// Duration in microseconds
    pub duration: i64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub has_audio: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

impl MediaMaterial {
    pub fn new(
        material_type: MaterialType,
        path: String,
        name: String,
        duration: i64,
        width: u32,
        height: u32,
        fps: f64,
        has_audio: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            material_type,
            path,
            name,
            duration,
            width,
            height,
            fps,
            has_audio,
            sample_rate: 44100,
            channels: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTransform {
    pub scale_x: f64,
    pub scale_y: f64,
    pub position_x: f64, // normalized -1.0 to 1.0 (center = 0.0)
    pub position_y: f64,
    pub rotation: f64,   // degrees
    pub opacity: f64,    // 0.0 to 1.0
}

impl Default for ClipTransform {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            position_x: 0.0,
            position_y: 0.0,
            rotation: 0.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipAudio {
    pub volume: f64,     // 0.0 to 2.0 (1.0 = 100%)
    pub fade_in: i64,    // duration in microseconds
    pub fade_out: i64,   // duration in microseconds
    pub is_muted: bool,
}

impl Default for ClipAudio {
    fn default() -> Self {
        Self {
            volume: 1.0,
            fade_in: 0,
            fade_out: 0,
            is_muted: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_size: u32,
    pub font_family: String,
    pub color: String,       // e.g. "#FFFFFF"
    pub bg_color: Option<String>,
    pub alignment: String,   // "center", "left", "right"
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 48,
            font_family: "Inter, sans-serif".to_string(),
            color: "#FFFFFF".to_string(),
            bg_color: None,
            alignment: "center".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSegment {
    pub id: String,
    pub material_id: String,
    /// Time range extracted from source media
    pub source_timerange: TimeRange,
    /// Time range placed on timeline
    pub target_timerange: TimeRange,
    /// Playback speed multiplier (1.0 = normal, 2.0 = 2x faster)
    pub speed: f64,
    pub transform: ClipTransform,
    pub audio: ClipAudio,
    pub text_content: Option<String>,
    pub text_style: Option<TextStyle>,
}

impl ClipSegment {
    pub fn new(
        material_id: String,
        source_timerange: TimeRange,
        target_start: i64,
        speed: f64,
    ) -> Self {
        let actual_speed = if speed <= 0.0 { 1.0 } else { speed };
        let target_duration = (source_timerange.duration as f64 / actual_speed) as i64;
        Self {
            id: Uuid::new_v4().to_string(),
            material_id,
            source_timerange,
            target_timerange: TimeRange::new(target_start, target_duration),
            speed: actual_speed,
            transform: ClipTransform::default(),
            audio: ClipAudio::default(),
            text_content: None,
            text_style: None,
        }
    }

    pub fn new_text(
        text: String,
        target_start: i64,
        duration: i64,
        style: Option<TextStyle>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            material_id: "text_material".to_string(),
            source_timerange: TimeRange::new(0, duration),
            target_timerange: TimeRange::new(target_start, duration),
            speed: 1.0,
            transform: ClipTransform::default(),
            audio: ClipAudio {
                is_muted: true,
                ..Default::default()
            },
            text_content: Some(text),
            text_style: Some(style.unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackType {
    Video,
    Audio,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub track_type: TrackType,
    pub name: String,
    pub segments: Vec<ClipSegment>,
    pub is_muted: bool,
    pub is_locked: bool,
    pub is_visible: bool,
}

impl Track {
    pub fn new(track_type: TrackType, name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            track_type,
            name,
            segments: Vec::new(),
            is_muted: false,
            is_locked: false,
            is_visible: true,
        }
    }

    pub fn duration(&self) -> i64 {
        self.segments
            .iter()
            .map(|s| s.target_timerange.end())
            .max()
            .unwrap_or(0)
    }

    pub fn find_segment(&self, id: &str) -> Option<&ClipSegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn find_segment_mut(&mut self, id: &str) -> Option<&mut ClipSegment> {
        self.segments.iter_mut().find(|s| s.id == id)
    }

    pub fn sort_segments(&mut self) {
        self.segments
            .sort_by_key(|s| s.target_timerange.start);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub materials: Vec<MediaMaterial>,
    pub tracks: Vec<Track>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Project {
    pub fn new(name: String, width: u32, height: u32, fps: f64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let video_track = Track::new(TrackType::Video, "Video 1".to_string());
        let audio_track = Track::new(TrackType::Audio, "Audio 1".to_string());
        let text_track = Track::new(TrackType::Text, "Text 1".to_string());

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            width,
            height,
            fps,
            materials: Vec::new(),
            tracks: vec![video_track, audio_track, text_track],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn duration(&self) -> i64 {
        self.tracks.iter().map(|t| t.duration()).max().unwrap_or(0)
    }

    pub fn find_material(&self, id: &str) -> Option<&MediaMaterial> {
        self.materials.iter().find(|m| m.id == id)
    }

    pub fn find_segment(&self, id: &str) -> Option<(&Track, &ClipSegment)> {
        for track in &self.tracks {
            if let Some(segment) = track.find_segment(id) {
                return Some((track, segment));
            }
        }
        None
    }

    pub fn find_segment_indices(&self, id: &str) -> Option<(usize, usize)> {
        for (t_idx, track) in self.tracks.iter().enumerate() {
            if let Some(s_idx) = track.segments.iter().position(|s| s.id == id) {
                return Some((t_idx, s_idx));
            }
        }
        None
    }

    pub fn find_segment_mut(&mut self, id: &str) -> Option<&mut ClipSegment> {
        for track in &mut self.tracks {
            if let Some(segment) = track.find_segment_mut(id) {
                return Some(segment);
            }
        }
        None
    }
}
