import { useEffect, useRef } from "react";
import { X, Film, ShieldCheck, SlidersHorizontal } from "lucide-react";
export function HelpDialog({ onClose }: { onClose: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    ref.current?.showModal();
  }, []);
  return (
    <dialog
      ref={ref}
      className="help-dialog"
      onCancel={onClose}
      onClick={(e) => {
        if (e.target === ref.current) onClose();
      }}
    >
      <div className="dialog-heading">
        <h2>让每一段录屏，轻一点。</h2>
        <button className="icon-button" aria-label="关闭帮助" onClick={onClose}>
          <X size={20} />
        </button>
      </div>
      <section>
        <Film size={21} />
        <div>
          <h3>为 NVIDIA 与 OBS 录屏准备</h3>
          <p>
            支持 MP4、MKV、MOV 等视频容器，以及 H.264、HEVC、AV1
            编码。自动读取音轨、帧率和 HDR 信息；处理 NVIDIA 录屏无需配备 NVIDIA
            显卡。
          </p>
        </div>
      </section>
      <section>
        <SlidersHorizontal size={21} />
        <div>
          <h3>从「高画质 + 1080p」开始</h3>
          <p>
            H.264 适合分享和广泛播放；H.265 可争取更小体积，播放设备需支持
            HEVC。2K 指
            2560×1440；保持比例、不放大小视频。升高帧率只会重复帧，不会生成新的运动细节。
          </p>
          <p>
            默认保留所有音轨，HDR 转为 SDR；选择 H.265 后可以保留
            HDR。音轨是否同时播放取决于播放器，音轨不会自动混合。
          </p>
        </div>
      </section>
      <section>
        <ShieldCheck size={21} />
        <div>
          <h3>源文件始终保留</h3>
          <p>
            视频只在本机处理，不上传。输出自动避开同名文件；取消任务会清理未完成的输出。已高度压缩的视频再次编码可能变大，请以实际结果为准。
          </p>
        </div>
      </section>
      <button className="primary" onClick={onClose}>
        知道了
      </button>
    </dialog>
  );
}
