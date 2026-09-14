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
        #[cfg(windows)]
        if tauri::webview_version().is_err() {
            // This dialog works even when no WebView can be created. Never
            // download or install a runtime silently on the user's computer.
            #[link(name = "user32")]
            extern "system" {
                fn MessageBoxW(
                    window: *mut std::ffi::c_void,
                    text: *const u16,
                    caption: *const u16,
                    flags: u32,
                ) -> i32;
            }
            let message: Vec<u16> = "轻量版需要 Microsoft Edge WebView2 运行库。\n\n请从微软官网下载并安装 WebView2 Runtime，然后重新打开程序。\nhttps://developer.microsoft.com/microsoft-edge/webview2/\n\n程序未自动安装任何组件。"
                .encode_utf16().chain(std::iter::once(0)).collect();
            let caption: Vec<u16> = "酱菇婆专用视频压缩"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            // SAFETY: Both buffers are NUL-terminated and remain alive for the
            // synchronous call. A null HWND requests an unowned dialog.
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    message.as_ptr(),
                    caption.as_ptr(),
                    0x30,
                );
            }
            return;
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
