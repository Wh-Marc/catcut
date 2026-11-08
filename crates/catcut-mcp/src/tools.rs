use catcut_core::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct McpTool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

pub fn get_tools_list() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "get_timeline",
            description: "Get a comprehensive summary of the current video editing project, including resolution, fps, duration, media library, and all tracks/clips on the timeline.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_raw_json": {
                        "type": "boolean",
                        "description": "If true, also includes raw JSON data of the project"
                    }
                }
            }),
        },
        McpTool {
            name: "import_media",
            description: "Probe a media file (video, audio, or image) using ffprobe and add it to the project's media library.",
            input_schema: json!({
                "type": "object",
                "required": ["file_path"],
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute or relative file path to the video/audio/image file"
                    }
                }
            }),
        },
        McpTool {
            name: "add_clip",
            description: "Place a clip from the media library onto a track on the timeline.",
            input_schema: json!({
                "type": "object",
                "required": ["material_id", "target_start_sec"],
                "properties": {
                    "material_id": {
                        "type": "string",
                        "description": "ID of the media material from get_timeline or import_media"
                    },
                    "track_id": {
                        "type": "string",
                        "description": "Optional track ID. If omitted, the default video/audio track is used."
                    },
                    "target_start_sec": {
                        "type": "number",
                        "description": "Position on the timeline where the clip starts (in seconds)"
                    },
                    "source_start_sec": {
                        "type": "number",
                        "description": "Start time within the source media file (in seconds, default 0)"
                    },
                    "duration_sec": {
                        "type": "number",
                        "description": "Duration to play (in seconds). Defaults to remainder of media."
                    },
                    "speed": {
                        "type": "number",
                        "description": "Playback speed multiplier (e.g. 1.0 for normal, 2.0 for 2x fast, 0.5 for slow-mo)"
                    }
                }
            }),
        },
        McpTool {
            name: "split_clip",
            description: "Cut / split an existing clip on the timeline into two separate clips at a specified timestamp.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id", "split_time_sec"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip to split"
                    },
                    "split_time_sec": {
                        "type": "number",
                        "description": "Timestamp on the timeline where the cut should occur (in seconds)"
                    }
                }
            }),
        },
        McpTool {
            name: "trim_clip",
            description: "Trim the in-point or duration of an existing clip on the timeline.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip to trim"
                    },
                    "new_source_start_sec": {
                        "type": "number",
                        "description": "New in-point within the source media (in seconds)"
                    },
                    "new_duration_sec": {
                        "type": "number",
                        "description": "New duration of the clip (in seconds)"
                    }
                }
            }),
        },
        McpTool {
            name: "remove_clip",
            description: "Delete a clip from the timeline. Set ripple to true if you want subsequent clips to automatically shift left to close the gap.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip to remove"
                    },
                    "ripple": {
                        "type": "boolean",
                        "description": "If true, shifts subsequent clips to fill the empty space (ripple delete)"
                    }
                }
            }),
        },
        McpTool {
            name: "move_clip",
            description: "Move a clip to a different position on the timeline or to a different track.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id", "new_target_start_sec"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip to move"
                    },
                    "new_target_start_sec": {
                        "type": "number",
                        "description": "New start time on the timeline (in seconds)"
                    },
                    "new_track_id": {
                        "type": "string",
                        "description": "Optional new track ID"
                    }
                }
            }),
        },
        McpTool {
            name: "set_speed",
            description: "Change the playback speed of a clip on the timeline.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id", "speed"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip"
                    },
                    "speed": {
                        "type": "number",
                        "description": "Speed factor (e.g., 0.5 = 2x slower, 1.0 = normal, 2.0 = 2x faster)"
                    }
                }
            }),
        },
        McpTool {
            name: "adjust_audio",
            description: "Adjust audio properties of a clip (volume, fade in, fade out, mute).",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip"
                    },
                    "volume": {
                        "type": "number",
                        "description": "Volume multiplier (1.0 = 100%, 0.5 = 50%, 2.0 = 200%)"
                    },
                    "fade_in_sec": {
                        "type": "number",
                        "description": "Fade in duration in seconds"
                    },
                    "fade_out_sec": {
                        "type": "number",
                        "description": "Fade out duration in seconds"
                    },
                    "is_muted": {
                        "type": "boolean",
                        "description": "Mute audio on this clip"
                    }
                }
            }),
        },
        McpTool {
            name: "set_transform",
            description: "Set video transform properties: position, scale, rotation, and opacity.",
            input_schema: json!({
                "type": "object",
                "required": ["clip_id"],
                "properties": {
                    "clip_id": {
                        "type": "string",
                        "description": "ID of the clip"
                    },
                    "scale_x": { "type": "number", "description": "Horizontal scale (default 1.0)" },
                    "scale_y": { "type": "number", "description": "Vertical scale (default 1.0)" },
                    "position_x": { "type": "number", "description": "Normalized X (-1.0 to 1.0, center=0.0)" },
                    "position_y": { "type": "number", "description": "Normalized Y (-1.0 to 1.0, center=0.0)" },
                    "rotation": { "type": "number", "description": "Rotation in degrees" },
                    "opacity": { "type": "number", "description": "Opacity from 0.0 to 1.0" }
                }
            }),
        },
        McpTool {
            name: "add_text",
            description: "Add a text / title / caption overlay to the timeline.",
            input_schema: json!({
                "type": "object",
                "required": ["text", "target_start_sec", "duration_sec"],
                "properties": {
                    "text": { "type": "string", "description": "Text content to display" },
                    "target_start_sec": { "type": "number", "description": "Timeline start time in seconds" },
                    "duration_sec": { "type": "number", "description": "Duration in seconds" },
                    "font_size": { "type": "integer", "description": "Font size in pixels (default 48)" },
                    "color": { "type": "string", "description": "Text color in hex (e.g. #FFFFFF)" },
                    "track_id": { "type": "string", "description": "Optional text track ID" }
                }
            }),
        },
        McpTool {
            name: "inspect_frame",
            description: "Extract a preview frame at a given timestamp and return it as a base64 PNG data URL so you (the AI) can visually inspect the video frame.",
            input_schema: json!({
                "type": "object",
                "required": ["timestamp_sec"],
                "properties": {
                    "timestamp_sec": {
                        "type": "number",
                        "description": "Timestamp on the timeline to inspect (in seconds)"
                    },
                    "max_width": {
                        "type": "integer",
                        "description": "Max width in pixels for thumbnail (default 640)"
                    }
                }
            }),
        },
        McpTool {
            name: "export_project",
            description: "Render the timeline into an MP4 video file using FFmpeg.",
            input_schema: json!({
                "type": "object",
                "required": ["output_path"],
                "properties": {
                    "output_path": {
                        "type": "string",
                        "description": "Path where the rendered MP4 file should be saved"
                    },
                    "width": { "type": "integer", "description": "Output video width (default project width)" },
                    "height": { "type": "integer", "description": "Output video height (default project height)" },
                    "fps": { "type": "number", "description": "Output frame rate (default project fps)" },
                    "crf": { "type": "integer", "description": "H.264 CRF quality (default 23, lower is higher quality)" }
                }
            }),
        },
        McpTool {
            name: "import_capcut_draft",
            description: "Import an existing CapCut draft_content.json file directly into the CatCut timeline.",
            input_schema: json!({
                "type": "object",
                "required": ["draft_path"],
                "properties": {
                    "draft_path": {
                        "type": "string",
                        "description": "File path to CapCut draft_content.json"
                    }
                }
            }),
        },
        McpTool {
            name: "export_capcut_draft",
            description: "Export the current project into CapCut-compatible draft_content.json format.",
            input_schema: json!({
                "type": "object",
                "required": ["output_path"],
                "properties": {
                    "output_path": {
                        "type": "string",
                        "description": "Path where draft_content.json should be saved"
                    }
                }
            }),
        },
        McpTool {
            name: "undo",
            description: "Undo the last editing operation.",
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "redo",
            description: "Redo the previously undone operation.",
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "detect_scenes",
            description: "Fast scene change detection. Analyzes a video to find all visual scene transitions and cut points.",
            input_schema: json!({
                "type": "object",
                "required": ["material_id"],
                "properties": {
                    "material_id": { "type": "string", "description": "ID of media material from library" },
                    "threshold": { "type": "number", "description": "Sensitivity threshold 0.05-0.5 (default 0.25)" }
                }
            }),
        },
        McpTool {
            name: "detect_silence",
            description: "Detect silence and hesitation intervals in a video/audio file for smart cut and pacing analysis.",
            input_schema: json!({
                "type": "object",
                "required": ["material_id"],
                "properties": {
                    "material_id": { "type": "string", "description": "ID of media material from library" },
                    "noise_db": { "type": "number", "description": "Noise threshold in dB (default -30.0)" },
                    "min_duration_sec": { "type": "number", "description": "Minimum silence duration in seconds (default 0.4)" }
                }
            }),
        },
        McpTool {
            name: "auto_rough_cut",
            description: "Automatically cut out all dead air, silence, and hesitations from a video, then ripple-stitch speech segments onto the timeline for lazy editing.",
            input_schema: json!({
                "type": "object",
                "required": ["material_id"],
                "properties": {
                    "material_id": { "type": "string", "description": "ID of media material from library" },
                    "noise_db": { "type": "number", "description": "Noise threshold in dB (default -30.0)" },
                    "min_silence_sec": { "type": "number", "description": "Minimum silence duration to remove (default 0.4)" },
                    "padding_sec": { "type": "number", "description": "Padding around voice cuts to keep natural pacing (default 0.08)" }
                }
            }),
        },
        McpTool {
            name: "get_transcript",
            description: "Get speech transcript with timestamps from embedded subtitles or accompanying SRT/VTT file to understand video content.",
            input_schema: json!({
                "type": "object",
                "required": ["material_id"],
                "properties": {
                    "material_id": { "type": "string", "description": "ID of media material from library" },
                    "srt_path": { "type": "string", "description": "Optional path to subtitle file (.srt/.vtt)" }
                }
            }),
        },
    ]
}

/// Execute a tool call against the shared TimelineEngine
pub async fn execute_tool(
    engine: Arc<Mutex<TimelineEngine>>,
    project_file_path: Option<String>,
    name: &str,
    args: Value,
) -> Result<Value, String> {
    let mut eng = engine.lock().await;

    let result_text = match name {
        "get_timeline" => {
            let summary = eng.summary();
            let include_raw = args.get("include_raw_json").and_then(|v| v.as_bool()).unwrap_or(false);
            if include_raw {
                let raw = serde_json::to_string_pretty(&eng.project).unwrap_or_default();
                format!("{}\n\n```json\n{}\n```", summary.markdown, raw)
            } else {
                summary.markdown
            }
        }
        "import_media" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter 'file_path'".to_string())?;

            match probe_media(file_path) {
                Ok(mat) => {
                    let id = mat.id.clone();
                    let name = mat.name.clone();
                    let dur_sec = mat.duration as f64 / MICROSECONDS_PER_SECOND as f64;
                    let width = mat.width;
                    let height = mat.height;
                    let fps = mat.fps;
                    eng.add_material(mat)
                        .map_err(|e| format!("Failed to add material: {}", e))?;

                    format!(
                        "✅ Imported media successfully!\n- **Name**: {}\n- **ID**: `{}`\n- **Duration**: {:.2}s\n- **Resolution**: {}x{} @ {:.1} fps",
                        name, id, dur_sec, width, height, fps
                    )
                }
                Err(e) => return Err(format!("Failed to probe media file: {}", e)),
            }
        }
        "add_clip" => {
            let material_id = args
                .get("material_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'material_id'".to_string())?;

            let target_start_sec = args
                .get("target_start_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'target_start_sec'".to_string())?;

            let track_id = args.get("track_id").and_then(|v| v.as_str());

            let source_start_us = args
                .get("source_start_sec")
                .and_then(|v| v.as_f64())
                .map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);

            let duration_us = args
                .get("duration_sec")
                .and_then(|v| v.as_f64())
                .map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);

            let speed = args.get("speed").and_then(|v| v.as_f64());

            let target_start_us = (target_start_sec * MICROSECONDS_PER_SECOND as f64) as i64;

            match eng.add_clip(track_id, material_id, target_start_us, source_start_us, duration_us, speed) {
                Ok(seg) => {
                    let (t_start, t_dur) = seg.target_timerange.to_seconds();
                    format!(
                        "✅ Added clip to timeline!\n- **Clip ID**: `{}`\n- **Position**: {:.2}s - {:.2}s (Duration: {:.2}s)\n- **Speed**: {:.1}x",
                        seg.id, t_start, t_start + t_dur, t_dur, seg.speed
                    )
                }
                Err(e) => return Err(format!("Failed to add clip: {}", e)),
            }
        }
        "split_clip" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let split_time_sec = args
                .get("split_time_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'split_time_sec'".to_string())?;

            let split_time_us = (split_time_sec * MICROSECONDS_PER_SECOND as f64) as i64;

            match eng.split_clip(clip_id, split_time_us) {
                Ok((part1, part2)) => {
                    let (p1_s, p1_d) = part1.target_timerange.to_seconds();
                    let (p2_s, p2_d) = part2.target_timerange.to_seconds();
                    format!(
                        "✂️ Clip split into two:\n1. Clip ID `{}`: [{:.2}s - {:.2}s]\n2. Clip ID `{}`: [{:.2}s - {:.2}s]",
                        part1.id, p1_s, p1_s + p1_d, part2.id, p2_s, p2_s + p2_d
                    )
                }
                Err(e) => return Err(format!("Failed to split clip: {}", e)),
            }
        }
        "trim_clip" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let src_start_us = args
                .get("new_source_start_sec")
                .and_then(|v| v.as_f64())
                .map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);

            let dur_us = args
                .get("new_duration_sec")
                .and_then(|v| v.as_f64())
                .map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);

            match eng.trim_clip(clip_id, src_start_us, dur_us) {
                Ok(seg) => {
                    let (s, d) = seg.target_timerange.to_seconds();
                    format!("✂️ Trimmed clip `{}`: now [{:.2}s - {:.2}s]", seg.id, s, s + d)
                }
                Err(e) => return Err(format!("Failed to trim clip: {}", e)),
            }
        }
        "remove_clip" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let ripple = args.get("ripple").and_then(|v| v.as_bool()).unwrap_or(false);

            match eng.remove_clip(clip_id, ripple) {
                Ok(seg) => {
                    format!("🗑️ Removed clip `{}` (ripple = {})", seg.id, ripple)
                }
                Err(e) => return Err(format!("Failed to remove clip: {}", e)),
            }
        }
        "move_clip" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let new_start_sec = args
                .get("new_target_start_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'new_target_start_sec'".to_string())?;

            let new_start_us = (new_start_sec * MICROSECONDS_PER_SECOND as f64) as i64;
            let new_track_id = args.get("new_track_id").and_then(|v| v.as_str());

            match eng.move_clip(clip_id, new_track_id, new_start_us) {
                Ok(seg) => {
                    let (s, d) = seg.target_timerange.to_seconds();
                    format!("Moved clip `{}` to [{:.2}s - {:.2}s]", seg.id, s, s + d)
                }
                Err(e) => return Err(format!("Failed to move clip: {}", e)),
            }
        }
        "set_speed" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let speed = args
                .get("speed")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'speed'".to_string())?;

            match eng.set_speed(clip_id, speed) {
                Ok(seg) => {
                    let (s, d) = seg.target_timerange.to_seconds();
                    format!("⚡ Set clip `{}` speed to {:.2}x (new duration: {:.2}s, [{:.2}s - {:.2}s])", seg.id, speed, d, s, s + d)
                }
                Err(e) => return Err(format!("Failed to set speed: {}", e)),
            }
        }
        "adjust_audio" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let volume = args.get("volume").and_then(|v| v.as_f64());
            let fade_in = args.get("fade_in_sec").and_then(|v| v.as_f64()).map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);
            let fade_out = args.get("fade_out_sec").and_then(|v| v.as_f64()).map(|s| (s * MICROSECONDS_PER_SECOND as f64) as i64);
            let is_muted = args.get("is_muted").and_then(|v| v.as_bool());

            match eng.adjust_audio(clip_id, volume, fade_in, fade_out, is_muted) {
                Ok(aud) => {
                    format!("🔊 Adjusted audio for clip `{}`: volume={:.0}%, muted={}", clip_id, aud.volume * 100.0, aud.is_muted)
                }
                Err(e) => return Err(format!("Failed to adjust audio: {}", e)),
            }
        }
        "set_transform" => {
            let clip_id = args
                .get("clip_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'clip_id'".to_string())?;

            let scale_x = args.get("scale_x").and_then(|v| v.as_f64());
            let scale_y = args.get("scale_y").and_then(|v| v.as_f64());
            let pos_x = args.get("position_x").and_then(|v| v.as_f64());
            let pos_y = args.get("position_y").and_then(|v| v.as_f64());
            let rot = args.get("rotation").and_then(|v| v.as_f64());
            let op = args.get("opacity").and_then(|v| v.as_f64());

            match eng.set_transform(clip_id, scale_x, scale_y, pos_x, pos_y, rot, op) {
                Ok(tr) => {
                    format!("🎨 Updated transform for clip `{}`: scale=({:.2},{:.2}), pos=({:.2},{:.2}), opacity={:.2}", clip_id, tr.scale_x, tr.scale_y, tr.position_x, tr.position_y, tr.opacity)
                }
                Err(e) => return Err(format!("Failed to set transform: {}", e)),
            }
        }
        "add_text" => {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'text'".to_string())?;

            let target_start_sec = args
                .get("target_start_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'target_start_sec'".to_string())?;

            let duration_sec = args
                .get("duration_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'duration_sec'".to_string())?;

            let font_size = args.get("font_size").and_then(|v| v.as_u64()).map(|n| n as u32).unwrap_or(48);
            let color = args.get("color").and_then(|v| v.as_str()).unwrap_or("#FFFFFF").to_string();
            let track_id = args.get("track_id").and_then(|v| v.as_str());

            let style = TextStyle {
                font_size,
                font_family: "Inter, sans-serif".to_string(),
                color,
                bg_color: None,
                alignment: "center".to_string(),
            };

            let start_us = (target_start_sec * MICROSECONDS_PER_SECOND as f64) as i64;
            let dur_us = (duration_sec * MICROSECONDS_PER_SECOND as f64) as i64;

            match eng.add_text(track_id, text, start_us, dur_us, Some(style)) {
                Ok(seg) => {
                    let (s, d) = seg.target_timerange.to_seconds();
                    format!("🔤 Added text \"{}\" at [{:.2}s - {:.2}s] [Clip ID: `{}`]", text, s, s + d, seg.id)
                }
                Err(e) => return Err(format!("Failed to add text: {}", e)),
            }
        }
        "inspect_frame" => {
            let timestamp_sec = args
                .get("timestamp_sec")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing required 'timestamp_sec'".to_string())?;

            let max_w = args.get("max_width").and_then(|v| v.as_u64()).map(|n| n as u32).unwrap_or(640);
            let time_us = (timestamp_sec * MICROSECONDS_PER_SECOND as f64) as i64;

            // Find top-most video clip at time_us
            let mut top_clip_media = None;
            for track in &eng.project.tracks {
                if track.track_type == TrackType::Video {
                    for seg in &track.segments {
                        if seg.target_timerange.contains(time_us) {
                            if let Some(mat) = eng.project.find_material(&seg.material_id) {
                                let offset_in_target = time_us - seg.target_timerange.start;
                                let offset_in_source = (offset_in_target as f64 * seg.speed) as i64;
                                let source_time_us = seg.source_timerange.start + offset_in_source;
                                let source_time_sec = source_time_us as f64 / MICROSECONDS_PER_SECOND as f64;
                                top_clip_media = Some((mat.path.clone(), source_time_sec));
                            }
                        }
                    }
                }
            }

            match top_clip_media {
                Some((path, source_time_sec)) => {
                    match extract_frame_base64(&path, source_time_sec, max_w) {
                        Ok(data_uri) => {
                            return Ok(json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": format!("Frame extracted at timeline timestamp {:.2}s (source: {}, offset {:.2}s)", timestamp_sec, path, source_time_sec)
                                    },
                                    {
                                        "type": "image",
                                        "data": data_uri.trim_start_matches("data:image/png;base64,"),
                                        "mimeType": "image/png"
                                    }
                                ]
                            }));
                        }
                        Err(e) => return Err(format!("Failed to extract frame: {}", e)),
                    }
                }
                None => {
                    return Err(format!("No video clip found at timeline position {:.2}s", timestamp_sec));
                }
            }
        }
        "export_project" => {
            let output_path = args
                .get("output_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'output_path'".to_string())?;

            let width = args.get("width").and_then(|v| v.as_u64()).map(|n| n as u32);
            let height = args.get("height").and_then(|v| v.as_u64()).map(|n| n as u32);
            let fps = args.get("fps").and_then(|v| v.as_f64());
            let crf = args.get("crf").and_then(|v| v.as_u64()).map(|n| n as u32);

            let options = ExportOptions {
                output_path: output_path.to_string(),
                width,
                height,
                fps,
                crf,
                preset: Some("medium".to_string()),
            };

            match export_project(&eng.project, &options) {
                Ok(_) => format!("🎬 Project exported successfully to `{}`!", output_path),
                Err(e) => return Err(format!("Export failed: {}", e)),
            }
        }
        "import_capcut_draft" => {
            let draft_path = args
                .get("draft_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'draft_path'".to_string())?;

            match import_from_capcut_file(std::path::Path::new(draft_path)) {
                Ok(proj) => {
                    let name = proj.name.clone();
                    let t_count = proj.tracks.len();
                    let m_count = proj.materials.len();
                    *eng = TimelineEngine::from_project(proj);
                    format!("✅ Imported CapCut draft: \"{}\" with {} tracks and {} materials.", name, t_count, m_count)
                }
                Err(e) => return Err(format!("Failed to import CapCut draft: {}", e)),
            }
        }
        "export_capcut_draft" => {
            let output_path = args
                .get("output_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'output_path'".to_string())?;

            match export_to_capcut_json(&eng.project) {
                Ok(json_str) => {
                    if let Err(e) = std::fs::write(output_path, json_str) {
                        return Err(format!("Failed to write CapCut draft file: {}", e));
                    }
                    format!("📁 Exported CapCut draft successfully to `{}`", output_path)
                }
                Err(e) => return Err(format!("Failed to serialize CapCut draft: {}", e)),
            }
        }
        "undo" => {
            match eng.undo() {
                Ok(true) => "↩️ Undid last action successfully.".to_string(),
                Ok(false) => "Nothing to undo.".to_string(),
                Err(e) => return Err(format!("Undo failed: {}", e)),
            }
        }
        "redo" => {
            match eng.redo() {
                Ok(true) => "↪️ Redid action successfully.".to_string(),
                Ok(false) => "Nothing to redo.".to_string(),
                Err(e) => return Err(format!("Redo failed: {}", e)),
            }
        }
        "detect_scenes" => {
            let material_id = args
                .get("material_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'material_id'".to_string())?;

            let threshold = args.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.25);

            let mat = eng
                .project
                .find_material(material_id)
                .ok_or_else(|| format!("Material not found: {}", material_id))?
                .clone();

            match detect_scenes(&mat.path, threshold) {
                Ok(scenes) => {
                    let mut md = format!("🎬 **Scene Detection for {}:** Found {} visual scene cuts\n\n", mat.name, scenes.len());
                    if scenes.is_empty() {
                        md.push_str("No abrupt scene transitions detected (camera may be stationary/talking head).\n");
                    } else {
                        for (i, sc) in scenes.iter().enumerate() {
                            let m = (sc.timestamp_sec / 60.0).floor();
                            let s = sc.timestamp_sec % 60.0;
                            md.push_str(&format!("- **Scene {}**: cut at `{:02.0}:{:05.2}` ({:.2}s)\n", i + 1, m, s, sc.timestamp_sec));
                        }
                    }
                    md
                }
                Err(e) => return Err(format!("Scene detection failed: {}", e)),
            }
        }
        "detect_silence" => {
            let material_id = args
                .get("material_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'material_id'".to_string())?;

            let noise_db = args.get("noise_db").and_then(|v| v.as_f64()).unwrap_or(-30.0);
            let min_dur = args.get("min_duration_sec").and_then(|v| v.as_f64()).unwrap_or(0.4);

            let mat = eng
                .project
                .find_material(material_id)
                .ok_or_else(|| format!("Material not found: {}", material_id))?
                .clone();

            match detect_silence(&mat.path, noise_db, min_dur) {
                Ok(silences) => {
                    let total_dead_air: f64 = silences.iter().map(|s| s.duration_sec).sum();
                    let mut md = format!(
                        "🔇 **Silence Analysis for {}:** Found {} pause/hesitation intervals (Total dead air: **{:.2}s**)\n\n",
                        mat.name, silences.len(), total_dead_air
                    );
                    for (i, s) in silences.iter().take(15).enumerate() {
                        md.push_str(&format!(
                            "{}. `[{:05.2}s - {:05.2}s]` (duration: {:.2}s)\n",
                            i + 1, s.start_sec, s.end_sec, s.duration_sec
                        ));
                    }
                    if silences.len() > 15 {
                        md.push_str(&format!("... and {} more silence intervals.\n", silences.len() - 15));
                    }
                    md.push_str("\n👉 *Tip: Run `auto_rough_cut` to automatically cut out these dead air gaps!*");
                    md
                }
                Err(e) => return Err(format!("Silence detection failed: {}", e)),
            }
        }
        "auto_rough_cut" => {
            let material_id = args
                .get("material_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'material_id'".to_string())?;

            let noise_db = args.get("noise_db").and_then(|v| v.as_f64());
            let min_sil = args.get("min_silence_sec").and_then(|v| v.as_f64());
            let pad = args.get("padding_sec").and_then(|v| v.as_f64());

            let orig_duration_sec = eng
                .project
                .find_material(material_id)
                .map(|m| m.duration as f64 / 1_000_000.0)
                .unwrap_or(0.0);

            match eng.auto_rough_cut(material_id, noise_db, min_sil, pad) {
                Ok(segments) => {
                    let new_duration_sec = eng.project.duration() as f64 / 1_000_000.0;
                    let saved_sec = (orig_duration_sec - new_duration_sec).max(0.0);
                    format!(
                        "⚡ **Auto Rough Cut Complete!**\n- Extracted **{}** active speech segments\n- Cut out **{:.2}s** of hesitation & dead air\n- Video duration tightened from **{:.2}s** down to **{:.2}s** ({:.0}% retention)",
                        segments.len(), saved_sec, orig_duration_sec, new_duration_sec, (new_duration_sec / orig_duration_sec.max(0.001)) * 100.0
                    )
                }
                Err(e) => return Err(format!("Auto rough cut failed: {}", e)),
            }
        }
        "get_transcript" => {
            let material_id = args
                .get("material_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required 'material_id'".to_string())?;

            let srt_path = args.get("srt_path").and_then(|v| v.as_str());

            let mat = eng
                .project
                .find_material(material_id)
                .ok_or_else(|| format!("Material not found: {}", material_id))?
                .clone();

            match get_transcript(&mat.path, srt_path) {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!(
                            "📝 **Transcript for {}:**\nNo embedded subtitle track or .srt/.vtt file found alongside `{}`.\n\n*To provide transcripts, place a `.srt` file next to your video or supply the path via `srt_path`.*",
                            mat.name, mat.path
                        )
                    } else {
                        let mut md = format!("📝 **Transcript for {}:** ({} dialogue lines)\n\n", mat.name, entries.len());
                        for (i, entry) in entries.iter().take(25).enumerate() {
                            let m = (entry.start_sec / 60.0).floor();
                            let s = entry.start_sec % 60.0;
                            md.push_str(&format!("{}. `[{:02.0}:{:05.2}]` {}\n", i + 1, m, s, entry.text));
                        }
                        if entries.len() > 25 {
                            md.push_str(&format!("... and {} more lines.\n", entries.len() - 25));
                        }
                        md
                    }
                }
                Err(e) => return Err(format!("Failed to read transcript: {}", e)),
            }
        }
        _ => return Err(format!("Unknown tool: {}", name)),
    };

    // Auto-save project file if configured
    if let Some(path) = &project_file_path {
        if let Ok(json_data) = serde_json::to_string_pretty(&eng.project) {
            let _ = std::fs::write(path, json_data);
        }
    }

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": result_text
            }
        ]
    }))
}
