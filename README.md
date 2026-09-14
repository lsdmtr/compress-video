# FrameFold

Windows 视频批量压缩工具，重点兼容 **NVIDIA App / ShadowPlay、OBS 录制的视频**。这指输入文件兼容性，处理这些视频不要求拥有 NVIDIA 显卡。

采用 **Tauri 2 + Rust + React / TypeScript + FFmpeg**。界面为非纯白浅灰配色，全部视频在本机处理。

![界面](docs/design/implemented-desktop.png)

## 已实现

- 原生多选导入、拖入文件、逐项媒体探测；批量顺序处理、取消当前任务/停止批次、失败重试。
- MP4 输出，H.264 兼容优先 / H.265 体积优先；高画质、均衡、更小体积。
- 原始、720p、1080p、2K（2560×1440）；按比例缩小，兼容竖屏/旋转信息，不放大原视频。
- 原始时间戳、30/60 fps、自定义 1–240 fps（支持 29.97 等小数）。升帧不包含运动插帧。
- 全部音轨保留，AAC 复制，其他音频转 AAC，保留标题和语言。
- HDR→SDR 色调映射；选择 HEVC 可保留 HDR10/HLG 信息。保留 HDR 使用 CPU x265，显式 NVIDIA+HDR 保留会提示调整设备。
- 自动检测可用编码器，NVENC 实际短编码检测；无 NVIDIA 支持可用 CPU。
- 实际进度、速度、预计剩余时间、批量进度、输出大小及节省比例。已经高效压缩的源视频可能得到更大输出。
- 原文件保留、自动避开同名结果、临时文件及输出校验；中文/空格路径支持。

## 开发运行

需要 Node.js **24 LTS**、Rust stable，以及系统对应的 Tauri 开发依赖。Windows 需要 Visual Studio Build Tools 的“使用 C++ 的桌面开发”和 WebView2。

```sh
npm ci
npm run desktop
```

开发环境需在 PATH 中提供 `ffmpeg` 和 `ffprobe`，或指定：

```powershell
$env:VIDEO_COMPRESS_FFMPEG = 'C:\tools\ffmpeg.exe'
$env:VIDEO_COMPRESS_FFPROBE = 'C:\tools\ffprobe.exe'
npm run desktop
```

macOS 本机开发与 HDR 验证可使用完整 FFmpeg：

```sh
brew install ffmpeg-full
export VIDEO_COMPRESS_FFMPEG=/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg
export VIDEO_COMPRESS_FFPROBE=/opt/homebrew/opt/ffmpeg-full/bin/ffprobe
npm run desktop
```

`npm run dev` 仅提供浏览器界面预览，**不能压缩视频，也不会模拟处理结果**。桌面启动命令会自动启动前端，请先停止单独运行的同端口开发服务。

## Windows 便携 ZIP

在 **Windows x64** 开发环境运行 `npm ci` 和 `npm run build:windows`。
输出为 `release/FrameFold-0.1.0-windows-x64-portable.zip` 及 SHA-256 校验文件。
用户完整解压到本机可写文件夹后，双击 `FrameFold.exe` 即可；这是应用程序，不是安装器。

包内包含 FFmpeg 和微软固定版 WebView2，无需安装运行库或管理员权限，视频处理可离线进行。随包运行库会增加包体积；设置和缓存仍保存在当前用户应用数据目录。Windows 10 的沙箱兼容权限仅授予包内 WebView2Runtime 目录读取/执行权限，参见 [微软部署文档](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution#the-fixed-version-runtime-distribution-mode)。不支持从网络共享路径运行。

构建流程固定 FFmpeg/WebView2 版本并校验 SHA-256，验证 WebView2 的微软数字签名；Windows 使用静态 C 运行库。仅打包明确列出的主程序、依赖、说明和许可证，不使用自解压、加壳、混淆或脚本启动器，不注册服务、开机启动或 Defender 排除项。

打包前更新 Defender 病毒库并扫描完整目录，检测失败或无法扫描时终止；ZIP 带逐文件校验清单。若构建机器已配置代码签名证书，可设置 `FRAMEFOLD_SIGN_THUMBPRINT`（证书存储区指纹，且 signtool 在 PATH）对主程序签名。没有证书时生成未签名测试包，不伪造签名。新程序即使签名仍可能出现 SmartScreen 信誉提示，不能保证零误报。

`.github/workflows/windows.yml` 运行测试并构建 ZIP artifact，不自动发布。当前只有 macOS 验证，**尚未生成或实测 Windows ZIP，也未执行 Windows Defender 扫描**。正式交付还需在干净 Windows 10/11 标准用户、无系统 WebView2、断网及中文路径场景验收，并配置有效的发布者签名。固定运行库需要随应用版本更新安全补丁。

## 验证

```sh
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
# 需要真实 FFmpeg/ffprobe：
cargo test --manifest-path src-tauri/Cargo.toml --test real_media -- --ignored --test-threads=1
```

真实媒体测试覆盖中文路径、多音轨、音轨语言/名称、小数帧率、损坏输入隔离、重名保护、取消清理、HDR 转换/保留及旋转视频。源码核心也可用 `--no-default-features` 独立测试。

## 当前边界

- 尚无实际 NVIDIA/OBS 原始样本提供：兼容性测试使用合成的对应编码、音轨、色彩及旋转组合；品牌录制软件的真实样本还需验收。
- 画质预设是可用起点，未针对大量真实游戏录屏做感知指标标定，不保证无损或固定压缩比。
- 首版顺序编码；不提供暂停编码后续传。关闭应用会取消运行中的批次。
- 设置会保留，任务历史不跨进程持久化。浏览器刷新可重新读取当前桌面进程中的队列。
- NTFS/APFS 使用硬链接原子发布；exFAT 等使用不覆盖已有文件的独占创建复制，复制失败会清理本次新建文件。
- Windows 主程序尚未配置发布者签名。公开分发前应配置代码签名，并补齐所携带 GPL FFmpeg 及依赖的对应源码材料；脚本内的来源链接不等于完整的再分发合规包。

架构与 IPC 见 `docs/ipc.md`，验证记录见 `docs/verification.md`。
