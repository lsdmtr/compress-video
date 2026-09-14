use crate::{encoding, media, model::*};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
#[derive(Default)]
struct Inner {
    snapshot: QueueSnapshot,
    cancelled: HashSet<String>,
    shutdown: bool,
}
#[derive(Clone, Default)]
pub struct Engine {
    inner: Arc<Mutex<Inner>>,
}
pub type Listener = Arc<dyn Fn(QueueSnapshot) + Send + Sync>;
impl Engine {
    pub fn snapshot(&self) -> QueueSnapshot {
        self.inner.lock().unwrap().snapshot.clone()
    }
    fn update(&self, id: &str, f: impl FnOnce(&mut Job), notify: &Listener) {
        let snap = {
            let mut inner = self.inner.lock().unwrap();
            if let Some(j) = inner.snapshot.jobs.iter_mut().find(|j| j.id == id) {
                if j.status != "cancelled" {
                    f(j)
                }
            }
            inner.snapshot.revision += 1;
            inner.snapshot.clone()
        };
        notify(snap);
    }
    fn cancelled(&self, id: &str) -> bool {
        let i = self.inner.lock().unwrap();
        i.shutdown || i.cancelled.contains(id)
    }
    pub fn cancel(&self, id: &str) -> QueueSnapshot {
        let mut i = self.inner.lock().unwrap();
        i.cancelled.insert(id.into());
        if let Some(j) = i
            .snapshot
            .jobs
            .iter_mut()
            .find(|j| j.id == id && j.status == "pending")
        {
            j.status = "cancelled".into()
        }
        i.snapshot.revision += 1;
        i.snapshot.clone()
    }
    pub fn stop(&self) -> QueueSnapshot {
        let ids: Vec<_> = self
            .snapshot()
            .jobs
            .iter()
            .filter(|j| ["pending", "encoding", "validating"].contains(&j.status.as_str()))
            .map(|j| j.id.clone())
            .collect();
        for id in ids {
            self.cancel(&id);
        }
        self.snapshot()
    }
    pub fn shutdown(&self) {
        media::shutdown();
        self.inner.lock().unwrap().shutdown = true;
        self.stop();
    }
    pub fn start(
        &self,
        items: Vec<MediaInfo>,
        settings: Settings,
        notify: Listener,
    ) -> Result<QueueSnapshot, String> {
        if items.is_empty() {
            return Err("请先添加视频".into());
        }
        if settings.output_dir.trim().is_empty() {
            return Err("请选择输出目录".into());
        }
        encoding::build_plan(&items[0], &settings, true)?;
        let ffmpeg = media::resolve("ffmpeg").ok_or("未找到 FFmpeg，请检查安装")?;
        let ffprobe = media::resolve("ffprobe").ok_or("未找到 ffprobe，请检查安装")?;
        std::fs::create_dir_all(&settings.output_dir)
            .map_err(|e| format!("无法创建输出目录：{e}"))?;
        let snapshot = {
            let mut i = self.inner.lock().unwrap();
            if i.snapshot.running {
                return Err("已有批次正在处理".into());
            }
            i.cancelled.clear();
            i.shutdown = false;
            i.snapshot = QueueSnapshot {
                revision: i.snapshot.revision + 1,
                running: true,
                jobs: items
                    .into_iter()
                    .map(|media| Job {
                        id: uuid::Uuid::new_v4().to_string(),
                        progress: None,
                        media,
                        status: "pending".into(),
                        speed: None,
                        eta_seconds: None,
                        output_path: None,
                        output_size: None,
                        error: None,
                        encoder: None,
                    })
                    .collect(),
            };
            i.snapshot.clone()
        };
        let engine = self.clone();
        thread::spawn(move || {
            let nvidia = settings.device != "cpu"
                && media::nvidia_test_cancellable(&ffmpeg, &settings.codec, || {
                    engine
                        .snapshot()
                        .jobs
                        .iter()
                        .all(|j| engine.cancelled(&j.id))
                })
                .is_ok();
            for job in engine.snapshot().jobs {
                if engine.cancelled(&job.id) {
                    engine.update(&job.id, |j| j.status = "cancelled".into(), &notify);
                    continue;
                }
                let result = engine.run_job(&job, &settings, nvidia, &ffmpeg, &ffprobe, &notify);
                if let Err(err) = result {
                    let cancelled = engine.cancelled(&job.id);
                    engine.update(
                        &job.id,
                        |j| {
                            j.status = if cancelled { "cancelled" } else { "failed" }.into();
                            j.error = if cancelled { None } else { Some(err) };
                            j.eta_seconds = None;
                        },
                        &notify,
                    );
                }
            }
            let snap = {
                let mut i = engine.inner.lock().unwrap();
                i.snapshot.running = false;
                i.snapshot.revision += 1;
                i.snapshot.clone()
            };
            notify(snap);
        });
        Ok(snapshot)
    }
    fn run_job(
        &self,
        job: &Job,
        s: &Settings,
        nvidia: bool,
        ffmpeg: &Path,
        ffprobe: &Path,
        notify: &Listener,
    ) -> Result<(), String> {
        let m = media::probe_cancellable(ffprobe, Path::new(&job.media.path), || {
            self.cancelled(&job.id)
        })?;
        if self.cancelled(&job.id) {
            return Err("任务已取消".into());
        }
        let plan = encoding::build_plan(&m, s, nvidia)?;
        let dir = PathBuf::from(&s.output_dir);
        let temp = dir.join(format!(".framefold-{}.partial.mp4", job.id));
        let result = (|| {
            self.update(
                &job.id,
                |j| {
                    j.media = m.clone();
                    j.status = "encoding".into();
                    j.encoder = Some(plan.encoder.clone());
                    j.progress = m.duration.map(|_| 0.);
                },
                notify,
            );
            if self.cancelled(&job.id) {
                return Err("任务已取消".into());
            }
            let mut child = media::command(ffmpeg)
                .args(["-hide_banner", "-nostdin", "-n", "-i"])
                .arg(&m.path)
                .args(&plan.args)
                .args(["-progress", "pipe:1", "-nostats"])
                .arg(&temp)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("无法启动编码器：{e}"))?;
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            let logs = thread::spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut out = String::new();
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            out.push_str(&String::from_utf8_lossy(&buf[..n]));
                            if out.len() > 65536 {
                                let cut = out
                                    .char_indices()
                                    .map(|(i, _)| i)
                                    .find(|&i| i >= out.len() - 60000)
                                    .unwrap_or(0);
                                out.drain(..cut);
                            }
                        }
                    }
                }
                out
            });
            let (tx, rx) = std::sync::mpsc::channel();
            let progress = thread::spawn(move || {
                for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            });
            let mut time = 0.;
            let mut speed = None;
            let status = loop {
                if self.cancelled(&job.id) {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err("任务已取消".to_string());
                }
                while let Ok(line) = rx.try_recv() {
                    if let Some((key, val)) = line.split_once('=') {
                        match key {
                            "out_time_us" => time = val.parse::<f64>().unwrap_or(0.) / 1_000_000.,
                            "speed" => {
                                speed = val
                                    .strip_suffix('x')
                                    .and_then(|x| x.trim().parse::<f64>().ok())
                            }
                            "progress" => {
                                let fraction = m.duration.map(|d| (time / d).clamp(0., 0.99));
                                let eta = m.duration.zip(speed).and_then(|(d, sp)| {
                                    if sp > 0. {
                                        Some(((d - time) / sp).max(0.))
                                    } else {
                                        None
                                    }
                                });
                                self.update(
                                    &job.id,
                                    |j| {
                                        j.progress = fraction;
                                        j.speed = speed.map(|x| format!("{x:.2}x"));
                                        j.eta_seconds = eta;
                                    },
                                    notify,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        break if status.success() {
                            Ok(())
                        } else {
                            Err(format!("FFmpeg 编码失败（{status}）"))
                        }
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(80)),
                    Err(e) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        break Err(e.to_string());
                    }
                }
            };
            let _ = progress.join();
            let log = logs.join().unwrap_or_default();
            status.map_err(|e| {
                let hint=if log.contains("No such filter: 'zscale'") {"当前 FFmpeg 缺少 HDR 色调映射组件 zscale，请安装完整版本或选择 HEVC 保留 HDR。\n"}
                else if log.contains("No space left on device") {"输出磁盘空间不足，请释放空间或更换输出目录。\n"}
                else if log.contains("Permission denied") {"没有文件读写权限，请更换输出目录或检查源文件权限。\n"}
                else {""};
                format!("{hint}{e}\n{log}")
            })?;
            if self.cancelled(&job.id) {
                return Err("任务已取消".into());
            }
            self.update(&job.id, |j| j.status = "validating".into(), notify);
            let out = media::probe_cancellable(ffprobe, &temp, || self.cancelled(&job.id))?;
            if out.size == 0 || out.audio_tracks != m.audio_tracks {
                return Err("输出校验失败：视频为空或音轨数量不一致".into());
            }
            let expected =
                encoding::dimensions(m.width, m.height, m.sar, m.rotation, &s.resolution)?;
            if (out.width, out.height) != expected
                || out.codec != s.codec
                || out.hdr != (m.hdr && s.preserve_hdr)
            {
                return Err("输出校验失败：尺寸、编码或 HDR 色彩状态与设置不一致".into());
            }
            if let Some(fps) = s.fps {
                if out.fps.is_none_or(|actual| (actual - fps).abs() > 0.05) {
                    return Err("输出校验失败：帧率与设置不一致".into());
                }
            }
            if let (Some(a), Some(b)) = (m.duration, out.duration) {
                if (a - b).abs() > 2.0f64.max(a * 0.02) {
                    return Err("输出校验失败：时长与源视频不一致".into());
                }
            }
            if self.cancelled(&job.id) {
                return Err("任务已取消".into());
            }
            // Hard-link publication is atomic and cannot replace an existing path.
            let final_path = loop {
                let candidate = encoding::unique_output(&dir, &m.name);
                match std::fs::hard_link(&temp, &candidate) {
                    Ok(()) => break candidate,
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(_) => match copy_exclusive(&temp, &candidate, || self.cancelled(&job.id)) {
                        Ok(()) => break candidate,
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                        Err(e) => return Err(format!("无法保存输出文件：{e}")),
                    },
                }
            };
            self.update(
                &job.id,
                |j| {
                    j.status = "completed".into();
                    j.progress = Some(1.);
                    j.eta_seconds = None;
                    j.output_size = Some(out.size);
                    j.output_path = Some(final_path.to_string_lossy().into_owned());
                },
                notify,
            );
            Ok(())
        })();
        let _ = std::fs::remove_file(&temp);
        result
    }
}

/// Filesystems without hard links (such as exFAT) require a non-atomic copy.
/// create_new prevents replacement, and only our newly created file is removed on failure.
pub fn copy_exclusive(
    source: &Path,
    destination: &Path,
    cancelled: impl Fn() -> bool,
) -> std::io::Result<()> {
    use std::io::Write;
    let mut source = std::fs::File::open(source)?;
    let mut target = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let result = (|| {
        let mut buffer = vec![0u8; 1024 * 1024];
        loop {
            if cancelled() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "任务已取消",
                ));
            }
            let count = source.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            target.write_all(&buffer[..count])?;
        }
        target.sync_all()
    })();
    drop(target);
    if result.is_err() {
        let _ = std::fs::remove_file(destination);
    }
    result
}
