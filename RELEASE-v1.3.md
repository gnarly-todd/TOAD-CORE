# TITAN CORE v1.3 — Kraken Mount 0.3.2 integration

Stable-channel successor to v1.2. This is based on main, not the experimental performance branch. The previous v1.2 and v1.2-experimental releases and assets are preserved.

## What's new

- Ported selected upstream Kraken Mount v0.3.2 fixes from commit dbe78a7a7c8d3f2381247ca478fbef65f4c58b77.
- Extract secondary ELF files outside decrypted/ for PKG exit-spawn apps. Existing PKG caches rebuild once using TITAN CORE's own cache stamp; folder cache stamps are unchanged. Game-written files are preserved by the existing cache builder.
- Exit handling releases owned sandbox nullfs layers immediately and retains the library overlay for a bounded 3-second exit-spawn grace period. Same-title exec cancels cleanup and restores needed source/data mounts. Adopt known fonts and package-folder mounts by identity. Existing game pause and mount ownership guards remain.
- Mount available system fonts read-only inside the game sandbox.
- Restore web listener after an accept error such as a rest-mode wake; guard payload takeover against active work, installation and games.
- Queue the TITAN CORE home-screen tile for the existing batched registration call on firmware without per-title install (including FW 12+). Retain the folder ownership marker and existing lightweight app.db byte check; this does not port upstream's full SQLite row query and may not detect a deleted row still present in unused DB pages.

## Integration scope

This remains a selective source integration, not a replacement with the upstream Kraken Mount ELF. Upstream Playden API/scan exclusions, per-folder overlay toggle, crash-report UI, self-updater, payload-manager repository and upstream library redesign are not included. TITAN CORE preserves its own updater/installer, UI, game source ownership and ShadowMount coordination. No experimental AMPR runtime or other experimental branch changes are included.

| Component | Version |
|---|---|
| Kraken Mount | 0.3.2 (selected source ports) |
| AMPR-EMU-NG | Official upstream 0.5.1 |
| Kstuff Lite / FPKG | 1.13-fpkg-dr-test8-rest-fast |
| ShadowMountPlus | 1.7beta12 |

Kstuff SHA-256: 221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc. Official AMPR SHA-256: 7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d.

## Use and validation

Download TITAN-CORE.elf and SHA256SUMS. Reboot before switching resident payload versions. Use your usual loader and existing console setup; access the local dashboard at http://PS5-IP:8097. Application version 1.3.0; process identity titan-core.elf. Existing PKG caches may take time to rebuild on first scan; do not delete game saves or manually purge caches unnecessarily.

This stable-channel release is build-validated, not a guarantee of physical-console stability. Verify game mounts, secondary-ELF launches, normal exits, rest-mode recovery and tile registration on hardware. No measured game loading-speed improvement is claimed.

Credits: devchmod, quer3q, xtrackkkz, Zhuwenxue2002, EchoStretch, drakmor and all upstream authors. [Upstream release](https://github.com/quer3q/kraken-mount/releases/tag/v0.3.2).
