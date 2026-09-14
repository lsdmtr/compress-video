# FrameFold visual implementation contract

Reference: concept-light.png (built-in imagegen, ui-mockup). User explicitly changed the theme to light but not pure white during implementation. This supersedes concept.png.

Cool gray #eceef2 background, #f5f6f8 surfaces, #d8dce5 borders, #282d3d foreground, #747b8d secondary, #7564cc violet, #199971 success. Fonts: Inter/system/Segoe UI/Microsoft YaHei; 34px heading, 20px panel titles, 13–14px controls, 12px caption. Radii: 10px panels, 7px controls. Thin borders, no decorative images, gradients or shadows. No pure white surfaces.

Header 62px, content 32px padding, two-column layout (flexible queue + 360px settings). Queue has upload area, toolbar tabs, empty state or media rows. Settings reuse segmented buttons, radio rows, select, folder field, primary action. Icons: Lucide outlined, 1.7–2px stroke. Simple overlapping squares mark is native vector, consistent with concept.

Copy follows concept: FrameFold / 视频压缩 / 本地处理 / 帮助 / 视频更轻，精彩不减。 / 为你的录屏保留细节，释放更多空间。 / 拖入视频，开始压缩 / 支持 NVIDIA App / ShadowPlay 与 OBS 录屏 / 添加视频 / 处理队列 / 全部 / 待处理 / 已完成 / 还没有视频 / 添加多个文件，一次处理。 / 压缩设置 / 为整批视频统一设置。 / 输出分辨率 / 帧率 / 画质 / 视频编码 / 编码设备 / 输出目录 / 选择文件夹 / 输出格式 / 开始压缩.

Intentional functional extensions: browser-only notice; missing engine notice; device details; HDR toggle for HEVC; fps validation; progress/error/retry states; persistent-settings validation; help modal. No fake directory or job placeholders. The window uses system titlebar rather than mimicking window controls from the mockup. AV1 is identified as a codec in help, not a file extension. Narrow views stack settings below queue with no horizontal overflow. Reduced motion respected.
