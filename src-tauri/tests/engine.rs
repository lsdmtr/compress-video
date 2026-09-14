use framefold_lib::{
    encoding::{build_plan, dimensions, unique_output},
    model::*,
};
#[test]
fn scale_portrait_and_no_upscale() {
    assert_eq!(
        dimensions(2160, 3840, 1.0, 0, "1080").unwrap(),
        (1080, 1920)
    );
    assert_eq!(dimensions(640, 360, 1.0, 0, "1080").unwrap(), (640, 360));
    assert_eq!(dimensions(1920, 1080, 1.0, 90, "720").unwrap(), (720, 1280));
}
#[test]
fn settings_and_audio_contract() {
    let m = MediaInfo::fixture();
    let mut s = Settings {
        fps: Some(0.0),
        ..Default::default()
    };
    assert!(build_plan(&m, &s, false).is_err());
    s.fps = None;
    let p = build_plan(&m, &s, false).unwrap();
    assert!(p.args.windows(2).any(|a| a == ["-c:a:0", "copy"]));
    assert!(p.args.windows(2).any(|a| a == ["-c:a:1", "aac"]));
    assert_eq!(p.encoder, "libx264");
    s.device = "nvidia".into();
    assert!(build_plan(&m, &s, false).is_err());
    assert_eq!(build_plan(&m, &s, true).unwrap().encoder, "h264_nvenc");
}
#[test]
fn avoids_existing_output() {
    let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("test_compressed.mp4"), b"keep").unwrap();
    assert_eq!(
        unique_output(&dir, "test.mp4").file_name().unwrap(),
        "test_compressed_2.mp4"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn frame_rate_and_hdr_contract() {
    let mut m = MediaInfo::fixture();
    let mut s = Settings::default();
    let p = build_plan(&m, &s, false).unwrap();
    assert!(p.args.windows(2).any(|a| a == ["-fps_mode", "passthrough"]));
    s.fps = Some(29.97);
    let p = build_plan(&m, &s, false).unwrap();
    assert!(p.args.iter().any(|a| a.contains("fps=29.97")));
    for invalid in [f64::NAN, f64::INFINITY, -1., 240.01] {
        s.fps = Some(invalid);
        assert!(build_plan(&m, &s, false).is_err());
    }
    s.fps = None;
    m.hdr = true;
    m.color_transfer = Some("smpte2084".into());
    let p = build_plan(&m, &s, false).unwrap();
    assert!(p.args.iter().any(|a| a.contains("tonemap=")));
    assert!(p.args.windows(2).any(|a| a == ["-color_trc", "bt709"]));
    s.preserve_hdr = true;
    assert!(build_plan(&m, &s, false).is_err());
    s.codec = "hevc".into();
    let p = build_plan(&m, &s, false).unwrap();
    assert!(!p.args.iter().any(|a| a.contains("tonemap=")));
    assert!(p.args.windows(2).any(|a| a == ["-color_trc", "smpte2084"]));
}
#[test]
fn anamorphic_and_odd_sizes() {
    assert_eq!(
        dimensions(720, 576, 16.0 / 15.0, 0, "original").unwrap(),
        (768, 576)
    );
    assert_eq!(
        dimensions(1921, 1081, 1., 0, "original").unwrap(),
        (1920, 1080)
    );
    assert!(dimensions(0, 1080, 1., 0, "1080").is_err());
}

#[test]
fn copy_fallback_preserves_existing_and_cleans_cancelled() {
    use framefold_lib::queue::copy_exclusive;
    let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(&dir).unwrap();
    let source = dir.join("source.mp4");
    let target = dir.join("target.mp4");
    std::fs::write(&source, b"source bytes").unwrap();
    std::fs::write(&target, b"keep").unwrap();
    assert!(copy_exclusive(&source, &target, || false).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep");
    let cancelled = dir.join("cancelled.mp4");
    assert!(copy_exclusive(&source, &cancelled, || true).is_err());
    assert!(!cancelled.exists());
    let copied = dir.join("copied.mp4");
    copy_exclusive(&source, &copied, || false).unwrap();
    assert_eq!(
        std::fs::read(&source).unwrap(),
        std::fs::read(copied).unwrap()
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cancellation_interrupts_slow_probe() {
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(&dir).unwrap();
    let binary = dir.join("slow-ffprobe");
    std::fs::write(&binary, b"#!/bin/sh\nexec sleep 30\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    let input = dir.join("video.mp4");
    std::fs::write(&input, b"dummy input").unwrap();
    let start = Instant::now();
    let result = framefold_lib::media::probe_cancellable(&binary, &input, || {
        start.elapsed() > Duration::from_millis(80)
    });
    assert!(result.is_err());
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "cancel waited for the probe timeout"
    );
    std::fs::remove_dir_all(dir).unwrap();
}
