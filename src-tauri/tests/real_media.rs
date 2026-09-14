//! Opt-in integration tests requiring real FFmpeg with libx264, AAC, and libopus.
use framefold_lib::{media, model::*, queue::Engine};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
struct Fixture {
    dir: PathBuf,
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    input: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let ffmpeg = media::resolve("ffmpeg").expect("real ffmpeg is required");
        let ffprobe = media::resolve("ffprobe").expect("real ffprobe is required");
        let dir =
            std::env::temp_dir().join(format!("framefold-测试 空格-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("录屏 sample.mkv");
        let out = media::command(&ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=1280x720:rate=30000/1001",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=880:sample_rate=48000",
                "-t",
                "2",
                "-map",
                "0:v",
                "-map",
                "1:a",
                "-map",
                "2:a",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-c:a:0",
                "aac",
                "-c:a:1",
                "libopus",
                "-metadata:s:a:0",
                "title=游戏声音",
                "-metadata:s:a:0",
                "language=zho",
                "-metadata:s:a:1",
                "title=麦克风",
            ])
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        Self {
            dir,
            ffmpeg,
            ffprobe,
            input,
        }
    }
    fn settings(&self) -> Settings {
        Settings {
            output_dir: self.dir.join("输出").to_string_lossy().into(),
            device: "cpu".into(),
            resolution: "720".into(),
            fps: Some(29.97),
            ..Default::default()
        }
    }
    fn probe(&self, path: &Path) -> MediaInfo {
        media::probe(&self.ffprobe, path).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
fn finish(engine: &Engine) -> QueueSnapshot {
    let start = Instant::now();
    loop {
        let q = engine.snapshot();
        if !q.running {
            return q;
        }
        assert!(
            start.elapsed() < Duration::from_secs(90),
            "queue timed out: {q:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
#[test]
#[ignore = "requires installed FFmpeg; run --ignored"]
fn real_queue_preserves_tracks_and_isolates_failure() {
    let f = Fixture::new();
    let media = f.probe(&f.input);
    assert_eq!(media.audio_tracks, 2);
    let mut bad = media.clone();
    bad.path = f.dir.join("missing.mp4").to_string_lossy().into();
    let engine = Engine::default();
    engine
        .start(vec![bad, media.clone()], f.settings(), Arc::new(|_| {}))
        .unwrap();
    let q = finish(&engine);
    assert_eq!(q.jobs[0].status, "failed");
    assert_eq!(q.jobs[1].status, "completed", "{:?}", q.jobs[1].error);
    let output = PathBuf::from(q.jobs[1].output_path.as_ref().unwrap());
    let out = f.probe(&output);
    assert_eq!((out.width, out.height), (1280, 720));
    assert_eq!(out.audio_codecs, vec!["aac", "aac"]);
    assert!((out.fps.unwrap() - 29.97).abs() < 0.02);
    assert!((out.duration.unwrap() - media.duration.unwrap()).abs() < 0.2);
    let decode = media::command(&f.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(&output)
        .args(["-map", "0:v", "-map", "0:a", "-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        decode.status.success(),
        "{}",
        String::from_utf8_lossy(&decode.stderr)
    );
    let probe = media::command(&f.ffprobe)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&output)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    let streams = json["streams"].as_array().unwrap();
    assert_eq!(streams[1]["tags"]["language"], "zho");
    assert_eq!(streams[1]["tags"]["handler_name"], "游戏声音");
    assert_eq!(streams[2]["tags"]["handler_name"], "麦克风");
    let original = std::fs::read(&f.input).unwrap();
    engine
        .start(vec![media], f.settings(), Arc::new(|_| {}))
        .unwrap();
    let next = finish(&engine);
    assert_ne!(next.jobs[0].output_path, q.jobs[1].output_path);
    assert_eq!(std::fs::read(&f.input).unwrap(), original);
    assert!(output.exists());
}
#[test]
#[ignore = "requires installed FFmpeg; run --ignored"]
fn real_cancel_cleans_temporary_output() {
    let f = Fixture::new();
    let m = f.probe(&f.input);
    let engine = Engine::default();
    let controller = engine.clone();
    engine
        .start(
            vec![m.clone(), m],
            f.settings(),
            Arc::new(move |q| {
                if q.jobs.iter().any(|j| j.status == "encoding") {
                    controller.stop();
                }
            }),
        )
        .unwrap();
    let q = finish(&engine);
    assert!(q.jobs.iter().all(|j| j.status == "cancelled"), "{q:?}");
    let names: Vec<_> = std::fs::read_dir(f.dir.join("输出"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(names.is_empty(), "owned temporary files remain: {names:?}");
    assert!(f.input.exists());
}

#[test]
#[ignore = "requires FFmpeg with libx265 and zscale; run --ignored"]
fn real_hdr_tonemap_and_preservation() {
    let f = Fixture::new();
    let hdr_input = f.dir.join("HDR.mkv");
    let result=media::command(&f.ffmpeg).args(["-v","error","-f","lavfi","-i","testsrc2=size=320x180:rate=10","-t","1","-vf","format=yuv420p10le","-c:v","libx265","-preset","ultrafast","-color_primaries","bt2020","-color_trc","smpte2084","-colorspace","bt2020nc","-x265-params","pools=2:colorprim=bt2020:transfer=smpte2084:colormatrix=bt2020nc:master-display=G(13250,34500)B(7500,3000)R(34000,16000)WP(15635,16450)L(10000000,1):max-cll=1000,400"]).arg(&hdr_input).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let hdr = f.probe(&hdr_input);
    assert!(hdr.hdr);
    assert_eq!(hdr.bit_depth, 10);
    let engine = Engine::default();
    let mut settings = f.settings();
    settings.fps = None;
    engine
        .start(vec![hdr.clone()], settings.clone(), Arc::new(|_| {}))
        .unwrap();
    let q = finish(&engine);
    let filters = media::command(&f.ffmpeg)
        .args(["-hide_banner", "-filters"])
        .output()
        .unwrap();
    if String::from_utf8_lossy(&filters.stdout).contains(" zscale ") {
        assert_eq!(q.jobs[0].status, "completed", "{:?}", q.jobs[0].error);
        let sdr = f.probe(Path::new(q.jobs[0].output_path.as_ref().unwrap()));
        assert!(!sdr.hdr);
        assert_eq!(sdr.bit_depth, 8);
        assert_eq!(sdr.color_transfer.as_deref(), Some("bt709"));
    } else {
        assert_eq!(q.jobs[0].status, "failed");
        assert!(q.jobs[0].error.as_ref().unwrap().contains("zscale"));
        assert!(q.jobs[0].output_path.is_none());
        eprintln!("zscale unavailable: confirmed safe rejection; successful tone mapping remains unverified");
    }
    settings.codec = "hevc".into();
    settings.preserve_hdr = true;
    engine.start(vec![hdr], settings, Arc::new(|_| {})).unwrap();
    let q = finish(&engine);
    assert_eq!(q.jobs[0].status, "completed", "{:?}", q.jobs[0].error);
    let preserved = f.probe(Path::new(q.jobs[0].output_path.as_ref().unwrap()));
    assert!(preserved.hdr);
    assert_eq!(preserved.bit_depth, 10);
    assert_eq!(preserved.color_transfer.as_deref(), Some("smpte2084"));
    assert!(preserved
        .hdr_metadata
        .iter()
        .any(|x| x["side_data_type"] == "Mastering display metadata"));
    assert!(
        preserved
            .hdr_metadata
            .iter()
            .any(|x| x["side_data_type"] == "Content light level metadata"
                && x["max_content"] == 1000)
    );
}

#[test]
#[ignore = "requires installed FFmpeg; run --ignored"]
fn real_rotation_is_baked_once() {
    let f = Fixture::new();
    let rotated = f.dir.join("portrait-rotation.mp4");
    let result = media::command(&f.ffmpeg)
        .args(["-v", "error", "-display_rotation", "90", "-i"])
        .arg(&f.input)
        .args([
            "-map",
            "0:v:0",
            "-c",
            "copy",
            "-metadata:s:v:0",
            "rotate=90",
        ])
        .arg(&rotated)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let input = f.probe(&rotated);
    assert_eq!(input.rotation.abs(), 90);
    let engine = Engine::default();
    engine
        .start(vec![input], f.settings(), Arc::new(|_| {}))
        .unwrap();
    let q = finish(&engine);
    assert_eq!(q.jobs[0].status, "completed", "{:?}", q.jobs[0].error);
    let output = f.probe(Path::new(q.jobs[0].output_path.as_ref().unwrap()));
    assert_eq!((output.width, output.height), (720, 1280));
    assert_eq!(output.rotation, 0);
    assert_eq!(output.sar, 1.);
}

#[test]
#[ignore = "requires FFmpeg with libaom-av1; run --ignored"]
fn real_av1_recording_transcodes_to_h264_mp4() {
    let f = Fixture::new();
    let av1 = f.dir.join("AV1 录屏.mkv");
    let result = media::command(&f.ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=10",
            "-t",
            "1",
            "-c:v",
            "libaom-av1",
            "-cpu-used",
            "8",
            "-row-mt",
            "1",
            "-threads",
            "2",
            "-crf",
            "40",
            "-b:v",
            "0",
        ])
        .arg(&av1)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let source = f.probe(&av1);
    assert_eq!(source.codec, "av1");
    let mut settings = f.settings();
    settings.fps = None;
    let engine = Engine::default();
    engine
        .start(vec![source.clone()], settings, Arc::new(|_| {}))
        .unwrap();
    let q = finish(&engine);
    assert_eq!(q.jobs[0].status, "completed", "{:?}", q.jobs[0].error);
    let output_path = Path::new(q.jobs[0].output_path.as_ref().unwrap());
    assert_eq!(output_path.extension().unwrap(), "mp4");
    let output = f.probe(output_path);
    assert_eq!(output.codec, "h264");
    assert_eq!((output.width, output.height), (320, 180));
    assert!((output.duration.unwrap() - source.duration.unwrap()).abs() < 0.11);
    let decoded = media::command(&f.ffmpeg)
        .args(["-v", "error", "-xerror", "-i"])
        .arg(output_path)
        .args(["-map", "0:v:0", "-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
}

#[test]
#[ignore = "requires installed FFmpeg; run --ignored"]
fn real_original_fps_preserves_variable_presentation_timestamps() {
    let f = Fixture::new();
    let vfr = f.dir.join("VFR 录屏.mkv");
    let result = media::command(&f.ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=30",
            "-t",
            "2",
            "-vf",
            "select='not(mod(n,2))+not(mod(n,5))'",
            "-fps_mode",
            "vfr",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-bf",
            "0",
        ])
        .arg(&vfr)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let timestamps = |path: &Path| -> Vec<f64> {
        let result = media::command(&f.ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "V:0",
                "-show_frames",
                "-show_entries",
                "frame=best_effort_timestamp_time",
                "-of",
                "json",
            ])
            .arg(path)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let data: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        data["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|frame| {
                frame["best_effort_timestamp_time"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect()
    };
    let input_pts = timestamps(&vfr);
    assert!(input_pts.len() > 10);
    let intervals: Vec<_> = input_pts.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let minimum = intervals.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = intervals.iter().copied().fold(0., f64::max);
    assert!(
        maximum > minimum * 1.8,
        "fixture must contain genuinely variable frame spacing"
    );
    let source = f.probe(&vfr);
    let mut settings = f.settings();
    settings.fps = None;
    let engine = Engine::default();
    engine
        .start(vec![source.clone()], settings, Arc::new(|_| {}))
        .unwrap();
    let q = finish(&engine);
    assert_eq!(q.jobs[0].status, "completed", "{:?}", q.jobs[0].error);
    let output_path = Path::new(q.jobs[0].output_path.as_ref().unwrap());
    let output_pts = timestamps(output_path);
    assert_eq!(
        input_pts.len(),
        output_pts.len(),
        "original FPS must not duplicate or discard frames"
    );
    for (input, output) in input_pts.iter().zip(&output_pts) {
        assert!(
            (input - output).abs() <= 0.0011,
            "presentation timestamp changed: {input} → {output}"
        );
    }
    let output = f.probe(output_path);
    assert!((source.duration.unwrap() - output.duration.unwrap()).abs() < 0.05);
}
