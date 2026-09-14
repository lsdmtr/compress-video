# Desktop IPC

All commands use Tauri `invoke`; all JSON keys are camelCase. Errors reject with a readable string. `queue-update` carries the full QueueSnapshot (also returned by get_queue/start_batch/cancel_job/stop_batch).

```ts
type Settings = { resolution: 'original'|'720'|'1080'|'1440'; fps: number|null; quality: 'high'|'balanced'|'small'; device: 'auto'|'cpu'|'nvidia'; codec: 'h264'|'hevc'; preserveHdr: boolean; outputDir: string };
type MediaInfo = { path: string; name: string; size: number; duration: number|null; width: number; height: number; fps: number|null; codec: string; audioTracks: number; hdr: boolean; bitDepth: number; audioCodecs: string[]; rotation: number; sar: number; colorTransfer?: string|null; hdrMetadata?: unknown[]; audioMetadata?: { title: string|null; language: string|null }[] };
type ProbeResult = { path: string; media: MediaInfo|null; error: string|null };
type Job = { id: string; media: MediaInfo; status: 'pending'|'encoding'|'validating'|'completed'|'failed'|'cancelled'; progress: number|null; speed: string|null; etaSeconds: number|null; outputPath: string|null; outputSize: number|null; error: string|null; encoder: string|null };
type QueueSnapshot = { revision: number; jobs: Job[]; running: boolean };
type Environment = { ffmpegAvailable: boolean; ffprobeAvailable: boolean; nvidiaAvailable: boolean; nvidiaReason: string|null; ffmpegPath: string|null; ffprobePath: string|null };
```

- `get_environment()` → Environment. NVIDIA availability is measured with a real short encoding; software decoding is used.
- `probe_files({ paths: string[] })` → ProbeResult[]; one error/result per path.
- `start_batch({ items: MediaInfo[], settings: Settings })` → QueueSnapshot. Rejects if already running. Replaces prior snapshot, freezes settings. Backend reprobes each input before encoding.
- `get_queue()` → QueueSnapshot.
- `cancel_job({ id: string })` → QueueSnapshot. Cancels a waiting job or kills the current encoder and removes its temporary output.
- `stop_batch()` → QueueSnapshot. Cancels all unfinished jobs including the current one.
- `open_output({ path: string })` → void. Reveals an existing output file in its containing folder.

Import and output-directory selection use `@tauri-apps/plugin-dialog` (`open`). Native file drops use Tauri webview drag-drop events. Output paths are backend-owned; source files are never replaced. Unknown duration uses null progress. Errors include FFmpeg stderr for details.

HDR preservation uses CPU HEVC (auto selects CPU for this case); explicit NVIDIA + HDR preservation rejects with an actionable error. PQ/HLG transfer plus supported HDR10 mastering/content light metadata are carried forward.

`revision` increases on every snapshot mutation for the entire app session. Consumers should ignore snapshots with revision lower than the highest applied revision. Job progress is a fraction 0–1, null for unknown duration; completed is 1.

Supported HDR preservation is static HDR10/PQ and HLG. Detected Dolby Vision or dynamic HDR metadata is rejected in preserve mode. Output publication is atomic on filesystems with hard links; exFAT/FAT use an exclusive-create, cancellation-aware copy without overwriting existing files.
