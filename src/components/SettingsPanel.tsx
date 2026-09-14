import { Folder, Play, Square, ChevronDown, Info } from "lucide-react";
import { useState } from "react";
import { Segmented } from "./Segmented";
import { settingsError } from "../format";
import type { Environment, Settings } from "../types";

export function SettingsPanel({
  settings: s,
  onChange,
  chooseOutput,
  start,
  stop,
  busy,
  running,
  starting,
  pending,
  environment,
  desktop,
}: {
  settings: Settings;
  onChange: (settings: Settings) => void;
  chooseOutput: () => void;
  start: () => void;
  stop: () => void;
  busy: boolean;
  running: boolean;
  starting: boolean;
  pending: number;
  environment: Environment | null;
  desktop: boolean;
}) {
  const [custom, setCustom] = useState(
    s.fps !== null && s.fps !== 30 && s.fps !== 60,
  );
  const [customText, setCustomText] = useState(String(s.fps ?? "24"));
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    onChange({ ...s, [key]: value });
  const fpsMode = custom
    ? "custom"
    : s.fps === null
      ? "original"
      : String(s.fps);
  const invalidFps =
    s.fps !== null && (!Number.isFinite(s.fps) || s.fps < 1 || s.fps > 240);
  const ready =
    desktop && environment?.ffmpegAvailable && environment?.ffprobeAvailable;
  return (
    <aside className="settings-panel panel" aria-label="压缩设置">
      <div className="panel-heading">
        <h2>压缩设置</h2>
        <p>为整批视频统一设置。</p>
      </div>
      <fieldset disabled={busy}>
        <section className="setting">
          <label>输出分辨率</label>
          <Segmented
            label="输出分辨率"
            value={s.resolution}
            options={[
              { value: "original", label: "原始" },
              { value: "720", label: "720p" },
              { value: "1080", label: "1080p" },
              { value: "1440", label: "2K" },
            ]}
            onChange={(v) => update("resolution", v)}
          />
          <p className="hint">
            保持画面比例，不放大原视频
            {s.resolution === "1440" ? " · 2560 × 1440" : ""}
          </p>
        </section>
        <section className="setting">
          <label>帧率</label>
          <Segmented
            label="帧率"
            value={fpsMode}
            options={[
              { value: "original", label: "原始" },
              { value: "30", label: "30 fps" },
              { value: "60", label: "60 fps" },
              { value: "custom", label: "自定义" },
            ]}
            onChange={(v) => {
              setCustom(v === "custom");
              update(
                "fps",
                v === "original"
                  ? null
                  : v === "custom"
                    ? Number(customText)
                    : Number(v),
              );
            }}
          />
          {custom ? (
            <div className="custom-fps">
              <input
                aria-label="自定义帧率"
                inputMode="decimal"
                type="number"
                min="1"
                max="240"
                step="0.01"
                value={customText}
                aria-invalid={invalidFps}
                onChange={(e) => {
                  setCustomText(e.target.value);
                  update(
                    "fps",
                    e.target.value === "" ? NaN : Number(e.target.value),
                  );
                }}
              />
              <span>fps</span>
            </div>
          ) : null}
          {invalidFps ? (
            <p className="hint error-text">请输入 1–240 之间的帧率</p>
          ) : null}
        </section>
        <section className="setting">
          <label>画质</label>
          <Segmented
            label="画质"
            value={s.quality}
            options={[
              { value: "high", label: "高画质" },
              { value: "balanced", label: "均衡" },
              { value: "small", label: "更小体积" },
            ]}
            onChange={(v) => update("quality", v)}
          />
          <p className="hint">
            {s.quality === "high"
              ? "优先保留画面细节，适合游戏和文字录屏。"
              : s.quality === "balanced"
                ? "在画面细节与文件大小之间取得平衡。"
                : "进一步减小体积，画面细节可能减少。"}
          </p>
        </section>
        <section className="setting">
          <label>视频编码</label>
          <div className="codec-options">
            <label
              className={`codec-option ${s.codec === "h264" ? "active" : ""}`}
            >
              <input
                type="radio"
                name="codec"
                value="h264"
                checked={s.codec === "h264"}
                onChange={() =>
                  onChange({ ...s, codec: "h264", preserveHdr: false })
                }
              />
              <span>
                <strong>
                  H.264 <i>/</i> 兼容优先
                </strong>
                <small>适合大多数播放器和分享场景</small>
              </span>
            </label>
            <label
              className={`codec-option ${s.codec === "hevc" ? "active" : ""}`}
            >
              <input
                type="radio"
                name="codec"
                value="hevc"
                checked={s.codec === "hevc"}
                onChange={() => update("codec", "hevc")}
              />
              <span>
                <strong>
                  H.265 <i>/</i> 体积优先
                </strong>
                <small>更高压缩效率，需要 HEVC 播放支持</small>
              </span>
            </label>
          </div>
          {s.codec === "hevc" ? (
            <label className="checkbox">
              <input
                type="checkbox"
                checked={s.preserveHdr}
                onChange={(e) => update("preserveHdr", e.target.checked)}
              />
              保留 HDR（仅对 HDR 源视频生效）
            </label>
          ) : null}
        </section>
        <section className="setting">
          <label htmlFor="device">编码设备</label>
          <div className="select-wrap">
            <select
              id="device"
              value={s.device}
              onChange={(e) =>
                update("device", e.target.value as Settings["device"])
              }
            >
              <option value="auto">自动选择</option>
              <option value="cpu">CPU · 软件编码</option>
              <option value="nvidia" disabled={!environment?.nvidiaAvailable}>
                NVIDIA · 硬件编码
                {environment && !environment.nvidiaAvailable
                  ? "（不可用）"
                  : ""}
              </option>
            </select>
            <ChevronDown size={15} />
          </div>
          <p className="hint">
            {s.device === "cpu"
              ? "速度较慢，适合对压缩效率要求较高的任务。"
              : environment?.nvidiaAvailable
                ? "NVIDIA 编码可用，也可手动选择 CPU。"
                : desktop && !environment
                  ? "正在检测编码环境…"
                  : "无需 N 卡，也能压缩 NVIDIA 录屏。"}
          </p>
        </section>
        <section className="setting">
          <label htmlFor="output">输出目录</label>
          <div className="folder-field">
            <input
              id="output"
              readOnly
              placeholder="请选择保存位置"
              value={s.outputDir}
              title={s.outputDir}
            />
            <button
              type="button"
              className="icon-button folder-button"
              aria-label="选择文件夹"
              title="选择文件夹"
              onClick={chooseOutput}
            >
              <Folder size={18} />
            </button>
          </div>
        </section>
      </fieldset>
      <div className="output-meta">
        <span>
          输出格式 <strong>MP4</strong>
        </span>
        <span>保留全部音轨</span>
      </div>
      {running ? (
        <button className="primary stop" onClick={stop}>
          <Square size={16} />
          停止处理
        </button>
      ) : (
        <button
          className="primary start"
          disabled={busy || !pending || !ready || !!settingsError(s)}
          onClick={start}
        >
          <Play size={17} fill="currentColor" />
          {starting ? "正在启动…" : "开始压缩"}
          {pending > 0 ? <span className="button-count">{pending}</span> : null}
        </button>
      )}
      <p className="start-note">
        <Info size={12} />
        {running
          ? "本批设置已锁定，停止后可重新调整。"
          : "实际大小取决于原视频内容与编码。"}
      </p>
    </aside>
  );
}
