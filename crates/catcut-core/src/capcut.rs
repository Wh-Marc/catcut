use crate::model::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutTimeRange {
    pub duration: i64,
    pub start: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutVideoMaterial {
    pub id: String,
    pub path: String,
    pub duration: i64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub material_name: String,
    #[serde(default)]
    pub has_audio: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutAudioMaterial {
    pub id: String,
    pub path: String,
    pub duration: i64,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutTextMaterial {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CapCutMaterials {
    #[serde(default)]
    pub videos: Vec<CapCutVideoMaterial>,
    #[serde(default)]
    pub audios: Vec<CapCutAudioMaterial>,
    #[serde(default)]
    pub texts: Vec<CapCutTextMaterial>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutSegment {
    pub id: String,
    pub material_id: String,
    pub source_timerange: CapCutTimeRange,
    pub target_timerange: CapCutTimeRange,
    #[serde(default = "default_speed")]
    pub speed: f64,
}

fn default_speed() -> f64 {
    1.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutTrack {
    pub id: String,
    #[serde(rename = "type")]
    pub track_type: String, // "video", "audio", "text"
    #[serde(default)]
    pub segments: Vec<CapCutSegment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapCutDraftContent {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub fps: f64,
    pub duration: i64,
    pub materials: CapCutMaterials,
    pub tracks: Vec<CapCutTrack>,
}

/// Convert CatCut Project into CapCut draft_content.json string
pub fn export_to_capcut_json(project: &Project) -> Result<String> {
    let mut capcut_materials = CapCutMaterials::default();

    for m in &project.materials {
        match m.material_type {
            MaterialType::Video | MaterialType::Image => {
                capcut_materials.videos.push(CapCutVideoMaterial {
                    id: m.id.clone(),
                    path: m.path.clone(),
                    duration: m.duration,
                    width: m.width,
                    height: m.height,
                    material_name: m.name.clone(),
                    has_audio: m.has_audio,
                });
            }
            MaterialType::Audio => {
                capcut_materials.audios.push(CapCutAudioMaterial {
                    id: m.id.clone(),
                    path: m.path.clone(),
                    duration: m.duration,
                    name: m.name.clone(),
                });
            }
            MaterialType::Text => {}
        }
    }

    let mut capcut_tracks = Vec::new();
    for track in &project.tracks {
        let type_str = match track.track_type {
            TrackType::Video => "video",
            TrackType::Audio => "audio",
            TrackType::Text => "text",
        };

        let mut segments = Vec::new();
        for seg in &track.segments {
            segments.push(CapCutSegment {
                id: seg.id.clone(),
                material_id: seg.material_id.clone(),
                source_timerange: CapCutTimeRange {
                    start: seg.source_timerange.start,
                    duration: seg.source_timerange.duration,
                },
                target_timerange: CapCutTimeRange {
                    start: seg.target_timerange.start,
                    duration: seg.target_timerange.duration,
                },
                speed: seg.speed,
            });

            // If text segment, also add to materials.texts
            if let Some(text) = &seg.text_content {
                capcut_materials.texts.push(CapCutTextMaterial {
                    id: seg.material_id.clone(),
                    content: text.clone(),
                });
            }
        }

        capcut_tracks.push(CapCutTrack {
            id: track.id.clone(),
            track_type: type_str.to_string(),
            segments,
        });
    }

    let draft = CapCutDraftContent {
        id: project.id.clone(),
        name: project.name.clone(),
        fps: project.fps,
        duration: project.duration(),
        materials: capcut_materials,
        tracks: capcut_tracks,
    };

    serde_json::to_string_pretty(&draft).context("Failed to serialize CapCut draft")
}

/// Import CapCut draft_content.json file into a CatCut Project
pub fn import_from_capcut_file(path: &Path) -> Result<Project> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read CapCut draft file: {:?}", path))?;
    let draft: CapCutDraftContent = serde_json::from_str(&content)
        .with_context(|| "Failed to parse CapCut draft JSON")?;

    let mut materials = Vec::new();
    for v in draft.materials.videos {
        materials.push(MediaMaterial {
            id: v.id,
            material_type: MaterialType::Video,
            path: v.path,
            name: if v.material_name.is_empty() { "Video".to_string() } else { v.material_name },
            duration: v.duration,
            width: v.width,
            height: v.height,
            fps: draft.fps,
            has_audio: v.has_audio,
            sample_rate: 44100,
            channels: 2,
        });
    }
    for a in draft.materials.audios {
        materials.push(MediaMaterial {
            id: a.id,
            material_type: MaterialType::Audio,
            path: a.path,
            name: if a.name.is_empty() { "Audio".to_string() } else { a.name },
            duration: a.duration,
            width: 0,
            height: 0,
            fps: 0.0,
            has_audio: true,
            sample_rate: 44100,
            channels: 2,
        });
    }

    let mut tracks = Vec::new();
    for (i, t) in draft.tracks.into_iter().enumerate() {
        let (t_type, def_name) = match t.track_type.as_str() {
            "video" => (TrackType::Video, format!("Video {}", i + 1)),
            "audio" => (TrackType::Audio, format!("Audio {}", i + 1)),
            _ => (TrackType::Text, format!("Text {}", i + 1)),
        };

        let mut segments = Vec::new();
        for s in t.segments {
            let mut seg = ClipSegment::new(
                s.material_id,
                TimeRange::new(s.source_timerange.start, s.source_timerange.duration),
                s.target_timerange.start,
                s.speed,
            );
            seg.id = s.id;
            segments.push(seg);
        }

        tracks.push(Track {
            id: t.id,
            track_type: t_type,
            name: def_name,
            segments,
            is_muted: false,
            is_locked: false,
            is_visible: true,
        });
    }

    let now = chrono::Utc::now().timestamp_millis();
    Ok(Project {
        id: if draft.id.is_empty() { Uuid::new_v4().to_string() } else { draft.id },
        name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("CapCut Imported").to_string(),
        width: 1920,
        height: 1080,
        fps: draft.fps,
        materials,
        tracks,
        created_at: now,
        updated_at: now,
    })
}
