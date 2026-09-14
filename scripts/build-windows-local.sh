#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# All compilation takes place on this Mac. Downloads are build dependencies only.
export PATH="$PWD/.cache/cross-tools/bin:$(brew --prefix llvm)/bin:$(brew --prefix lld)/bin:$PATH"
export XWIN_CACHE_DIR="$PWD/.cache/xwin"
# cargo-xwin handles the fully static CRT selected in .cargo/config.toml.
# Tauri's separate static-VCRuntime/dynamic-UCRT override conflicts with it.
export STATIC_VCRUNTIME=false
command -v cargo-xwin >/dev/null
npm run prepare:windows
npm exec tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc --no-bundle --config src-tauri/tauri.portable.conf.json
python3 scripts/portable-local.py package
