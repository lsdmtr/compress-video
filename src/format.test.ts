import { describe, expect, it } from "vitest";
import {
  bytes,
  duration,
  savings,
  readSettings,
  settingsError,
  pendingJob,
} from "./format";
import type { MediaInfo } from "./types";

describe("result reporting", () => {
  it("reports larger outputs honestly", () => {
    expect(savings(1000, 1200)).toBe("增大 20%");
    expect(savings(1000, 400)).toBe("节省 60%");
    expect(savings(0, 400)).toBe("—");
  });
  it("formats unknown duration without pretending it is zero", () => {
    expect(duration(null)).toBe("未知时长");
    expect(duration(3661)).toBe("1:01:01");
    expect(bytes(1024 * 1024 * 1.5)).toBe("1.5 MB");
  });
});
describe("settings trust boundary", () => {
  it("recovers malformed persistence and rejects invalid enums", () => {
    expect(readSettings("{broken").resolution).toBe("1080");
    expect(
      readSettings('{"resolution":"4k","fps":-1,"codec":"av1"}'),
    ).toMatchObject({ resolution: "1080", fps: null, codec: "h264" });
    expect(readSettings('{"resolution":"720","fps":29.97}')).toMatchObject({
      resolution: "720",
      fps: 29.97,
    });
  });
  it("requires output directory and bounded finite frame rate", () => {
    const base = { ...readSettings(null), outputDir: "/outputs" };
    expect(settingsError(base)).toBeNull();
    expect(settingsError({ ...base, fps: 0 })).toContain("1–240");
    expect(settingsError({ ...base, fps: NaN })).toContain("1–240");
    expect(settingsError({ ...base, outputDir: " " })).toContain("输出目录");
  });
  it("creates pending rows from actual media only", () => {
    const media = { path: "/a.mp4", name: "a.mp4" } as MediaInfo;
    expect(pendingJob(media)).toMatchObject({
      media,
      status: "pending",
      outputSize: null,
      progress: null,
    });
  });
});
