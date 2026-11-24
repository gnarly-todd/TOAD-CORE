# Kraken Mount v0.3.1 / AMPR-EMU-NG v0.5.1 integration

## Verified sources and current bundle

Kraken Mount v0.3.1 is a selective source integration, not a second payload.
Upstream Kraken commit: 387e665f6372e1a11ae834c52e6f77ce99b2012b.
Archived Kraken source: vendor/kraken-mount/upstream-v0.3.1.tar.gz.

AMPR v0.5.1 official release:
https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.1

Official non-debug libSceAmpr.sprx SHA-256:
7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d

The supplied library matches this official release exactly, size 706,976 bytes.
AMPR pinned source commit: e1789c11bf36c2f4e9b67c5452935c1816674a2c.
Archived source: vendor/ampr-emu-ng/upstream-v0.5.1.tar.gz.
Historical v0.5.0 source is retained for attribution.

## Upstream AMPR 0.5.1 changes

Faster read batches through overlapping work, increased disk read pool size, and changes for pkg-mode save icons, shared save mounts and directory listings. These are reported by upstream, not independently tested in this fork.

## Selectively ported Kraken code

One-level nested scan root discovery, buffered metadata copying, per-title playgo_off configuration, resilient array parsing and firmware-12+ fallback batch registration.

Unported Kraken UI features include per-game PlayGo UI controls, multilingual title media, automatic removal of missing titles and USB unmount. TITAN CORE independently retains its installer, source cache ownership, original-path debug PKG handling and ShadowMount coordination.

## Limitations and attribution

The release workflow validates upstream source commit, checksums and PS5 ELF compilation. Gameplay, save compatibility, performance and actual console process display still require testing. Manually supplied custom AMPR files are preserved if not owned by TITAN CORE's bundle. Native A53 code execution has not been demonstrated.

TITAN CORE by devchmod; AMPR/Kraken by quer3q; kstuff and ShadowMountPlus copyrights belong to their respective upstream contributors.
