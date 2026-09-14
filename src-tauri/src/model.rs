use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub resolution: String,
    pub fps: Option<f64>,
    pub quality: String,
    pub device: String,
    pub codec: String,
    pub preserve_hdr: bool,
    pub output_dir: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: "1080".into(),
            fps: None,
            quality: "high".into(),
            device: "auto".into(),
            codec: "h264".into(),
            preserve_hdr: false,
            output_dir: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub duration: Option<f64>,
    pub width: u32,
    pub height: u32,
    pub fps: Option<f64>,
    pub codec: String,
    pub audio_tracks: usize,
    pub hdr: bool,
    pub bit_depth: u32,
    pub audio_codecs: Vec<String>,
    pub rotation: i32,
    pub sar: f64,
    #[serde(default)]
    pub color_transfer: Option<String>,
    #[serde(default)]
    pub hdr_metadata: Vec<serde_json::Value>,
    #[serde(default)]
    pub audio_metadata: Vec<AudioMetadata>,
}
impl MediaInfo {
    pub fn fixture() -> Self {
        Self {
            path: "test.mp4".into(),
            name: "test.mp4".into(),
            size: 100,
            duration: Some(10.),
            width: 1920,
            height: 1080,
            fps: Some(60.),
            codec: "h264".into(),
            audio_tracks: 2,
            hdr: false,
            bit_depth: 8,
            audio_codecs: vec!["aac".into(), "opus".into()],
            rotation: 0,
            sar: 1.,
            color_transfer: None,
            hdr_metadata: vec![],
            audio_metadata: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub path: String,
    pub media: Option<MediaInfo>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub ffmpeg_available: bool,
    pub ffprobe_available: bool,
    pub nvidia_available: bool,
    pub nvidia_reason: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub media: MediaInfo,
    pub status: String,
    pub progress: Option<f64>,
    pub speed: Option<String>,
    pub eta_seconds: Option<f64>,
    pub output_path: Option<String>,
    pub output_size: Option<u64>,
    pub error: Option<String>,
    pub encoder: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub language: Option<String>,
}
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub revision: u64,
    pub jobs: Vec<Job>,
    pub running: bool,
}
