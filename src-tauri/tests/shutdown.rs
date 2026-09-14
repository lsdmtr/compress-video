#[cfg(unix)]
#[test]
fn shutdown_waits_for_import_probe_and_prevents_new_children() {
    use framefold_lib::media;
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let dir = std::env::temp_dir().join(format!("framefold-shutdown-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let binary = dir.join("slow-ffprobe");
    std::fs::write(&binary, b"#!/bin/sh\nexec sleep 30\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    let input = dir.join("video.mp4");
    std::fs::write(&input, b"dummy input").unwrap();
    let probe_binary = binary.clone();
    let probe_input = input.clone();
    let probe = std::thread::spawn(move || media::probe(&probe_binary, &probe_input));
    let start = Instant::now();
    while media::active_processes() == 0 {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    }
    media::shutdown();
    assert!(probe.join().unwrap().is_err());
    assert_eq!(media::active_processes(), 0);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(media::probe(&binary, &input).is_err());
    assert_eq!(media::active_processes(), 0);
    std::fs::remove_dir_all(dir).unwrap();
}
