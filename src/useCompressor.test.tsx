// @vitest-environment jsdom
import { act, renderHook, waitFor, cleanup } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Job, MediaInfo, QueueSnapshot } from "./types";
const api = vi.hoisted(() => ({
  environment: vi.fn(),
  queue: vi.fn(),
  onQueue: vi.fn(),
  onDrop: vi.fn(),
  probe: vi.fn(),
  pickVideos: vi.fn(),
  pickOutput: vi.fn(),
  start: vi.fn(),
  cancel: vi.fn(),
  stop: vi.fn(),
  reveal: vi.fn(),
}));
vi.mock("./bridge", () => ({ desktop: true, bridge: api }));
import { useCompressor } from "./useCompressor";
let update: (q: QueueSnapshot) => void;
const media: MediaInfo = {
  path: "/recording.mp4",
  name: "recording.mp4",
  size: 1000,
  duration: 10,
  width: 1920,
  height: 1080,
  fps: 60,
  codec: "h264",
  audioTracks: 2,
  hdr: false,
  bitDepth: 8,
  audioCodecs: ["aac", "aac"],
  rotation: 0,
  sar: 1,
};
const job: Job = {
  id: "job-1",
  media,
  status: "encoding",
  progress: 0.2,
  speed: "2x",
  etaSeconds: 4,
  outputPath: null,
  outputSize: null,
  error: null,
  encoder: "libx264",
};
afterEach(cleanup);
beforeEach(() => {
  vi.resetAllMocks();
  localStorage.clear();
  api.onQueue.mockImplementation(async (cb) => {
    update = cb;
    return () => {};
  });
  api.onDrop.mockResolvedValue(() => {});
  api.environment.mockResolvedValue({
    ffmpegAvailable: true,
    ffprobeAvailable: true,
    nvidiaAvailable: false,
  });
  api.queue.mockResolvedValue({ jobs: [], running: false, revision: 0 });
});
describe("queue bridge", () => {
  it("rejects stale progress snapshots so completed jobs never regress", async () => {
    const { result } = renderHook(() => useCompressor());
    await waitFor(() => expect(api.queue).toHaveBeenCalled());
    act(() =>
      update({
        revision: 4,
        running: false,
        jobs: [{ ...job, status: "completed", progress: 1, outputSize: 400 }],
      }),
    );
    act(() => update({ revision: 3, running: true, jobs: [job] }));
    expect(result.current.running).toBe(false);
    expect(result.current.jobs[0].status).toBe("completed");
  });
  it("imports successful probes despite another invalid file and deduplicates repeated inputs", async () => {
    api.pickVideos.mockResolvedValue(["/recording.mp4", "/bad.mp4"]);
    api.probe.mockResolvedValue([
      { path: media.path, media, error: null },
      { path: "/bad.mp4", media: null, error: "invalid media" },
    ]);
    const { result } = renderHook(() => useCompressor());
    await waitFor(() => expect(api.queue).toHaveBeenCalled());
    await act(() => result.current.importVideos());
    await act(() => result.current.importVideos());
    expect(result.current.jobs).toHaveLength(1);
    expect(result.current.error).toContain("bad.mp4");
  });
  it("snapshots settings at start and starts only pending media", async () => {
    api.pickVideos.mockResolvedValue([media.path]);
    api.probe.mockResolvedValue([{ path: media.path, media, error: null }]);
    const { result } = renderHook(() => useCompressor());
    await waitFor(() => expect(api.queue).toHaveBeenCalled());
    await act(() => result.current.importVideos());
    act(() =>
      result.current.setSettings({
        ...result.current.settings,
        outputDir: "/out",
        fps: 29.97,
      }),
    );
    api.queue.mockResolvedValue({ revision: 1, running: true, jobs: [job] });
    await act(() => result.current.start());
    expect(api.start).toHaveBeenCalledExactlyOnceWith(
      [media],
      expect.objectContaining({ fps: 29.97, outputDir: "/out" }),
    );
    await act(() => result.current.start());
    expect(api.start).toHaveBeenCalledTimes(1);
  });
});
