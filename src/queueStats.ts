import type { Job } from "./types";
/** Item-weighted work progress; unknown active duration stays indeterminate. */
export function queueStats(jobs: Job[]) {
  let finished = 0,
    sum = 0,
    unknown = false;
  for (const job of jobs) {
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      finished++;
      sum++;
    } else if (job.status === "encoding" || job.status === "validating") {
      if (job.progress === null) unknown = true;
      else sum += Math.max(0, Math.min(1, job.progress));
    }
  }
  return {
    finished,
    total: jobs.length,
    progress: unknown ? null : jobs.length ? sum / jobs.length : 0,
  };
}
