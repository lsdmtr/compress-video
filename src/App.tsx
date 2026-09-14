import { useState } from "react";
import {
  Upload,
  Plus,
  HelpCircle,
  ShieldCheck,
  X,
  Info,
  LoaderCircle,
  Copy,
} from "lucide-react";
import { desktop } from "./bridge";
import { useCompressor } from "./useCompressor";
import { SettingsPanel } from "./components/SettingsPanel";
import { QueuePanel } from "./components/QueuePanel";
import { HelpDialog } from "./components/HelpDialog";

export default function App() {
  const c = useCompressor();
  const [help, setHelp] = useState(false);
  const busy = c.running || c.starting || c.probing;
  const pending = c.jobs.filter((j) => j.status === "pending").length;
  const missing =
    desktop &&
    c.environment &&
    (!c.environment.ffmpegAvailable || !c.environment.ffprobeAvailable);
  return (
    <div className="app-shell">
      <header className="topbar">
        <a className="brand" href="#" aria-label="酱菇婆专用视频压缩 首页">
          <Copy size={29} strokeWidth={1.7} />
          <strong>酱菇婆专用视频压缩</strong>
        </a>
        <div className="header-actions">
          <span className="local-status">
            <i />
            本地处理
          </span>
          <button
            className="secondary help-button"
            onClick={() => setHelp(true)}
          >
            <HelpCircle size={16} />
            帮助
          </button>
        </div>
      </header>
      <main className="workspace">
        <div className="main-column">
          <div className="intro">
            <h1>视频更轻，精彩不减。</h1>
            <p>为你的录屏保留细节，释放更多空间。</p>
          </div>
          {!desktop ? (
            <div className="notice">
              <Info size={16} />
              <span>当前为界面预览，导入和压缩请在桌面应用中使用。</span>
            </div>
          ) : null}
          {missing ? (
            <div className="notice warning">
              <Info size={16} />
              <span>
                未找到完整的视频引擎。请使用包含 FFmpeg 的安装包，开发环境请配置
                FFmpeg 和 ffprobe。
              </span>
            </div>
          ) : null}
          {c.error ? (
            <div className="notice error-notice" role="alert">
              <Info size={16} />
              <pre>{c.error}</pre>
              <button
                className="icon-button"
                aria-label="关闭提示"
                onClick={() => c.setError(null)}
              >
                <X size={15} />
              </button>
            </div>
          ) : null}
          <section
            className={`dropzone ${c.hovering ? "drag-over" : ""} ${busy ? "busy" : ""}`}
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              e.preventDefault();
              if (!desktop)
                c.setError(
                  "浏览器无法获取视频的完整本地路径。请在桌面应用中拖入视频。",
                );
            }}
            aria-label="视频导入区域"
          >
            <Upload className="upload-icon" size={43} strokeWidth={1.5} />
            <h2>
              {c.probing
                ? "正在读取视频信息…"
                : c.hovering
                  ? "松开鼠标，添加视频"
                  : "拖入视频，开始压缩"}
            </h2>
            <p>支持 NVIDIA App / ShadowPlay 与 OBS 录屏</p>
            <span className="formats">MP4 · MKV · MOV · WEBM</span>
            <button
              className="primary add-button"
              disabled={busy}
              onClick={() => void c.importVideos()}
            >
              {c.probing ? (
                <LoaderCircle size={17} className="spin" />
              ) : (
                <Plus size={17} />
              )}
              添加视频
            </button>
          </section>
          <QueuePanel
            jobs={c.jobs}
            running={c.running || c.starting}
            onRemove={(j) => void c.remove(j)}
            onRetry={c.retry}
            onReveal={(p) => void c.reveal(p)}
            onClear={c.clearCompleted}
          />
          <div className="privacy-note">
            <ShieldCheck size={14} />
            源文件始终保留<span>·</span>所有处理均在本机完成
          </div>
        </div>
        <SettingsPanel
          settings={c.settings}
          onChange={c.setSettings}
          chooseOutput={() => void c.chooseOutput()}
          start={() => void c.start()}
          stop={() => void c.stop()}
          busy={busy}
          running={c.running}
          starting={c.starting}
          pending={pending}
          environment={c.environment}
          desktop={desktop}
        />
      </main>
      <footer>
        <span>
          酱菇婆专用视频压缩 <b>v0.1.0</b>
        </span>
        <span>
          本地离线 <i>·</i> 隐私优先
        </span>
      </footer>
      {help ? <HelpDialog onClose={() => setHelp(false)} /> : null}
    </div>
  );
}
