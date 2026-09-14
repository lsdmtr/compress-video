pub mod encoding;
pub mod media;
pub mod model;
pub mod queue;
#[cfg(feature = "desktop")]
mod desktop {
    use super::*;
    use std::{path::Path, sync::Arc};
    use tauri::{Emitter, Manager, State};
    fn notify(app: tauri::AppHandle) -> queue::Listener {
        Arc::new(move |snapshot| {
            let _ = app.emit("queue-update", snapshot);
        })
    }
    #[tauri::command]
    async fn get_environment() -> model::Environment {
        tauri::async_runtime::spawn_blocking(media::environment)
            .await
            .unwrap()
    }
    #[tauri::command]
    async fn probe_files(paths: Vec<String>) -> Vec<model::ProbeResult> {
        tauri::async_runtime::spawn_blocking(move || {
            let probe = media::resolve("ffprobe");
            paths
                .into_iter()
                .map(|path| {
                    let result = probe
                        .as_ref()
                        .ok_or("未找到 ffprobe，请检查安装".to_string())
                        .and_then(|p| media::probe(p, Path::new(&path)));
                    match result {
                        Ok(media) => model::ProbeResult {
                            path,
                            media: Some(media),
                            error: None,
                        },
                        Err(error) => model::ProbeResult {
                            path,
                            media: None,
                            error: Some(error),
                        },
                    }
                })
                .collect()
        })
        .await
        .unwrap()
    }
    #[tauri::command]
    fn get_queue(engine: State<'_, queue::Engine>) -> model::QueueSnapshot {
        engine.snapshot()
    }
    #[tauri::command]
    fn start_batch(
        items: Vec<model::MediaInfo>,
        settings: model::Settings,
        engine: State<'_, queue::Engine>,
        app: tauri::AppHandle,
    ) -> Result<model::QueueSnapshot, String> {
        engine.start(items, settings, notify(app))
    }
    #[tauri::command]
    fn cancel_job(
        id: String,
        engine: State<'_, queue::Engine>,
        app: tauri::AppHandle,
    ) -> model::QueueSnapshot {
        let snapshot = engine.cancel(&id);
        let _ = app.emit("queue-update", &snapshot);
        snapshot
    }
    #[tauri::command]
    fn stop_batch(engine: State<'_, queue::Engine>, app: tauri::AppHandle) -> model::QueueSnapshot {
        let snapshot = engine.stop();
        let _ = app.emit("queue-update", &snapshot);
        snapshot
    }
    #[tauri::command]
    fn open_output(path: String) -> Result<(), String> {
        let path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        if !path.is_file() {
            return Err("输出文件不存在".into());
        }
        #[cfg(target_os = "windows")]
        let result = std::process::Command::new("explorer.exe")
            .arg(format!("/select,{}", path.display()))
            .spawn();
        #[cfg(target_os = "macos")]
        let result = std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn();
        #[cfg(target_os = "linux")]
        let result = std::process::Command::new("xdg-open")
            .arg(path.parent().unwrap())
            .spawn();
        result.map(|_| ()).map_err(|e| e.to_string())
    }
    pub fn run() {
        // Fixed WebView2 v120+ requires AppContainer read/execute access on
        // Windows 10. Keep the sandbox enabled and restrict the grant to our
        // bundled runtime, following Microsoft's distribution documentation.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let executable = std::env::current_exe().expect("无法定位应用程序");
            let runtime = executable.parent().unwrap().join("WebView2Runtime");
            if runtime.join("msedgewebview2.exe").is_file() {
                let icacls = std::path::PathBuf::from(
                    std::env::var_os("SystemRoot").expect("缺少 Windows 系统目录"),
                )
                .join("System32/icacls.exe");
                let status = std::process::Command::new(icacls)
                    .arg(&runtime)
                    .args([
                        "/grant",
                        "*S-1-15-2-2:(OI)(CI)(RX)",
                        "*S-1-15-2-1:(OI)(CI)(RX)",
                        "/Q",
                    ])
                    .creation_flags(0x08000000)
                    .status()
                    .expect("无法设置 WebView2 沙箱读取权限");
                assert!(
                    status.success(),
                    "请将完整软件包解压到当前用户可写的本地文件夹"
                );
            }
        }
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .manage(queue::Engine::default())
            .invoke_handler(tauri::generate_handler![
                get_environment,
                probe_files,
                get_queue,
                start_batch,
                cancel_job,
                stop_batch,
                open_output
            ])
            .build(tauri::generate_context!())
            .expect("无法启动 FrameFold")
            .run(|app, event| {
                if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                    let engine = app.state::<queue::Engine>();
                    engine.shutdown();
                    let start = std::time::Instant::now();
                    while (engine.snapshot().running || media::active_processes() > 0)
                        && start.elapsed() < std::time::Duration::from_secs(12)
                    {
                        std::thread::sleep(std::time::Duration::from_millis(30));
                    }
                }
            });
    }
}
#[cfg(feature = "desktop")]
pub use desktop::run;
