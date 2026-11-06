#!/usr/bin/env bash
# Lance réellement Cargo Windows depuis WSL2, sans cross-compilation Linux.
set -euo pipefail
project_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$project_root"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$project_root/scripts/build-windows.ps1")" "$@"
