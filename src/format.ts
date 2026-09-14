import type { Job, MediaInfo, Settings } from "./types";

export const defaults: Settings = {
  resolution: "1080",
  fps: null,
  quality: "high",
  device: "auto",
  codec: "h264",
  preserveHdr: false,
  outputDir: "",
};
export const bytes = (n: number) => {
  if (!Number.isFinite(n) || n < 0) return "—";
  if (n < 1024) return `${n} B`;
  const i = Math.min(Math.floor(Math.log(n) / Math.log(1024)), 4);
  return `${(n / 1024 ** i).toFixed(1)} ${["B", "KB", "MB", "GB", "TB"][i]}`;
};
export function duration(n: number | null): string {
  if (n === null || !Number.isFinite(n)) return "未知时长";
  const t = Math.max(0, Math.round(n));
  const h = Math.floor(t / 3600);
  return `${h ? `${h}:` : ""}${String(Math.floor(t / 60) % 60).padStart(2, "0")}:${String(t % 60).padStart(2, "0")}`;
}
export function savings(input: number, output: number): string {
  if (input <= 0) return "—";
  const p = (1 - output / input) * 100;
  return p >= 0 ? `节省 ${Math.round(p)}%` : `增大 ${Math.round(-p)}%`;
}
export function readSettings(raw: string | null): Settings {
  let p: Record<string, unknown> = {};
  try {
    const parsed: unknown = JSON.parse(raw || "{}");
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed))
      p = parsed as Record<string, unknown>;
  } catch {
    /* recover defaults */
  }
  return {
    resolution: ["original", "720", "1080", "1440"].includes(
      String(p.resolution),
    )
      ? (p.resolution as Settings["resolution"])
      : defaults.resolution,
    fps:
      typeof p.fps === "number" &&
      Number.isFinite(p.fps) &&
      p.fps >= 1 &&
      p.fps <= 240
        ? p.fps
        : null,
    quality: ["high", "balanced", "small"].includes(String(p.quality))
      ? (p.quality as Settings["quality"])
      : defaults.quality,
    device: ["auto", "cpu", "nvidia"].includes(String(p.device))
      ? (p.device as Settings["device"])
      : defaults.device,
    codec: p.codec === "hevc" ? "hevc" : "h264",
    preserveHdr: p.codec === "hevc" && p.preserveHdr === true,
    outputDir: typeof p.outputDir === "string" ? p.outputDir : "",
  };
}
export function settingsError(s: Settings): string | null {
  if (s.fps !== null && (!Number.isFinite(s.fps) || s.fps < 1 || s.fps > 240))
    return "自定义帧率请输入 1–240 之间的数值";
  if (!s.outputDir.trim()) return "请先选择输出目录";
  return null;
}
export function pendingJob(media: MediaInfo): Job {
  return {
    id: crypto.randomUUID(),
    media,
    status: "pending",
    progress: null,
    speed: null,
    etaSeconds: null,
    outputPath: null,
    outputSize: null,
    error: null,
    encoder: null,
  };
}
