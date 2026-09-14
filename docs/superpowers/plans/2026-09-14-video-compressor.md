# Video Compressor Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development to implement the backend task; controller implements UI and integration. Steps use checkboxes for tracking.

**Goal:** Deliver a working Windows-oriented MP4 batch compressor for NVIDIA App / ShadowPlay and OBS recordings.
**Architecture:** React UI communicates with Rust Tauri commands. Rust probes media, builds validated FFmpeg arguments, owns queue execution and emits snapshots.
**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vite, FFmpeg/ffprobe.
**Spec:** docs/superpowers/specs/2026-09-14-video-compressor-design.md

## Global Constraints

- Windows 10/11 x64 target; macOS development does not prove NVIDIA or Windows compatibility.
- Never overwrite source or preexisting outputs; no fake processing data in browser preview.
- Output MP4; H.264/HEVC; original/720p/1080p/1440p; original/30/60/custom 1–240 fps.
- All audio tracks preserved; HDR requires tone mapping for SDR or explicit HEVC preserve mode.
- Single active encode; cancellation cleans only owned temporary files; no shell command interpolation.
- Ruling: use the user's empty checkout on a new codex branch; separate worktree provides no additional isolation here and would hide delivered files from the user's selected workspace.

## Task 1: Desktop media engine

Files: src-tauri/Cargo.toml, build.rs, tauri.conf.json, capabilities/default.json, src/{main,lib,model,media,encoding,queue}.rs, tests/engine.rs.

Interfaces: commands `get_environment`, `probe_files({paths})`, `start_batch({items,settings})`, `get_queue`, `cancel_job({id})`, `stop_batch`, `open_output({path})`; event `queue-update` carrying QueueSnapshot. Frontend types mirror Rust serde camelCase. Full concrete IPC definitions recorded in docs/ipc.md before UI integration.

- [x] Write Rust tests for resize geometry, invalid fps, audio mapping, NVENC/CPU plan, collision-safe outputs. Run `cargo test --manifest-path src-tauri/Cargo.toml` and observe missing behavior.
- [x] Implement structs and deterministic plan builder, then probe/queue/process lifecycle. Test short real video transcodes through same production functions.
- [x] Verify cancellation, failed-item isolation, HDR strategy and publish only validated output.
- [x] Run cargo test/check, report interface and test evidence for controller review.

## Task 2: UI and desktop bridge

Files: package.json, vite.config.ts, tsconfig.json, index.html, src/{main,App,types,bridge,useCompressor,format}.tsx/ts, src/components/*, src/styles.css.

Interfaces: consume Task 1 IPC; settings and snapshots are validated by Rust. Browser preview exposes a truthful desktop-required message, never simulated encoding.

- [x] Create full-screen visual concept and extract tokens: non-white pale-gray backgrounds, violet accent, two-column queue/settings, meaningful empty state.
- [x] Add tests for frontend setting serialization, progress/savings formatting and queue decisions; observe failing tests.
- [x] Implement usable import, native drop, settings, output picker, queue controls, errors, retry and real progress.
- [x] Run typecheck/build/tests, then use browser to inspect desktop/narrow views and settings interactions.

## Task 3: Integration and Windows delivery

Files: scripts/*, .github/workflows/windows.yml, README.md, docs/verification.md.

- [x] Bundle fixed FFmpeg/ffprobe sidecars with checksums and license materials; expose a reproducible Windows build command.
- [x] Run a real multi-audio fixture through the engine and decode-check final MP4 with dimensions/fps/track count assertions.
- [x] Review backend and UI interfaces, handle findings, run final tests and production build.
- [x] Record actual test evidence, Windows/NVIDIA limitations and launch instructions.

## Completion record

Source implementation, macOS build, browser checks and real-media engine tests completed. Windows installer workflow is authored; Windows/NVIDIA hardware execution and recorder-original fixtures remain unverified, as explicitly documented in README and docs/verification.md. No merge, push, or publication performed.
