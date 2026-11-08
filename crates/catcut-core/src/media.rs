use crate::model::*;
use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tracing::info;

#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    streams: Option<Vec<FFprobeStream>>,
    format: Option<FFprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FFprobeStream {
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    duration: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FFprobeFormat {
    duration: Option<String>,
    size: Option<String>,
}

/// Find ffmpeg binary (native Linux or fallback to Windows Scoop if running in WSL)
pub fn get_ffmpeg_cmd() -> String {
    if which::which("ffmpeg").is_ok() {
        return "ffmpeg".to_string();
    }
    let win_path = "/mnt/d/apps/Scoop/apps/ffmpeg/current/bin/ffmpeg.exe";
    if Path::new(win_path).exists() {
        return win_path.to_string();
    }
    "ffmpeg".to_string()
}

/// Find ffprobe binary
pub fn get_ffprobe_cmd() -> String {
    if which::which("ffprobe").is_ok() {
        return "ffprobe".to_string();
    }
    let win_path = "/mnt/d/apps/Scoop/apps/ffmpeg/current/bin/ffprobe.exe";
    if Path::new(win_path).exists() {
        return win_path.to_string();
    }
    "ffprobe".to_string()
}

/// Parse fps string from ffprobe like "30/1" or "29.97"
fn parse_fps(rate_str: &str) -> f64 {
    if let Some((num, den)) = rate_str.split_once('/') {
        if let (Ok(n), Ok(d)) = (num.parse::<f64>(), den.parse::<f64>()) {
            if d > 0.0 {
                return n / d;
            }
        }
    }
    rate_str.parse::<f64>().unwrap_or(30.0)
}

/// Probe media metadata using ffprobe
pub fn probe_media(file_path: &str) -> Result<MediaMaterial> {
    let ffprobe = get_ffprobe_cmd();
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_path,
        ])
        .output()
        .with_context(|| format!("Failed to run ffprobe on {}", file_path))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe error: {}", err));
    }

    let parsed: FFprobeOutput = serde_json::from_slice(&output.stdout)
        .with_context(|| "Failed to parse ffprobe json output")?;

    let path_obj = Path::new(file_path);
    let filename = path_obj
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "media".to_string());

    let streams = parsed.streams.unwrap_or_default();
    let format = parsed.format;

    let mut has_video = false;
    let mut has_audio = false;
    let mut width = 1920;
    let mut height = 1080;
    let mut fps = 30.0;
    let mut sample_rate = 44100;
    let mut channels = 2;
    let mut stream_duration_sec = 0.0f64;

    for s in &streams {
        match s.codec_type.as_deref() {
            Some("video") => {
                has_video = true;
                if let (Some(w), Some(h)) = (s.width, s.height) {
                    width = w;
                    height = h;
                }
                if let Some(r) = &s.r_frame_rate {
                    fps = parse_fps(r);
                }
                if let Some(d) = &s.duration {
                    if let Ok(sec) = d.parse::<f64>() {
                        stream_duration_sec = stream_duration_sec.max(sec);
                    }
                }
            }
            Some("audio") => {
                has_audio = true;
                if let Some(sr) = &s.sample_rate {
                    if let Ok(r) = sr.parse::<u32>() {
                        sample_rate = r;
                    }
                }
                if let Some(c) = s.channels {
                    channels = c;
                }
                if let Some(d) = &s.duration {
                    if let Ok(sec) = d.parse::<f64>() {
                        stream_duration_sec = stream_duration_sec.max(sec);
                    }
                }
            }
            _ => {}
        }
    }

    // Fallback to format duration if streams didn't specify it
    let total_duration_sec = if stream_duration_sec > 0.0 {
        stream_duration_sec
    } else if let Some(fmt) = format {
        fmt.duration
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let duration_us = (total_duration_sec * MICROSECONDS_PER_SECOND as f64) as i64;
    let material_type = if has_video {
        MaterialType::Video
    } else if has_audio {
        MaterialType::Audio
    } else {
        // Check extension for image
        let ext = path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ["jpg", "jpeg", "png", "webp", "gif"].contains(&ext.as_str()) {
            MaterialType::Image
        } else {
            MaterialType::Video
        }
    };

    let mut mat = MediaMaterial::new(
        material_type,
        file_path.to_string(),
        filename,
        duration_us,
        width,
        height,
        fps,
        has_audio,
    );
    mat.sample_rate = sample_rate;
    mat.channels = channels;

    Ok(mat)
}

/// Extract a single frame at a specific timestamp (seconds) and return as base64 PNG data
pub fn extract_frame_base64(file_path: &str, timestamp_seconds: f64, max_width: u32) -> Result<String> {
    let ffmpeg = get_ffmpeg_cmd();
    let vf = format!("scale='min(iw,{})':-1", max_width);
    let time_str = format!("{:.3}", timestamp_seconds.max(0.0));

    let output = Command::new(&ffmpeg)
        .args([
            "-ss",
            &time_str,
            "-i",
            file_path,
            "-vframes",
            "1",
            "-vf",
            &vf,
            "-f",
            "image2",
            "-c:v",
            "png",
            "pipe:1",
        ])
        .output()
        .with_context(|| format!("Failed to extract frame at {}s from {}", timestamp_seconds, file_path))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffmpeg frame extraction failed: {}", err));
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&output.stdout);
    Ok(format!("data:image/png;base64,{}", b64))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub output_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub crf: Option<u32>,
    pub preset: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_path: "output.mp4".to_string(),
            width: None,
            height: None,
            fps: None,
            crf: Some(23),
            preset: Some("medium".to_string()),
        }
    }
}

/// Export project to MP4 using FFmpeg
pub fn export_project(project: &Project, options: &ExportOptions) -> Result<()> {
    let ffmpeg = get_ffmpeg_cmd();
    let width = options.width.unwrap_or(project.width);
    let height = options.height.unwrap_or(project.height);
    let fps = options.fps.unwrap_or(project.fps);
    let crf = options.crf.unwrap_or(23);
    let preset = options.preset.as_deref().unwrap_or("medium");

    // Gather all unique video & audio segments
    let mut video_clips: Vec<(&ClipSegment, &MediaMaterial)> = Vec::new();
    let mut audio_clips: Vec<(&ClipSegment, &MediaMaterial)> = Vec::new();

    for track in &project.tracks {
        for seg in &track.segments {
            if let Some(mat) = project.find_material(&seg.material_id) {
                match track.track_type {
                    TrackType::Video => video_clips.push((seg, mat)),
                    TrackType::Audio => audio_clips.push((seg, mat)),
                    TrackType::Text => {}
                }
            }
        }
    }

    if video_clips.is_empty() && audio_clips.is_empty() {
        bail!("Timeline is empty, nothing to export!");
    }

    // Sort video clips by timeline start
    video_clips.sort_by_key(|(s, _)| s.target_timerange.start);

    // Build FFmpeg command inputs
    let mut args: Vec<String> = vec!["-y".to_string()];
    let mut input_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (_, mat) in video_clips.iter().chain(audio_clips.iter()) {
        if !input_map.contains_key(&mat.path) {
            let idx = input_map.len();
            input_map.insert(mat.path.clone(), idx);
            args.push("-i".to_string());
            args.push(mat.path.clone());
        }
    }

    // Build filter complex
    let mut filter_steps = Vec::new();
    let mut v_concat_labels = Vec::new();

    for (i, (seg, mat)) in video_clips.iter().enumerate() {
        let input_idx = input_map[&mat.path];
        let (src_start_sec, src_dur_sec) = seg.source_timerange.to_seconds();
        let src_end_sec = src_start_sec + src_dur_sec;

        let v_filter = format!(
            "[{}:v]trim=start={:.3}:end={:.3},setpts=PTS-STARTPTS,scale={}x{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1",
            input_idx, src_start_sec, src_end_sec, width, height, width, height
        );

        let v_label = format!("v_seg_{}", i);
        if (seg.speed - 1.0).abs() > 0.001 {
            filter_steps.push(format!("{}[{}_pre];[{}_pre]setpts=PTS/{:.4}[{}]", v_filter, v_label, v_label, seg.speed, v_label));
        } else {
            filter_steps.push(format!("{}[{}]", v_filter, v_label));
        }
        v_concat_labels.push(format!("[{}]", v_label));
    }

    if !v_concat_labels.is_empty() {
        let concat_filter = format!(
            "{}concat=n={}:v=1:a=0[vout]",
            v_concat_labels.join(""),
            v_concat_labels.len()
        );
        filter_steps.push(concat_filter);
    }

    // Handle audio clips
    let mut a_concat_labels = Vec::new();
    for (i, (seg, mat)) in audio_clips.iter().enumerate() {
        let input_idx = input_map[&mat.path];
        let (src_start_sec, src_dur_sec) = seg.source_timerange.to_seconds();
        let src_end_sec = src_start_sec + src_dur_sec;

        let a_label = format!("a_seg_{}", i);
        let vol_filter = format!("volume={:.2}", seg.audio.volume);
        let a_filter = format!(
            "[{}:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS,{}[{}]",
            input_idx, src_start_sec, src_end_sec, vol_filter, a_label
        );
        filter_steps.push(a_filter);
        a_concat_labels.push(format!("[{}]", a_label));
    }

    if !a_concat_labels.is_empty() {
        let a_concat = format!(
            "{}concat=n={}:v=0:a=1[aout]",
            a_concat_labels.join(""),
            a_concat_labels.len()
        );
        filter_steps.push(a_concat);
    }

    // Assemble final args
    if !filter_steps.is_empty() {
        args.push("-filter_complex".to_string());
        args.push(filter_steps.join(";"));
    }

    if !v_concat_labels.is_empty() {
        args.push("-map".to_string());
        args.push("[vout]".to_string());
        args.push("-c:v".to_string());
        args.push("libx264".to_string());
        args.push("-crf".to_string());
        args.push(crf.to_string());
        args.push("-preset".to_string());
        args.push(preset.to_string());
        args.push("-r".to_string());
        args.push(format!("{:.2}", fps));
        args.push("-pix_fmt".to_string());
        args.push("yuv420p".to_string());
    }

    if !a_concat_labels.is_empty() {
        args.push("-map".to_string());
        args.push("[aout]".to_string());
        args.push("-c:a".to_string());
        args.push("aac".to_string());
        args.push("-b:a".to_string());
        args.push("192k".to_string());
    }

    args.push(options.output_path.clone());

    info!("Executing FFmpeg export: {} {}", ffmpeg, args.join(" "));
    let output = Command::new(&ffmpeg)
        .args(&args)
        .output()
        .with_context(|| "Failed to execute ffmpeg export command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("FFmpeg export failed:\n{}", err);
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceInterval {
    pub start_sec: f64,
    pub end_sec: f64,
    pub duration_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCut {
    pub timestamp_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInterval {
    pub start_sec: f64,
    pub end_sec: f64,
    pub duration_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

/// Detect silence intervals in an audio/video file using FFmpeg silencedetect filter
pub fn detect_silence(
    file_path: &str,
    noise_db: f64,
    min_duration_sec: f64,
) -> Result<Vec<SilenceInterval>> {
    let ffmpeg = get_ffmpeg_cmd();
    let filter = format!("silencedetect=noise={:.1}dB:d={:.2}", noise_db, min_duration_sec);

    let output = Command::new(&ffmpeg)
        .args(["-i", file_path, "-af", &filter, "-f", "null", "-"])
        .output()
        .with_context(|| format!("Failed to run silence detection on {}", file_path))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut intervals = Vec::new();
    let mut current_start: Option<f64> = None;

    for line in stderr.lines() {
        if let Some(pos) = line.find("silence_start:") {
            let val_str = line[pos + "silence_start:".len()..].trim();
            if let Some(token) = val_str.split_whitespace().next() {
                if let Ok(val) = token.parse::<f64>() {
                    current_start = Some(val);
                }
            }
        } else if let Some(pos) = line.find("silence_end:") {
            let rest = &line[pos + "silence_end:".len()..];
            let parts: Vec<&str> = rest.split('|').collect();
            if let Some(end_token) = parts[0].trim().split_whitespace().next() {
                if let Ok(end_val) = end_token.parse::<f64>() {
                    let start_val = current_start.unwrap_or(0.0);
                    let dur = (end_val - start_val).max(0.0);
                    intervals.push(SilenceInterval {
                        start_sec: start_val,
                        end_sec: end_val,
                        duration_sec: dur,
                    });
                    current_start = None;
                }
            }
        }
    }

    Ok(intervals)
}

/// Detect scene cuts/transitions in video using FFmpeg scene filter
pub fn detect_scenes(file_path: &str, threshold: f64) -> Result<Vec<SceneCut>> {
    let ffmpeg = get_ffmpeg_cmd();
    let th = threshold.clamp(0.01, 1.0);
    let vf = format!("scale=320:-2,select='gt(scene\\,{:.2})',showinfo", th);
    let output = Command::new(&ffmpeg)
        .args(["-i", file_path, "-vf", &vf, "-f", "null", "-"])
        .output()
        .with_context(|| format!("Failed to run scene detection on {}", file_path))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut cuts = Vec::new();

    for line in stderr.lines() {
        if line.contains("Parsed_showinfo") && line.contains("pts_time:") {
            if let Some(pos) = line.find("pts_time:") {
                let val_str = &line[pos + "pts_time:".len()..];
                if let Some(token) = val_str.split_whitespace().next() {
                    if let Ok(t) = token.parse::<f64>() {
                        // Only keep if distinct from previous cut (>0.3s)
                        if cuts.is_empty() || (t - cuts.last().map(|c: &SceneCut| c.timestamp_sec).unwrap_or(0.0)) > 0.3 {
                            cuts.push(SceneCut { timestamp_sec: t });
                        }
                    }
                }
            }
        }
    }

    Ok(cuts)
}

/// Calculate speech/voice intervals by inverting silence intervals with optional padding
pub fn extract_voice_intervals(
    file_path: &str,
    total_duration_sec: f64,
    noise_db: f64,
    min_silence_sec: f64,
    padding_sec: f64,
) -> Result<Vec<VoiceInterval>> {
    let silences = detect_silence(file_path, noise_db, min_silence_sec)?;
    let mut voices = Vec::new();
    let mut current_pos = 0.0f64;
    let pad = padding_sec.max(0.0);

    for sil in silences {
        let voice_start = current_pos;
        let voice_end = (sil.start_sec + pad).min(sil.end_sec).min(total_duration_sec);

        if voice_end > voice_start + 0.15 {
            voices.push(VoiceInterval {
                start_sec: voice_start,
                end_sec: voice_end,
                duration_sec: voice_end - voice_start,
            });
        }

        current_pos = (sil.end_sec - pad).max(voice_end).min(total_duration_sec);
    }

    if current_pos < total_duration_sec - 0.15 {
        voices.push(VoiceInterval {
            start_sec: current_pos,
            end_sec: total_duration_sec,
            duration_sec: total_duration_sec - current_pos,
        });
    }

    Ok(voices)
}

fn parse_srt_timestamp(ts: &str) -> Option<f64> {
    let cleaned = ts.trim().replace(',', ".");
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().ok()?;
        let m: f64 = parts[1].parse().ok()?;
        let s: f64 = parts[2].parse().ok()?;
        Some(h * 3600.0 + m * 60.0 + s)
    } else {
        None
    }
}

/// Extract transcript from embedded subtitles or accompanying SRT/VTT file
pub fn get_transcript(
    file_path: &str,
    srt_override_path: Option<&str>,
) -> Result<Vec<TranscriptEntry>> {
    let path_to_check = if let Some(p) = srt_override_path {
        Some(std::path::PathBuf::from(p))
    } else {
        let p = std::path::Path::new(file_path);
        let srt_candidate = p.with_extension("srt");
        let vtt_candidate = p.with_extension("vtt");
        if srt_candidate.exists() {
            Some(srt_candidate)
        } else if vtt_candidate.exists() {
            Some(vtt_candidate)
        } else {
            None
        }
    };

    let srt_content = if let Some(path) = path_to_check {
        std::fs::read_to_string(path).context("Failed to read subtitle file")?
    } else {
        // Try to dump embedded subtitles using ffmpeg
        let ffmpeg = get_ffmpeg_cmd();
        let output = Command::new(&ffmpeg)
            .args(["-i", file_path, "-map", "0:s:0", "-f", "srt", "pipe:1"])
            .output();

        if let Ok(out) = output {
            if out.status.success() && !out.stdout.is_empty() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }
    };

    let mut entries = Vec::new();
    let mut lines = srt_content.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.contains("-->") {
            let times: Vec<&str> = line.split("-->").collect();
            if times.len() == 2 {
                let start = parse_srt_timestamp(times[0]).unwrap_or(0.0);
                let end = parse_srt_timestamp(times[1]).unwrap_or(0.0);

                let mut text_lines = Vec::new();
                while let Some(next_l) = lines.peek() {
                    if next_l.trim().is_empty() {
                        break;
                    }
                    text_lines.push(lines.next().unwrap().trim().to_string());
                }

                entries.push(TranscriptEntry {
                    start_sec: start,
                    end_sec: end,
                    text: text_lines.join(" "),
                });
            }
        }
    }

    Ok(entries)
}
