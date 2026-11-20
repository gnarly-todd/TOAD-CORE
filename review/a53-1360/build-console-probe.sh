#!/usr/bin/env bash
# Experimental A53 layout/readback ELF builder for PS5 Pro, firmware 13.60.
# Does not run the payload or change TITAN CORE's existing AMPR loader.
set -euo pipefail
# This user's console is an ORIGINAL PS5, not the PS5 Pro used for the research.
# This helper must be consciously unlocked only for independently confirmed Pro units.
if [[ "${TITAN_CONFIRMED_PS5_PRO_1360:-}" != "YES" ]]; then
  printf '%s\n' "BUILD REFUSED: This A53 probe is validated only on PS5 Pro at 13.60." >&2
  printf '%s\n' "For an original PS5 or Slim, do NOT build or load this experiment." >&2
  printf '%s\n' "The user's original PS5 needs an independently validated model-specific probe." >&2
  exit 2
fi
cd "$(dirname "$0")/../.."

SOURCE_COMMIT=13aa9d900ab87eb4ec89191091131f9b7620d243
SOURCE_URL=https://github.com/gangg111/ps5-a53-1360-research.git
SOURCE_DIR=target/a53-research-1360
IMAGE=titan-core-ps5
OUTPUT=TITAN-A53-READBACK-1360-PRO.elf

for tool in git docker python3 cp sha256sum; do
  command -v "$tool" >/dev/null || { echo "Missing required command: $tool" >&2; exit 1; }
done

if [[ ! -d "$SOURCE_DIR/.git" ]]; then
  mkdir -p target
  git clone "$SOURCE_URL" "$SOURCE_DIR"
fi

git -C "$SOURCE_DIR" fetch --quiet origin "$SOURCE_COMMIT"
git -C "$SOURCE_DIR" checkout --quiet --detach "$SOURCE_COMMIT"
actual=$(git -C "$SOURCE_DIR" rev-parse HEAD)
[[ "$actual" == "$SOURCE_COMMIT" ]] || { echo "Research source commit did not match pin" >&2; exit 1; }

source_file="$SOURCE_DIR/tools/a53layout/a53layout.c"
python3 - "$source_file" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
src = path.read_text()
marker = 'int main(void) {\n'
gate = '''int main(void) {
    /* TITAN CORE: abort before privileged operations on other firmware. */
    if ((kernel_get_fw_version() & 0xffff0000u) != 0x13600000u) {
        fprintf(stderr, "TITAN A53 research: firmware must be exactly 13.60\\n");
        return 2;
    }
'''
if src.count(marker) == 1:
    src = src.replace(marker, gate, 1)
elif src.count(gate) != 1:
    raise SystemExit('Upstream entry point changed; refusing an unguarded probe')
path.write_text(src)
PY

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo 'Preparing the existing TITAN CORE PS5 SDK container...'
  docker build -t "$IMAGE" ps5
fi

echo 'Cross-compiling standalone A53 research payload...'
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" -w "/work/$SOURCE_DIR/tools/a53layout" \
  "$IMAGE" make a53layout.elf

cp "$SOURCE_DIR/tools/a53layout/a53layout.elf" "$OUTPUT"
sha256sum "$OUTPUT"
echo "BUILT: $PWD/$OUTPUT"
echo 'WARNING: This is experimental, hardware-studied only on PS5 Pro 13.60.'
echo 'The source writes privileged kernel/device-register state to REQUEST A53 reads.'
echo 'Do NOT load on original/Slim PS5, or alongside the TITAN unified stack.'
