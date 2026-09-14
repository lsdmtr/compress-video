import { useState } from "react";
import {
  Film,
  X,
  FolderOpen,
  RotateCcw,
  CircleCheck,
  CircleAlert,
  LoaderCircle,
  Clock3,
  Trash2,
} from "lucide-react";
import { bytes, duration, savings } from "../format";
import type { Job } from "../types";
import { queueStats } from "../queueStats";
const labels: Record<Job["status"], string> = {
  pending: "待处理",
  encoding: "压缩中",
  validating: "校验中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};
export function QueuePanel({
  jobs,
  running,
  onRemove,
  onRetry,
  onReveal,
  onClear,
}: {
  jobs: Job[];
  running: boolean;
  onRemove: (job: Job) => void;
  onRetry: (job: Job) => void;
  onReveal: (path: string) => void;
  onClear: () => void;
}) {
  const [filter, setFilter] = useState("all");
  const stats = queueStats(jobs);
  const filtered = jobs.filter(
    (j) =>
      filter === "all" ||
      (filter === "pending" &&
        ["pending", "encoding", "validating"].includes(j.status)) ||
      (filter === "completed" && j.status === "completed"),
  );
  const done = jobs.filter((j) => j.status === "completed");
  const saved = done.reduce(
    (n, j) => n + j.media.size - (j.outputSize ?? j.media.size),
    0,
  );
  return (
    <section className="queue-panel panel" aria-label="处理队列">
      <div className="queue-toolbar">
        <h2>
          处理队列 <span className="count">{jobs.length}</span>
        </h2>
        <div className="queue-tabs" role="group" aria-label="筛选队列">
          {[
            ["all", "全部"],
            ["pending", "待处理"],
            ["completed", "已完成"],
          ].map(([value, label]) => (
            <button
              key={value}
              aria-pressed={filter === value}
              className={filter === value ? "selected" : ""}
              onClick={() => setFilter(value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      {running ? (
        <div className="batch-progress">
          <div>
            <span>批量处理中</span>
            <span>
              已处理 {stats.finished} / {stats.total} 个
            </span>
          </div>
          <div
            className={`progress-track ${stats.progress === null ? "indeterminate" : ""}`}
            role="progressbar"
            aria-label="批量处理总进度"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={
              stats.progress === null
                ? undefined
                : Math.round(stats.progress * 100)
            }
          >
            <span
              style={{
                width:
                  stats.progress === null ? "35%" : `${stats.progress * 100}%`,
              }}
            />
          </div>
        </div>
      ) : null}
      {!filtered.length ? (
        <div className="queue-empty">
          <Film size={34} strokeWidth={1.5} />
          <h3>{jobs.length ? "暂无匹配的任务" : "还没有视频"}</h3>
          <p>
            {jobs.length
              ? "切换筛选查看其他视频。"
              : "添加多个文件，一次处理。"}
          </p>
        </div>
      ) : (
        <div className="job-list">
          {filtered.map((j) => (
            <JobRow
              key={j.id}
              job={j}
              running={running}
              onRemove={onRemove}
              onRetry={onRetry}
              onReveal={onReveal}
            />
          ))}
        </div>
      )}
      {done.length > 0 ? (
        <div className="queue-summary">
          <span>
            <CircleCheck size={15} />
            {done.length} 个完成{" "}
            <b>
              {saved >= 0 ? `节省 ${bytes(saved)}` : `增加 ${bytes(-saved)}`}
            </b>
          </span>
          <button disabled={running} className="text-button" onClick={onClear}>
            <Trash2 size={13} />
            清除已完成
          </button>
        </div>
      ) : null}
    </section>
  );
}
function JobRow({
  job: j,
  running,
  onRemove,
  onRetry,
  onReveal,
}: {
  job: Job;
  running: boolean;
  onRemove: (job: Job) => void;
  onRetry: (job: Job) => void;
  onReveal: (path: string) => void;
}) {
  const active = j.status === "encoding" || j.status === "validating";
  const [details, setDetails] = useState(false);
  return (
    <article className={`job-row status-${j.status}`}>
      <div className="job-main">
        <div className="file-icon">
          <Film size={21} />
          <span>
            {j.media.path.split(".").pop()?.toUpperCase().slice(0, 4)}
          </span>
        </div>
        <div className="job-info">
          <h3 title={j.media.path}>{j.media.name}</h3>
          <p>
            {j.media.width} × {j.media.height}
            <em>·</em>
            {j.media.fps ? `${Number(j.media.fps.toFixed(2))} fps` : "可变帧率"}
            <em>·</em>
            {duration(j.media.duration)}
            <em>·</em>
            {bytes(j.media.size)}
            {j.media.hdr ? <span className="media-tag">HDR</span> : null}
            {j.media.audioTracks > 1 ? (
              <span className="media-tag">{j.media.audioTracks} 音轨</span>
            ) : null}
          </p>
        </div>
        <div className={`job-state ${j.status}`}>
          {active ? (
            <LoaderCircle className="spin" size={14} />
          ) : j.status === "completed" ? (
            <CircleCheck size={14} />
          ) : j.status === "failed" ? (
            <CircleAlert size={14} />
          ) : (
            <Clock3 size={14} />
          )}
          <span>{labels[j.status]}</span>
        </div>
        <div className="job-actions">
          {j.outputPath ? (
            <button
              className="icon-button"
              title="打开所在文件夹"
              aria-label={`打开 ${j.media.name} 所在文件夹`}
              onClick={() => onReveal(j.outputPath!)}
            >
              <FolderOpen size={16} />
            </button>
          ) : null}
          {(j.status === "failed" || j.status === "cancelled") && !running ? (
            <button
              className="icon-button"
              title="重新加入队列"
              aria-label={`重试 ${j.media.name}`}
              onClick={() => onRetry(j)}
            >
              <RotateCcw size={16} />
            </button>
          ) : null}
          {!running || j.status === "pending" || active ? (
            <button
              className="icon-button"
              title={running ? "取消任务" : "移除任务"}
              aria-label={`${running ? "取消" : "移除"} ${j.media.name}`}
              onClick={() => onRemove(j)}
            >
              <X size={16} />
            </button>
          ) : null}
        </div>
      </div>
      {active ? (
        <div className="job-progress">
          <div
            className={`progress-track ${j.progress === null ? "indeterminate" : ""}`}
            role="progressbar"
            aria-label={`${j.media.name} 压缩进度`}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={
              j.progress === null ? undefined : Math.round(j.progress * 100)
            }
          >
            <span
              style={{
                width:
                  j.progress === null
                    ? "35%"
                    : `${Math.min(100, j.progress * 100)}%`,
              }}
            />
          </div>
          <div className="progress-caption">
            <span>
              {j.status === "validating"
                ? "正在校验输出文件"
                : j.progress === null
                  ? "正在处理…"
                  : `${Math.floor(j.progress * 100)}%`}
              {j.encoder ? ` · ${j.encoder}` : ""}
            </span>
            <span>
              {j.speed ? `${j.speed} · ` : ""}
              {j.etaSeconds !== null ? `剩余 ${duration(j.etaSeconds)}` : ""}
            </span>
          </div>
        </div>
      ) : null}
      {j.outputSize !== null ? (
        <div
          className={`result-line ${j.outputSize > j.media.size ? "larger" : ""}`}
        >
          <span>
            {bytes(j.media.size)} → <strong>{bytes(j.outputSize)}</strong>
          </span>
          <span>{savings(j.media.size, j.outputSize)}</span>
        </div>
      ) : null}
      {j.error ? (
        <div className="job-error">
          <button className="text-button" onClick={() => setDetails(!details)}>
            <CircleAlert size={13} />
            {details ? "收起错误详情" : "查看失败原因"}
          </button>
          {details ? <pre>{j.error}</pre> : null}
        </div>
      ) : null}
    </article>
  );
}
