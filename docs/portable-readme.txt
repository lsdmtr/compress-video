酱菇婆专用视频压缩 · Windows x64 轻量便携版

先完整解压 ZIP，再双击 FrameFold.exe 启动；EXE 不是解压程序或安装器。
binaries 文件夹及其中所有 DLL 必须保留，不能只复制主程序。
软件只压缩已有视频，不提供录屏功能；支持 NVIDIA、OBS 已录制的视频。

轻量版使用电脑已有的 Microsoft Edge WebView2 Runtime。
若缺少运行库，启动会显示提示，请从微软官网下载并安装后重试：
https://developer.microsoft.com/microsoft-edge/webview2/
软件不会静默安装运行库，视频处理在本地完成。

设置和界面缓存保存在当前用户的应用数据目录。
不注册服务、不添加开机启动、不修改 Defender 设置或添加排除项。
普通 Windows 程序本身仍是 .exe，这不是安装器。
未签名或新发布的软件可能出现 SmartScreen 提醒，不能保证零误报。
不要关闭安全软件；出现告警时请保留告警名称和文件哈希交给开发者核查。
SHA256SUMS.txt 用于检测文件变化，不代替代码签名或杀毒检测。
本地 Mac 交叉编译包的验证范围见 BUILD-INFO.txt。
