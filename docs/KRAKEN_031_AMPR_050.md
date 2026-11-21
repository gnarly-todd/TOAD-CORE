# Kraken Mount v0.3.1 / ampr-emu-ng v0.5.0 integration

## Verified upstream releases (2026-11-20)

Kraken Mount: https://github.com/quer3q/kraken-mount/releases/tag/v0.3.1
Pinned tag/commit: `387e665f6372e1a11ae834c52e6f77ce99b2012b`.
Complete upstream source archive: `vendor/kraken-mount/upstream-v0.3.1.tar.gz`.

AMPR: https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.0
Pinned source commit: `d20210e224871864bee0888193c7c1062890639c`.
Complete upstream source archive: `vendor/ampr-emu-ng/upstream-v0.5.0.tar.gz`.
Embedded official *non-debug* runtime: `vendor/ampr-emu-ng/libSceAmpr.sprx`.
Official SHA-256: `7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174`.

The snapshots were fetched from pinned GitHub tags, independently checked against their expected commit hashes, then copied into `vendor`. The runtime was independently checked against the official release digest. No second Kraken Mount ELF is loaded alongside TITAN CORE.

## Kraken v0.3.1 changes ported into TITAN CORE

- One level of nested game folders in explicit scan roots; direct-root copies win over nested duplicates. External raw installer archives remain unplayable until a validated install.
- Stage `sce_sys`, icon and appmeta files using 1 MiB buffered copying instead of the FreeBSD small-buffer path. Original source permissions and files remain untouched.
- `playgo_off = ["PPSA01556"]` in TITAN's config disables adding or upgrading the *bundled* PlayGo stub for that title while preserving game-owned libraries.
- Invalid title IDs within supported TOML setting arrays are skipped, preserving valid items.
- On firmware where `sceAppInstUtilAppInstallTitleDir` is missing, game registrations are staged and `sceAppInstUtilAppInstallAll` is called **once per scan pass**, with a single completion notification. One-off internal restoration uses the original immediate registration fallback.

## Current limitations

**This is a selective, source-grounded port, not a wholesale replacement of TITAN CORE with Kraken Mount.** The original TITAN CORE installer, restore journals, original-path package handling, cache ownership and ShadowMount coordination take precedence.

Kraken upstream features **not yet merged** include: a game-details UI toggle for PlayGo (the config setting works), multilingual title sound/metadata, USB unmount in the interface, and automatic removal of missing titles. Existing TITAN stability checks remain in place. The integrated runtime's game fixes are from unmodified AMPR v0.5.0, not claims of changes to the TITAN cache engine.

The build and synthetic tests verify compilation and code behavior, **not** real console operation, preservation of an existing save in every game, or firmware compatibility. Test on a closed-game reboot with disposable homebrew/debug FPKG first and keep user data backups. Loading the updated ELF does not itself prove native A53 code execution; native FPKG access continues to use the AMPR emulation path.

## Attribution

TITAN CORE remains attributed to devchmod; Kraken Mount and AMPR remain attributed to quer3q and their respective upstream contributors. Third-party GPL notices and historical sources are preserved. The A53 research tools under `review/a53-1360` are unrelated to the production package-routing change.
