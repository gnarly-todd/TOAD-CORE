#!/usr/bin/env bash
# Build original PS5 diagnostic v3 with the same SDK as TOAD-CORE.
# No changes to the production loader or bundled dependencies.
set -euo pipefail
cd "$(dirname "$0")/../.."

output=TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v3.elf
image=titan-core-ps5
if ! command -v docker >/dev/null 2>&1; then
  echo 'ERROR: Docker required. Run in your TOAD-CORE Codespace.' >&2
  exit 1
fi
if ! docker image inspect "$image" >/dev/null 2>&1; then
  echo 'Preparing the PS5 SDK image...'
  docker build -t "$image" ps5
fi
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  "$image" bash -euc '
    "$PS5_PAYLOAD_SDK/bin/prospero-clang" -std=c11 -O2 -Wall -Wextra -Werror \
      -o TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v3.elf \
      review/a53-1360/original-preflight-v3.c -lkernel_web
    llvm-readelf -h TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v3.elf | grep -E "Class:|Machine:|Type:|Entry point"
  '
sha256sum "$output"
echo "BUILT: $PWD/$output"
echo 'Diagnostic only; no A53 memory reads or changes to installed games.'
