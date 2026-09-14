import { createHash } from "node:crypto";
import { createReadStream, createWriteStream } from "node:fs";
import {
  mkdir,
  readFile,
  readdir,
  copyFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

// Immutable release asset; SHA-256 from the publisher's GitHub release API.
const release = "8.0.1";
const filename = `ffmpeg-${release}-full_build.zip`;
const sha256 =
  "467cde100a47ed4b03a897988aeb4a296890c1e2b2d2864204657d002bc5fb90";
const url = `https://github.com/GyanD/codexffmpeg/releases/download/${release}/${filename}`;
const root = fileURLToPath(new URL("../", import.meta.url));
const cache = path.join(root, ".cache", "windows-ffmpeg");
const archive = path.join(cache, filename);
const destination = path.join(root, "src-tauri", "binaries");
const checksum = async (file) => {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
};
await mkdir(cache, { recursive: true });
let valid = false;
try {
  valid = (await checksum(archive)) === sha256;
} catch {
  /* download missing archive */
}
if (!valid) {
  console.log(`Downloading FFmpeg ${release} Windows x64 full build…`);
  const partial = `${archive}.download`;
  try {
    try {
      const response = await fetch(url, {
        signal: AbortSignal.timeout(600000),
      });
      if (!response.ok || !response.body)
        throw new Error(`Download HTTP ${response.status}`);
      await pipeline(
        Readable.fromWeb(response.body),
        createWriteStream(partial),
      );
    } catch {
      // System curl also supports environments with an OS-configured proxy.
      const download = spawnSync(
        process.platform === "win32" ? "curl.exe" : "curl",
        [
          "--fail",
          "--location",
          "--retry",
          "2",
          "--connect-timeout",
          "30",
          "--max-time",
          "600",
          "--output",
          partial,
          url,
        ],
        { stdio: "inherit" },
      );
      if (download.error || download.status !== 0)
        throw new Error(
          "FFmpeg download failed; check network access to github.com",
        );
    }
    if ((await checksum(partial)) !== sha256)
      throw new Error("FFmpeg SHA-256 mismatch; refusing to bundle");
    await copyFile(partial, archive);
  } finally {
    await rm(partial, { force: true });
  }
}
console.log(`Verified ${filename}: ${sha256}`);
const unpack = path.join(cache, "unpacked");
await mkdir(unpack, { recursive: true });
// Windows 10/11 ships bsdtar; argument arrays preserve spaces in local paths.
const extract = spawnSync("tar", ["-xf", archive, "-C", unpack], {
  stdio: "inherit",
});
if (extract.error || extract.status !== 0)
  throw new Error(
    `Unable to extract FFmpeg: ${extract.error?.message ?? extract.status}`,
  );
const source = path.join(unpack, `ffmpeg-${release}-full_build`);
await mkdir(destination, { recursive: true });
for (const binary of ["ffmpeg.exe", "ffprobe.exe"])
  await copyFile(
    path.join(source, "bin", binary),
    path.join(destination, binary),
  );
const license = (await readdir(source)).find((name) => /^license/i.test(name));
if (!license) throw new Error("FFmpeg license missing in publisher archive");
await copyFile(
  path.join(source, license),
  path.join(destination, "FFMPEG-LICENSE.txt"),
);
const readme = (await readdir(source)).find((name) => /^readme/i.test(name));
if (readme)
  await copyFile(
    path.join(source, readme),
    path.join(destination, "FFMPEG-README.txt"),
  );
await writeFile(
  path.join(destination, "FFMPEG-SOURCE.txt"),
  `FFmpeg ${release}, full Windows build by Gyan Doshi\nBinary release: ${url}\nArchive SHA-256: ${sha256}\nFFmpeg source: https://ffmpeg.org/releases/ffmpeg-${release}.tar.xz\nBuild configuration and dependency information: https://www.gyan.dev/ffmpeg/builds/\nThe bundled GPL FFmpeg executables run as separate processes. See FFMPEG-LICENSE.txt and publisher README.\nBefore public redistribution, provide the corresponding source for this exact build and its GPL dependencies as required by their licenses. A link alone is not a complete redistribution package.\n`,
);
if (process.platform === "win32") {
  const ffmpeg = path.join(destination, "ffmpeg.exe");
  const filters = spawnSync(ffmpeg, ["-hide_banner", "-filters"], {
    encoding: "utf8",
  });
  const encoders = spawnSync(ffmpeg, ["-hide_banner", "-encoders"], {
    encoding: "utf8",
  });
  if (filters.status !== 0 || encoders.status !== 0)
    throw new Error("Bundled FFmpeg cannot run");
  for (const name of ["zscale", "tonemap", "scale", "fps"])
    if (!filters.stdout.includes(name))
      throw new Error(`Missing required filter: ${name}`);
  for (const name of ["libx264", "libx265", "h264_nvenc", "hevc_nvenc", "aac"])
    if (!encoders.stdout.includes(name))
      throw new Error(`Missing required encoder: ${name}`);
  const probe = spawnSync(path.join(destination, "ffprobe.exe"), ["-version"], {
    encoding: "utf8",
  });
  if (probe.status !== 0) throw new Error("Bundled ffprobe cannot run");
}
// Reading these files catches missing/empty license output before packaging.
if (
  !(await readFile(path.join(destination, "FFMPEG-LICENSE.txt"), "utf8")).trim()
)
  throw new Error("Empty license");
console.log(`Windows engine and license materials ready: ${destination}`);

// Extraction is temporary; retain only the checked archive and bundled files.
await rm(unpack, { recursive: true, force: true });
