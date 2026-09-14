import { useCallback, useEffect, useRef, useState } from "react";
import { bridge, desktop } from "./bridge";
import { pendingJob, readSettings, settingsError } from "./format";
import type { Environment, Job, QueueSnapshot, Settings } from "./types";

const savedSettings = () => {
  try {
    return readSettings(localStorage.getItem("framefold.settings.v1"));
  } catch {
    return readSettings(null);
  }
};
export function useCompressor() {
  const [settings, setSettings] = useState<Settings>(savedSettings);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [environment, setEnvironment] = useState<Environment | null>(null);
  const [running, setRunning] = useState(false);
  const [probing, setProbing] = useState(false);
  const [starting, setStarting] = useState(false);
  const [hovering, setHovering] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const runningRef = useRef(false),
    probingRef = useRef(false),
    startingRef = useRef(false);
  const mounted = useRef(true);
  const revision = useRef(-1);
  const applySnapshot = useCallback((snapshot: QueueSnapshot) => {
    if (!mounted.current || snapshot.revision <= revision.current) return;
    revision.current = snapshot.revision;
    runningRef.current = snapshot.running;
    setRunning(snapshot.running);
    setJobs((old) => {
      const paths = new Set(snapshot.jobs.map((j) => j.media.path));
      return [...old.filter((j) => !paths.has(j.media.path)), ...snapshot.jobs];
    });
  }, []);
  const importPaths = useCallback(async (paths: string[]) => {
    if (
      !paths.length ||
      runningRef.current ||
      startingRef.current ||
      probingRef.current
    )
      return;
    probingRef.current = true;
    setProbing(true);
    setError(null);
    try {
      const results = await bridge.probe([...new Set(paths)]);
      setJobs((old) => {
        const known = new Set(old.map((j) => j.media.path));
        return [
          ...old,
          ...results.flatMap((r) =>
            r.media && !known.has(r.media.path) ? [pendingJob(r.media)] : [],
          ),
        ];
      });
      const errors = results
        .filter((r) => r.error)
        .map((r) => `${r.path.split(/[\\/]/).pop()}: ${r.error}`);
      if (errors.length) setError(errors.join("\n"));
    } catch (e) {
      setError(String(e));
    } finally {
      probingRef.current = false;
      setProbing(false);
    }
  }, []);
  useEffect(() => {
    mounted.current = true;
    if (!desktop) return;
    let disposed = false;
    const disposers: (() => void)[] = [];
    const keep = (f: () => void) => {
      if (disposed) f();
      else disposers.push(f);
    };
    // Register listener before fetching a snapshot so long encodes remain visible after a reload.
    void bridge
      .onQueue(applySnapshot)
      .then(async (unlisten) => {
        keep(unlisten);
        if (!disposed) applySnapshot(await bridge.queue());
      })
      .catch((e) => setError(String(e)));
    void bridge
      .onDrop((paths) => void importPaths(paths), setHovering)
      .then(keep)
      .catch((e) => setError(String(e)));
    void bridge
      .environment()
      .then((e) => {
        if (!disposed) setEnvironment(e);
      })
      .catch((e) => setError(String(e)));
    return () => {
      disposed = true;
      mounted.current = false;
      disposers.forEach((f) => f());
    };
  }, [applySnapshot, importPaths]);
  useEffect(() => {
    try {
      localStorage.setItem("framefold.settings.v1", JSON.stringify(settings));
    } catch {
      /* persistence is optional */
    }
  }, [settings]);
  const importVideos = async () => {
    try {
      await importPaths(await bridge.pickVideos());
    } catch (e) {
      setError(String(e));
    }
  };
  const chooseOutput = async () => {
    try {
      const p = await bridge.pickOutput();
      if (typeof p === "string") setSettings((s) => ({ ...s, outputDir: p }));
    } catch (e) {
      setError(String(e));
    }
  };
  const start = async () => {
    if (startingRef.current || runningRef.current || probingRef.current) return;
    const problem = settingsError(settings);
    if (problem) {
      setError(problem);
      return;
    }
    const items = jobs
      .filter((j) => j.status === "pending")
      .map((j) => j.media);
    if (!items.length) return;
    startingRef.current = true;
    setStarting(true);
    setError(null);
    try {
      await bridge.start(items, { ...settings });
      applySnapshot(await bridge.queue());
    } catch (e) {
      setError(String(e));
    } finally {
      startingRef.current = false;
      setStarting(false);
    }
  };
  const stop = async () => {
    try {
      await bridge.stop();
      applySnapshot(await bridge.queue());
    } catch (e) {
      setError(String(e));
    }
  };
  const remove = async (job: Job) => {
    if (runningRef.current) {
      try {
        await bridge.cancel(job.id);
        applySnapshot(await bridge.queue());
      } catch (e) {
        setError(String(e));
      }
    } else setJobs((old) => old.filter((j) => j.id !== job.id));
  };
  const retry = (job: Job) => {
    if (runningRef.current || startingRef.current) return;
    setJobs((old) =>
      old.map((j) => (j.id === job.id ? pendingJob(j.media) : j)),
    );
  };
  const reveal = async (path: string) => {
    try {
      await bridge.reveal(path);
    } catch (e) {
      setError(String(e));
    }
  };
  return {
    settings,
    setSettings,
    jobs,
    environment,
    running,
    probing,
    starting,
    hovering,
    error,
    setError,
    importVideos,
    chooseOutput,
    start,
    stop,
    remove,
    retry,
    reveal,
    clearCompleted: () =>
      setJobs((old) => old.filter((j) => j.status !== "completed")),
  };
}
