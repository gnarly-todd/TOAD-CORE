#!/usr/bin/env bash
# One-time pinned upstream snapshot import. Intended for CI.
set -euo pipefail
cd "$(dirname "$0")/.."
root="$PWD"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
kraken_sha=387e665f6372e1a11ae834c52e6f77ce99b2012b
ampr_sha=d20210e224871864bee0888193c7c1062890639c
ampr_binary_sha=7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174
git clone --quiet --depth 1 --branch v0.3.1 https://github.com/quer3q/kraken-mount.git "$tmp/kraken"
git clone --quiet --depth 1 --branch v0.5.0 https://github.com/quer3q/ampr-emu-ng.git "$tmp/ampr"
[[ "$(git -C "$tmp/kraken" rev-parse HEAD)" == "$kraken_sha" ]] || { echo "Wrong Kraken source commit" >&2; exit 1; }
[[ "$(git -C "$tmp/ampr" rev-parse HEAD)" == "$ampr_sha" ]] || { echo "Wrong AMPR source commit" >&2; exit 1; }
mkdir -p vendor/kraken-mount
git -C "$tmp/kraken" archive --format=tar.gz --prefix=kraken-mount-v0.3.1/ -o "$root/vendor/kraken-mount/upstream-v0.3.1.tar.gz" HEAD
git -C "$tmp/ampr" archive --format=tar.gz --prefix=ampr-emu-ng-v0.5.0/ -o "$root/vendor/ampr-emu-ng/upstream-v0.5.0.tar.gz" HEAD
curl --fail --location --silent --show-error --retry 4 --proto '=https' \
  https://github.com/quer3q/ampr-emu-ng/releases/download/v0.5.0/libSceAmpr.sprx \
  -o "$tmp/libSceAmpr.sprx"
printf '%s  %s\n' "$ampr_binary_sha" "$tmp/libSceAmpr.sprx" | sha256sum --check --strict
cp "$tmp/libSceAmpr.sprx" vendor/ampr-emu-ng/libSceAmpr.sprx
printf '0.5.0\n' > vendor/ampr-emu-ng/VERSION
python3 - <<'PY'
from pathlib import Path
s=Path("vendor/ampr-emu-ng/SHA256SUMS")
lines=s.read_text().splitlines()
out=[("7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174  libSceAmpr.sprx" if line.endswith("  libSceAmpr.sprx") else line) for line in lines]
assert sum(x.endswith("  libSceAmpr.sprx") for x in out)==1
s.write_text("\n".join(out)+"\n")
PY
(cd vendor/ampr-emu-ng && sha256sum --check --strict SHA256SUMS)
cat > vendor/kraken-mount/PROVENANCE.md <<'EOF'
# Kraken Mount upstream source snapshot

Official release: https://github.com/quer3q/kraken-mount/releases/tag/v0.3.1
Pinned source commit: 387e665f6372e1a11ae834c52e6f77ce99b2012b
Archive: upstream-v0.3.1.tar.gz, generated from git archive of the pinned source.
This is a reference snapshot for porting selected fixes; it is not a second
resident Kraken Mount payload. TITAN CORE's custom launcher, installer and
package handling remain independent. Original GPL and author notices apply.
EOF
cat > vendor/ampr-emu-ng/UPSTREAM_V050.md <<'EOF'
# AMPR upstream 0.5.0 source and runtime

Official release: https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.0
Source commit: d20210e224871864bee0888193c7c1062890639c
Archive: upstream-v0.5.0.tar.gz generated from git archive of the pinned source.
libSceAmpr.sprx is the official non-debug 0.5.0 release asset.
SHA-256: 7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174
Historical custom source and binaries remain for attribution; they are not active.
EOF
echo "Verified source tags, archives, version and runtime SHA-256."
