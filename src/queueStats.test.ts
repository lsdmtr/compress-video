import { expect, it } from "vitest";
import { queueStats } from "./queueStats";
import type { Job } from "./types";
const row = (status: Job["status"], progress: number | null) =>
  ({ status, progress }) as Job;
it("counts finished items including failures while preserving unknown active progress", () => {
  expect(
    queueStats([
      row("completed", 1),
      row("encoding", 0.5),
      row("pending", null),
    ]),
  ).toEqual({ finished: 1, total: 3, progress: 0.5 });
  expect(queueStats([row("failed", null), row("encoding", null)])).toEqual({
    finished: 1,
    total: 2,
    progress: null,
  });
  expect(queueStats([])).toEqual({ finished: 0, total: 0, progress: 0 });
});
