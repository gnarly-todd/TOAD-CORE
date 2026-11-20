#!/usr/bin/env bash
# Builds a standalone original PS5 diagnostic using the same PS5 SDK as TOAD-CORE.
# This replaces the earlier freestanding/static-PIE experiment.
set -euo pipefail
cd "$(dirname "$0")/../.."

output=TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v2.elf
image=titan-core-ps5
if ! command -v docker >/dev/null 2>&1; then
  echo 'ERROR: Docker is required. Run in your TOAD-CORE GitHub Codespace.' >&2
  exit 1
fi
if ! docker image inspect "$image" >/dev/null 2>&1; then
  echo 'Preparing TOAD-CORE PS5 SDK image...'
  docker build -t "$image" ps5
fi

docker run --rm --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w /work \
  "$image" bash -euc '
    "$PS5_PAYLOAD_SDK/bin/prospero-clang" -std=c11 -O2 -Wall -Wextra -Werror \
      -o TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v2.elf \
      review/a53-1360/original-preflight.c -lkernel_web
    llvm-readelf -h TOAD-A53-ORIGINAL-PS5-PREFLIGHT-v2.elf | grep -E "ELF64|Machine:|Entry point"
  '

sha256sum "$output"
echo "BUILT: $PWD/$output"
echo 'This is a userland diagnostic, NOT an A53 execution or FPKG payload.'
