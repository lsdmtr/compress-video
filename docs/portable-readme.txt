FrameFold 视频压缩 · Windows x64 便携版

将整个 ZIP 解压到本机可写文件夹，双击 FrameFold.exe。
无需安装，不需要管理员权限，不要只复制主程序或直接在压缩包内运行。
binaries 与 WebView2Runtime 必须和 FrameFold.exe 一起保留。
FFmpeg 与微软 WebView2 已随包提供，视频处理不需要联网。
设置和 WebView 缓存保存在当前用户的应用数据目录；删除软件文件夹不会删除这些设置。
不注册服务、不添加开机启动、不修改 Defender 设置或添加排除项。
为兼容 Windows 10 的 WebView2 沙箱，启动时仅为包内 WebView2Runtime
目录添加微软要求的应用容器读取/执行权限，不授予写权限。
请使用本地磁盘路径；WebView2 固定运行库不支持网络共享路径。

普通 Windows 程序本身仍然是 .exe，这不是安装器。
未签名或新发布的软件可能出现 SmartScreen 信誉提示；杀毒误报无法保证为零。
不要关闭安全软件。如果出现告警，请保留告警名称和文件哈希交给开发者核查。
SHA256SUMS.txt 用于检测文件变化，不代替代码签名或杀毒检测。
固定 WebView2 运行库随应用新版本更新，不在后台下载更新。
