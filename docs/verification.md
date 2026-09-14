# Verification record — 2026-09-14

## Implementation and review

- User-approved Tauri/Rust/React/FFmpeg architecture implemented in the selected checkout on `codex/video-compressor`.
- User's subsequent light-theme requirement supersedes initial dark concept. Final background is `rgb(236, 238, 242)`; surfaces are pale gray, not pure white.
- Backend task review identified uncancellable ffprobe calls. Fixed by propagating per-job cancellation through normal/HDR/output probes and NVIDIA short tests, checking before spawn, and waiting for in-flight probe cleanup on application exit. Slow probe regression went from ~30 seconds to under 0.2 seconds.
- Final integration review found a Node engine-range mismatch; aligned package and README to Node 24 LTS. No other confirmed blocker in IPC/resource resolution review. This is code review, not Windows execution evidence.

## Automated evidence

Final verification: 9 frontend tests passed; 8 desktop Rust tests passed; all 6 opt-in real FFmpeg tests passed in 6.11 seconds (23 total). Production frontend build and strict clippy `--all-targets -- -D warnings` passed. npm audit reported 0 vulnerabilities. Commands are listed in README. Real tests use `/opt/homebrew/opt/ffmpeg-full/bin/{ffmpeg,ffprobe}` on macOS.

Covered behaviors: settings recovery/validation; honest larger-output reporting; stale-event protection using monotonic revisions; partial import success; settings snapshot and active-batch guard; item-weighted aggregate progress; no-upscale geometry/portrait/SAR; original/custom frame-rate plans; audio mapping; HDR rules; collision naming/exclusive copy; probe cancellation; shutdown with idle queue and active probes.

Real tests also cover AV1 input to H.264 MP4 and VFR input preserving frame count and presentation timestamps within 1.1 ms.

Real media evidence includes successful FFmpeg encoding, ffprobe output inspection, and decoding checks rather than mocked transcoding. Source byte preservation and temporary file cleanup are asserted.

## Desktop and browser checks

- Built a real Tauri macOS `.app` with `npm run tauri -- build --debug --bundles app`; launched the bundled executable with real FFmpeg paths.
- Native window rendered the actual application at `tauri://localhost`, and clicking Add Video opened the native multi-file dialog. The computer-use driver could not reliably confirm a path in macOS's Go to Folder sheet, so no complete native file-picker-to-encode interaction is claimed. The real queue engine and frontend bridge are covered separately by automated tests.
- Codex in-app browser checked custom 29.97 fps, invalid 0 fps validation, H.264/H.265 selection and HDR toggle, help modal/Escape, and truthful desktop-required import error in preview mode.
- Vite dependency upgrade left the older development process active; restarted it, confirmed the current page loads. This was an environment restart, not an application workaround.
- Screenshots: `docs/design/implemented-desktop.png` and `implemented-mobile.png`. Desktop viewport 1506×1045; full-page output is taller because of explanatory labels. Narrow viewport 390×844; document width equals viewport width (390), with no horizontal overflow. Browser viewport restored after testing.

## Visual comparison ledger

Reference: `docs/design/concept-light.png`, generated using built-in imagegen from the initial mockup with user-requested light colors. Reference and final screenshots inspected using `view_image`.

| Point | Reference / implemented verification |
| --- | --- |
| Palette | Light gray #eceef2 backdrop, slightly lighter panels, slate text, restrained violet controls; no pure white page surface. |
| Layout | Left import and queue + right continuous settings panel; corrected wide-screen panel width to match reference. |
| Typography | Enlarged control labels and hints after initial comparison; hierarchy retained across heading, panels, buttons, captions. |
| Controls | Four resolution/fps segments, three quality choices, two codec rows, folder picker and full-width primary action. |
| Icons | Corrected initial layered icon to overlapping frames; consistent outline upload, film, help, folder icons. |
| States | Meaningful empty state, disabled start with no real input, selected options, honest browser notice, help modal; progress rows implemented as data-driven components. |
| Responsive | 390px stacks panels and keeps every control reachable without horizontal scrolling. |

Copy differences are intentional and functional: browser preview notice, HDR/device notes, no fabricated output directory, WEBM listed as a container instead of AV1 (codec), and preservation/size explanations. System window controls are native rather than fake HTML. No raster screenshot is used as the interface. Functional labels make the layout slightly taller than the image concept.

## Windows delivery evidence and limits

- Successfully downloaded and extracted fixed Windows FFmpeg 8.0.1 full build; SHA-256 `467cde100a47ed4b03a897988aeb4a296890c1e2b2d2864204657d002bc5fb90` matched publisher release metadata.
- `npm run prepare:windows` creates ffmpeg.exe, ffprobe.exe and license/source-reference files under ignored `src-tauri/binaries/`. Windows config maps them next to the installed executable under `binaries/`, matching runtime lookup.
- Windows workflow runs native engine/filter checks and real tests before NSIS packaging. It has been authored but not remotely executed or published.
- Not verified: Windows installation/launch, Windows cancellation, actual NVIDIA GPU runtime, actual original ShadowPlay/OBS recordings. Synthetic media covers corresponding format properties; it does not prove every recorder/version combination.
- The macOS app is a development smoke-test artifact, not a self-contained macOS distribution. Windows source/build workflow is the requested target.

## 便携交付修订

Windows 构建切换为 --no-bundle + 普通 ZIP，替代此前 NSIS 方案。包内携带 FFmpeg 和 Microsoft WebView2 Fixed Runtime；主程序配置固定运行库路径，Windows 目标静态链接 C 运行库。新增 SHA-256 清单、可选证书签名与 Defender 扫描失败阻断。未使用自解压、提权或安全软件排除项。

WebView2 153.0.4234.32 x64 CAB 从微软官方页面链接下载，在本机测得 SHA-256 为 2cb653a74426f0aa802c2396775c6bc674fd662d5396bd677f47bfa6e12eba9c；构建脚本还要求运行库微软 Authenticode 签名有效。固定版本需在后续发布中维护。

Windows PowerShell 打包、数字签名、Defender 扫描及干净 Windows 离线启动尚未在本机执行，不能将脚本配置等同于这些验收通过。此前安装包相关记录为历史方案。

本次便携修订后：前端 9 项测试、Rust 8 项测试、前端生产构建、Clippy 均通过；打包脚本布局人工复核通过。Windows 专用代码和脚本仍需 Windows 执行验证。

按用户要求移除 GitHub Actions workflow，保留 Windows 本机构建及便携启动检查脚本。此前远端构建未生成可交付 ZIP。

## 本地 Windows Release 包

2026-09-14：使用本机 cargo-xwin 0.23.1 + LLVM/LLD 23.1.1 完成 Windows x64 MSVC Release 交叉编译。修复 Tauri 的静态 VCRuntime/动态 UCRT 设置与 cargo-xwin 完全静态 CRT 的冲突；本地交叉构建设置 STATIC_VCRUNTIME=false，由 cargo-xwin 和目标 crt-static 配置负责链接。

产物：release/FrameFold-0.1.0-windows-x64-portable.zip，473689768 字节。主程序 9.1 MiB，PE machine AMD64，Windows GUI subsystem；导入表仅包含 Windows 系统 DLL，没有额外 VC++ Redistributable 或 WebView2Loader DLL 依赖。包内含 FFmpeg、微软固定 WebView2、许可证、构建说明和 SHA-256 清单。ZIP CRC 与哈希检查通过。

应用源码构建提交：1dbaefb。这是本地 Release 测试包，未签名，未执行 Windows Defender 扫描和 Windows 真机启动验收。远端构建已取消，workflow 已移除。
