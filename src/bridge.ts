import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type {
  Environment,
  MediaInfo,
  ProbeResult,
  QueueSnapshot,
  Settings,
} from "./types";

export const desktop = isTauri();
const requireDesktop = () => {
  if (!desktop)
    throw new Error(
      "当前为浏览器界面预览。请运行桌面应用以导入本地视频并开始压缩。",
    );
};
export const bridge = {
  environment: () => invoke<Environment>("get_environment"),
  queue: () => invoke<QueueSnapshot>("get_queue"),
  async pickVideos() {
    requireDesktop();
    const files = await open({
      multiple: true,
      directory: false,
      title: "添加录屏视频",
      filters: [
        {
          name: "视频文件",
          extensions: ["mp4", "mkv", "mov", "m4v", "webm", "avi", "ts", "flv"],
        },
      ],
    });
    return files ? (Array.isArray(files) ? files : [files]) : [];
  },
  async pickOutput() {
    requireDesktop();
    return await open({
      directory: true,
      multiple: false,
      title: "选择压缩视频的输出目录",
    });
  },
  probe: (paths: string[]) => invoke<ProbeResult[]>("probe_files", { paths }),
  start: (items: MediaInfo[], settings: Settings) =>
    invoke<QueueSnapshot>("start_batch", { items, settings }),
  cancel: (id: string) => invoke<QueueSnapshot>("cancel_job", { id }),
  stop: () => invoke<QueueSnapshot>("stop_batch"),
  reveal: (path: string) => invoke<void>("open_output", { path }),
  onQueue: (cb: (data: QueueSnapshot) => void) =>
    listen<QueueSnapshot>("queue-update", (e) => cb(e.payload)),
  onDrop: (cb: (paths: string[]) => void, hovering: (state: boolean) => void) =>
    getCurrentWebviewWindow().onDragDropEvent((event) => {
      hovering(event.payload.type === "enter" || event.payload.type === "over");
      if (event.payload.type === "drop") cb(event.payload.paths);
    }),
};
