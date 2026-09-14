export type Settings = {
  resolution: "original" | "720" | "1080" | "1440";
  fps: number | null;
  quality: "high" | "balanced" | "small";
  device: "auto" | "cpu" | "nvidia";
  codec: "h264" | "hevc";
  preserveHdr: boolean;
  outputDir: string;
};
export type MediaInfo = {
  path: string;
  name: string;
  size: number;
  duration: number | null;
  width: number;
  height: number;
  fps: number | null;
  codec: string;
  audioTracks: number;
  hdr: boolean;
  bitDepth: number;
  audioCodecs: string[];
  rotation: number;
  sar: number;
  colorTransfer?: string | null;
  hdrMetadata?: unknown[];
  audioMetadata?: { title: string | null; language: string | null }[];
};
export type ProbeResult = {
  path: string;
  media: MediaInfo | null;
  error: string | null;
};
export type Job = {
  id: string;
  media: MediaInfo;
  status:
    | "pending"
    | "encoding"
    | "validating"
    | "completed"
    | "failed"
    | "cancelled";
  progress: number | null;
  speed: string | null;
  etaSeconds: number | null;
  outputPath: string | null;
  outputSize: number | null;
  error: string | null;
  encoder: string | null;
};
export type QueueSnapshot = { jobs: Job[]; running: boolean; revision: number };
export type Environment = {
  ffmpegAvailable: boolean;
  ffprobeAvailable: boolean;
  nvidiaAvailable: boolean;
  nvidiaReason: string | null;
  ffmpegPath: string | null;
  ffprobePath: string | null;
};
