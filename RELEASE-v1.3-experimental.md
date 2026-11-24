# TITAN CORE v1.3-experimental (Performance)

**EXPERIMENTAL PRERELEASE.** This combines the stable v1.3 Kraken Mount 0.3.2 integration with the rolled-back, previously working performance baseline of v1.2-experimental. Stable v1.3 remains the latest stable release; neither stable main nor the older releases are modified.

## New since v1.2-experimental

- Selected Kraken Mount v0.3.2 source fixes from pinned upstream commit dbe78a7a7c8d3f2381247ca478fbef65f4c58b77, identical in scope to stable v1.3.
- Extract secondary ELFs outside decrypted/ for exit-spawn apps; existing PKG caches rebuild once. Folder cache stamps remain unchanged.
- Release owned sandbox nullfs mounts on exit, retain the library overlay for a bounded exit-spawn grace period, restore needed mounts on same-title exec, and adopt known package-folder and font mounts by identity.
- Mount available system fonts read-only.
- Recover the web listener after accept errors such as rest-mode wake; protect payload takeover while work, installations or games are active.
- FW12+ home-screen tile batch registration, retaining TITAN CORE's ownership marker and lightweight app.db byte check (not the upstream full SQLite query).
- Updated README, component provenance and credits.

## Additional differences from stable v1.3

Retains earlier performance optimizations: one backport directory snapshot per scan; mounted drive alias snapshot; direct /data free-space checks; skip unnecessary failure fingerprints; hash-indexed title registration, dashboard status, library merge and native ownership; static HTML/AppCache reuse and borrowed HTTP responses; fewer pending-exec title-set clones; allocation-free AMPR chunk planning; optional scan/cache timing logs and web latency benchmark.

**Excluded:** The custom AMPR scheduler/cache-retention runtime and later rewritten AMPR index/manifest generator, cache validation and scan reconciliation batch that were reverted after the reported mount failures. No patched AMPR SPRX is included. No experimental performance gain is claimed without console measurements.

Same scope as stable v1.3: upstream Playden API/scan exclusions, folder overlay toggle, crash report UI, self-updater, payload-manager repository and library redesign are not ported. TITAN CORE retains its own installer, web UI and ShadowMount integration.

| Component | Version |
|---|---|
| TITAN CORE | 1.3.0-experimental |
| Kraken Mount | 0.3.2 (selected source integration) |
| AMPR-EMU-NG | Official upstream 0.5.1 |
| Kstuff Lite / FPKG | 1.13-fpkg-dr-test8-rest-fast |
| ShadowMountPlus | 1.7beta12 |

Kstuff SHA-256: 221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc. Official AMPR SHA-256: 7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d.

## Testing

Download TITAN-CORE-experimental.elf and SHA256SUMS. Reboot before switching resident builds. Preserve caches and saves; the normal cache rebuild handles the new stamp. Compare mounts, warm/cold loading, secondary ELF exits, fonts and rest mode with stable v1.3. CI build checks do not establish physical-console compatibility.

Credits: devchmod, quer3q, xtrackkkz, Zhuwenxue2002, EchoStretch, drakmor and upstream contributors. See CREDITS.md and source provenance.
