#!/usr/bin/env bash
# Import official non-debug AMPR v0.5.1, matching the uploaded SPRX SHA exactly.
# This does not touch the persistently pinned Fast Kstuff binary.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
NEW=7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d
OLD=7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174
COMMIT=e1789c11bf36c2f4e9b67c5452935c1816674a2c
python3 review/verify-kstuff-fast-pin.py
test "$(cat vendor/ampr-emu-ng/VERSION)" = "0.5.0"
printf '%s  %s\n' "$OLD" vendor/ampr-emu-ng/libSceAmpr.sprx | sha256sum -c --strict
curl --fail --location --silent --show-error --retry 4 \
  https://github.com/quer3q/ampr-emu-ng/releases/download/v0.5.1/libSceAmpr.sprx \
  -o "$tmp/libSceAmpr.sprx"
test "$(wc -c < "$tmp/libSceAmpr.sprx")" = "706976"
printf '%s  %s\n' "$NEW" "$tmp/libSceAmpr.sprx" | sha256sum -c --strict
LC_ALL=C grep -aq 'ampr-emu-ng 0.5.1' "$tmp/libSceAmpr.sprx"
git clone --quiet --depth 1 --branch v0.5.1 https://github.com/quer3q/ampr-emu-ng.git "$tmp/upstream"
test "$(git -C "$tmp/upstream" rev-parse HEAD)" = "$COMMIT"
git -C "$tmp/upstream" archive --format=tar.gz \
  --prefix=ampr-emu-ng-v0.5.1/ -o "$ROOT/vendor/ampr-emu-ng/upstream-v0.5.1.tar.gz" HEAD
cp "$tmp/libSceAmpr.sprx" vendor/ampr-emu-ng/libSceAmpr.sprx
printf '0.5.1\n' > vendor/ampr-emu-ng/VERSION
python3 - <<'PY'
from pathlib import Path
p=Path("vendor/ampr-emu-ng/SHA256SUMS")
lines=p.read_text().splitlines()
old="7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174  libSceAmpr.sprx"
new="7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d  libSceAmpr.sprx"
assert lines.count(old)==1, "AMPR checksum listing has unexpectedly changed"
p.write_text("\n".join(new if line==old else line for line in lines)+"\n")
PY
cat > vendor/ampr-emu-ng/TITAN_BRANDING.md <<'EOF'
# AMPR v0.5.1 provenance (TITAN CORE)

The active libSceAmpr.sprx is the unmodified official non-debug v0.5.1 binary.
Release: https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.1
The uploaded libSceAmpr(1).sprx matched by exact full SHA-256.
SHA-256: 7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d
Length: 706,976 bytes.
Pinned upstream source commit: e1789c11bf36c2f4e9b67c5452935c1816674a2c.
Source snapshot: upstream-v0.5.1.tar.gz.
Historical 0.5.0 source is retained for attribution, not as the active runtime.

The release library is not patched or rebranded. TITAN CORE replaces its own
previously bundled AMPR copy when the stored ownership marker identifies it.
Manually installed custom/debug runtime libraries are preserved.
No physical-console performance or compatibility confirmation is claimed.
EOF
cat > vendor/ampr-emu-ng/UPSTREAM_V051.md <<'EOF'
# AMPR-EMU-NG v0.5.1

Release: https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.1
Source commit: e1789c11bf36c2f4e9b67c5452935c1816674a2c
Source archive: upstream-v0.5.1.tar.gz (verified git archive).
Official non-debug runtime: libSceAmpr.sprx
SHA-256: 7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d
The supplied user SPRX has the identical hash. Console validation is pending.
EOF
(cd vendor/ampr-emu-ng && sha256sum -c --strict SHA256SUMS)
python3 review/verify-kstuff-fast-pin.py
python3 review/verify-unified-payloads.py
python3 review/sync-component-readme.py --write
python3 review/sync-component-readme.py --check
echo "Verified AMPR v0.5.1 upgrade, Fast Kstuff retained."
