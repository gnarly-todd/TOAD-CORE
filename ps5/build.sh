#!/usr/bin/env bash
# Builds target/ps5/titan-core.elf, the PS5 payload, in Docker (toolchain from ps5-dump-forge).
set -euo pipefail
cd "$(dirname "$0")/.."

# Keep the owner-approved Fast Kstuff ELF in every production build.
python3 review/verify-kstuff-fast-pin.py

if [[ "${1:-}" != --inside ]]; then
  docker build -t titan-core-ps5 ps5
  exec docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp \
    -e CARGO_BUILD_JOBS=2 -e TITAN_UNIFIED_STACK="${TITAN_UNIFIED_STACK:-0}" \
    -e TITAN_EXPERIMENTAL_AMPR="${TITAN_EXPERIMENTAL_AMPR:-0}" \
    -e CARGO_HOME=/work/target/ps5/cargo -v "$PWD:/work" titan-core-ps5 bash ps5/build.sh --inside
fi

# The bundled ampr-emu-ng must be the released file (SHA256SUMS copied from its GitHub release).
(cd vendor/ampr-emu-ng && sha256sum -c --ignore-missing --strict SHA256SUMS)

features=()
if [[ "${TITAN_UNIFIED_STACK:-0}" == 1 ]]; then
  # Fail before compilation if historical source archives or stale binaries are substituted.
  python3 review/verify-unified-payloads.py
  (cd vendor/unified && sha256sum -c --strict SHA256SUMS)
  (cd vendor/dlc_emu && sha256sum -c --strict SHA256SUMS)
  (cd vendor/pgo_stub && sha256sum -c --strict SHA256SUMS)
  features=(--features unified-stack)
fi
out=target/ps5
cc="$PS5_PAYLOAD_SDK/bin/prospero-clang"
mkdir -p "$out/shim"
# Web-uploaded sources may not preserve Unix executable bits.
chmod +x ps5/linker.sh
for src in ps5/shim/freebsd11.c ps5/shim/syscalls.S ps5/entry.c ps5/compat.c; do
  name=$(basename "${src%.*}")
  "$cc" -O2 -Wall -Wextra -Werror -c "$src" -o "$out/shim/$name.o"
done

export RUSTC_BOOTSTRAP=1
# The PS5 kernel fills FreeBSD 11 structs (stat 120 bytes, dirent 264, old statfs); libc and the
# std built here default to FreeBSD 12's. libc >= 0.2.187 reads this cfg; rustflags reach std
# too under -Z build-std.
export CARGO_TARGET_X86_64_PS5_FREEBSD_RUSTFLAGS='--cfg libc_unstable_freebsd_version="11"'
export CARGO_TARGET_DIR="$out"
export CARGO_TARGET_X86_64_PS5_FREEBSD_LINKER=/work/ps5/linker.sh
export CC_x86_64_ps5_freebsd="$cc" AR_x86_64_ps5_freebsd="$PS5_PAYLOAD_SDK/bin/prospero-ar"
# Cargo doesn't see the C objects or linker.sh: drop the linked output so a shim change relinks.
build() {
  rm -f "$out"/x86_64-ps5-freebsd/release/deps/titan_core-*
  cargo build --release --locked \
    --target ps5/x86_64-ps5-freebsd.json -Z json-target-spec \
    -Z build-std=std,panic_abort -Z build-std-features=panic-unwind "$@"
}
# Stage 1 is the copy the tile starts (/data/titan-core/titan-core.elf); stage 2, the ELF you
# push, embeds it and saves it there (as Dump Forge's self copy).
build "${features[@]}"
cp "$out/x86_64-ps5-freebsd/release/titan-core" "$out/titan-core-stage1.elf"
TITAN_SELF_ELF="/work/$out/titan-core-stage1.elf" build "${features[@]}" --features self-copy
cp "$out/x86_64-ps5-freebsd/release/titan-core" "$out/titan-core.elf"
llvm-readelf -h -d "$out/titan-core.elf" > "$out/titan-core.elf.txt"
ls -l "$out/titan-core.elf"
