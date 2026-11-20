#!/usr/bin/env bash
# Experimental A53 layout/readback ELF builder for PS5 Pro, firmware 13.60.
# Does not run the payload or change TOAD-CORE's existing AMPR loader.
set -euo pipefail
cd "$(dirname "$0")/../.."

SOURCE_COMMIT=13aa9d900ab87eb4ec89191091131f9b7620d243
SOURCE_URL=https://github.com/gangg111/ps5-a53-1360-research.git
SOURCE_DIR=target/a53-research-1360
IMAGE=titan-core-ps5
OUTPUT=TOAD-A53-READBACK-1360-PRO.elf

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
    /* TOAD-CORE: abort before privileged operations on other firmware. */
    if ((kernel_get_fw_version() & 0xffff0000u) != 0x13600000u) {
        fprintf(stderr, "TOAD A53 research: firmware must be exactly 13.60\\n");
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
  echo 'Preparing the existing TOAD-CORE PS5 SDK container...'
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
echo 'Do NOT load on original/Slim PS5, or alongside the TOAD unified stack.'
