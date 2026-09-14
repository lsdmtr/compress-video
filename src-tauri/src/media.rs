use crate::model::*;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
static SHUTTING_DOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PROCESS_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
static ACTIVE_CAPTURES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn active_processes() -> usize {
    ACTIVE_CAPTURES.load(std::sync::atomic::Ordering::SeqCst)
}
pub fn shutdown() {
    let _gate = PROCESS_GATE.lock().unwrap();
    SHUTTING_DOWN.store(true, std::sync::atomic::Ordering::SeqCst);
}
struct CaptureGuard;
impl Drop for CaptureGuard {
    fn drop(&mut self) {
        ACTIVE_CAPTURES.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}
fn capture(
    mut command: Command,
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> Result<std::process::Output, String> {
    use std::io::Read;
    let gate = PROCESS_GATE.lock().unwrap();
    if SHUTTING_DOWN.load(std::sync::atomic::Ordering::SeqCst) || cancelled() {
        return Err("媒体探测已停止".into());
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    ACTIVE_CAPTURES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _capture_guard = CaptureGuard;
    drop(gate);
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let output_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        bytes
    });
    let error_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let start = Instant::now();
    let result = loop {
        if SHUTTING_DOWN.load(std::sync::atomic::Ordering::SeqCst)
            || cancelled()
            || start.elapsed() > timeout
        {
            let _ = child.kill();
            let _ = child.wait();
            break Err("媒体探测已停止或超时".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(e.to_string());
            }
        }
    };
    let stdout = output_reader.join().unwrap_or_default();
    let stderr = error_reader.join().unwrap_or_default();
    result.map(|status| std::process::Output {
        status,
        stdout,
        stderr,
    })
}
pub fn command(binary: &Path) -> Command {
    let mut c = Command::new(binary);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000);
    }
    c.stdin(Stdio::null());
    c
}
pub fn resolve(name: &str) -> Option<PathBuf> {
    let key = format!("VIDEO_COMPRESS_{}", name.to_uppercase());
    if let Some(p) = std::env::var_os(key) {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    let filename = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for p in [
                dir.join(&filename),
                dir.join("binaries").join(&filename),
                dir.join("../Resources/binaries").join(&filename),
            ] {
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(&filename))
            .find(|p| p.is_file())
    })
}
pub fn nvidia_test(ffmpeg: &Path, codec: &str) -> Result<(), String> {
    nvidia_test_cancellable(ffmpeg, codec, || false)
}
pub fn nvidia_test_cancellable(
    ffmpeg: &Path,
    codec: &str,
    cancelled: impl Fn() -> bool,
) -> Result<(), String> {
    let encoder = if codec == "hevc" {
        "hevc_nvenc"
    } else {
        "h264_nvenc"
    };
    let mut cmd = command(ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=size=128x128:rate=10",
        "-frames:v",
        "2",
        "-c:v",
        encoder,
        "-f",
        "null",
        "-",
    ]);
    let output = capture(cmd, Duration::from_secs(10), cancelled)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn environment() -> Environment {
    let f = resolve("ffmpeg");
    let p = resolve("ffprobe");
    let n = f
        .as_ref()
        .map(|p| nvidia_test(p, "h264"))
        .unwrap_or(Err("未找到 FFmpeg".into()));
    Environment {
        ffmpeg_available: f.is_some(),
        ffprobe_available: p.is_some(),
        nvidia_available: n.is_ok(),
        nvidia_reason: n.err(),
        ffmpeg_path: f.map(|p| p.to_string_lossy().into_owned()),
        ffprobe_path: p.map(|p| p.to_string_lossy().into_owned()),
    }
}
fn rational(s: &str) -> Option<f64> {
    let parts: Vec<_> = s.split(['/', ':']).collect();
    let a = parts.first()?.parse::<f64>().ok()?;
    let b = if parts.len() > 1 {
        parts[1].parse::<f64>().ok()?
    } else {
        1.
    };
    let x = a / b;
    if x.is_finite() && x > 0. {
        Some(x)
    } else {
        None
    }
}
pub fn probe(binary: &Path, path: &Path) -> Result<MediaInfo, String> {
    probe_cancellable(binary, path, || false)
}
pub fn probe_cancellable(
    binary: &Path,
    path: &Path,
    cancelled: impl Fn() -> bool,
) -> Result<MediaInfo, String> {
    if cancelled() {
        return Err("任务已取消".into());
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("无法读取文件：{e}"))?;
    if !meta.is_file() {
        return Err("请选择视频文件".into());
    }
    let mut cmd = command(binary);
    cmd.args([
        "-v",
        "error",
        "-show_streams",
        "-show_format",
        "-of",
        "json",
    ])
    .arg(path);
    let output = capture(cmd, Duration::from_secs(30), &cancelled)?;
    if !output.status.success() {
        return Err(format!(
            "视频探测失败：{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let streams = json["streams"].as_array().ok_or("无法读取媒体流")?;
    let v = streams
        .iter()
        .find(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1)
        .ok_or("文件没有视频流")?;
    let audio: Vec<_> = streams
        .iter()
        .filter(|s| s["codec_type"] == "audio")
        .collect();
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(rational)
        .or_else(|| v["duration"].as_str().and_then(rational));
    let rotation = v["side_data_list"]
        .as_array()
        .and_then(|list| list.iter().find_map(|x| x["rotation"].as_i64()))
        .or_else(|| v["tags"]["rotate"].as_str().and_then(|x| x.parse().ok()))
        .unwrap_or(0) as i32;
    let pix = v["pix_fmt"].as_str().unwrap_or("");
    // HDR mastering data and even transfer tags can live only in decoded frame SEI.
    let mut first_frame = Value::Null;
    if pix.contains("10") || pix.contains("12") || v["color_primaries"] == "bt2020" {
        let mut cmd = command(binary);
        cmd.args([
            "-v",
            "error",
            "-select_streams",
            "V:0",
            "-show_frames",
            "-read_intervals",
            "%+#1",
            "-of",
            "json",
        ])
        .arg(path);
        let frames = capture(cmd, Duration::from_secs(30), &cancelled)?;
        if !frames.status.success() {
            return Err(format!(
                "无法检查 HDR 帧元数据：{}",
                String::from_utf8_lossy(&frames.stderr)
            ));
        }
        let parsed: Value = serde_json::from_slice(&frames.stdout).map_err(|e| e.to_string())?;
        first_frame = parsed["frames"]
            .as_array()
            .and_then(|frames| frames.first())
            .cloned()
            .unwrap_or(Value::Null);
    }
    let transfer = first_frame["color_transfer"]
        .as_str()
        .or_else(|| v["color_transfer"].as_str())
        .map(String::from);
    let hdr = matches!(transfer.as_deref(), Some("smpte2084" | "arib-std-b67"));
    let mut hdr_metadata = v["side_data_list"].as_array().cloned().unwrap_or_default();
    for data in first_frame["side_data_list"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if !hdr_metadata.contains(data) {
            hdr_metadata.push(data.clone());
        }
    }
    Ok(MediaInfo {
        path: path.to_string_lossy().into_owned(),
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        size: meta.len(),
        duration,
        width: v["width"].as_u64().ok_or("缺少视频宽度")? as u32,
        height: v["height"].as_u64().ok_or("缺少视频高度")? as u32,
        fps: v["avg_frame_rate"].as_str().and_then(rational),
        codec: v["codec_name"].as_str().unwrap_or("unknown").into(),
        audio_tracks: audio.len(),
        hdr,
        bit_depth: if pix.contains("10") {
            10
        } else if pix.contains("12") {
            12
        } else {
            8
        },
        audio_codecs: audio
            .iter()
            .map(|a| a["codec_name"].as_str().unwrap_or("unknown").into())
            .collect(),
        rotation,
        audio_metadata: audio
            .iter()
            .map(|a| AudioMetadata {
                title: a["tags"]["title"]
                    .as_str()
                    .or_else(|| a["tags"]["handler_name"].as_str())
                    .map(String::from),
                language: a["tags"]["language"].as_str().map(String::from),
            })
            .collect(),
        color_transfer: transfer,
        hdr_metadata,
        sar: v["sample_aspect_ratio"]
            .as_str()
            .and_then(rational)
            .unwrap_or(1.),
    })
}
