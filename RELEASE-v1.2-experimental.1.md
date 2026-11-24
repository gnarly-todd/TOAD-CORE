# TITAN CORE v1.2 Experimental Performance 1

**EXPERIMENTAL PRERELEASE — not a replacement for stable v1.2.** This is the rollback build from before the custom AMPR experiment. Stable v1.2 and main remain untouched.

## Changes from stable v1.2

- Backport discovery: enumerate and sort backport directories once per scan, preserving exact-match and owner-root priority.
- Storage discovery: snapshot mounted USB/extended-storage aliases once per scan, instead of querying them per title; query internal SSD free space directly via /data.
- Cache/scan processing: avoid recursive input fingerprint calculations for titles with no previous failure.
- Title registration and dashboard: use hash indexes for status preservation, visible titles and registration reconciliation, replacing repeated linear searches.
- Library and ShadowMount inventory: index title/path ownership and native entries, reducing repeated inventory comparisons.
- Web interface: cache the versioned HTML and AppCache hash; borrow static response bytes rather than allocating a fresh copy; index external installs for API serialization.
- Launch watcher: avoid cloning the configured title set on pending-PID polls unless a PID resolves. Existing mount timing, readiness gates, overlay order and ownership checks remain intact.
- AMPR index planning: eliminate transient chunk-plan allocations during streamability checks. Index format and official AMPR binary remain unchanged.
- Diagnostics: optional discovery/cache-build timing logs, performance audit, AMPR review and host-side web latency benchmark.

## Mount regression rollback

The subsequent custom AMPR runtime (queue scheduling and decoded-cache retention), rewritten index/manifest generator, cache validation changes and scan reconciliation changes were reverted together after the maintainer reported that games did not mount and failed to launch. The precise cause was not isolated. This release restores the exact source tree of the earlier successful CI build; the maintainer reported that the previous build worked on PS5. No patched AMPR runtime is included. Comparative loading speed, broad game compatibility, gameplay and rest-mode stability have not been established.

## Pinned components (same as v1.2)

| Component | Version |
|---|---|
| AMPR-EMU-NG | Official upstream 0.5.1 |
| Kstuff Lite / FPKG | 1.13-fpkg-dr-test8-rest-fast |
| ShadowMountPlus | 1.7beta12 |
| Kraken Mount | 0.3.1 (partial source integration) |

Kstuff SHA-256: `221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc`. Official AMPR SHA-256: `7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d`.

## Build and testing

- Source: [rollback commit](https://github.com/gnarly-todd/TOAD-CORE/commit/1be3852bf914202608f5a560a1e29883c5743792), identical tree to [original successful build](https://github.com/gnarly-todd/TOAD-CORE/actions/runs/45305282232).
- The attached ELF is the original CI artifact, renamed to distinguish it from stable. SHA-256: `0c9248f588d6b1203ccf8fa8da8a1fa6eeeaea54c6a430aece0f2cb544e9d00f`.
- Internal app version remains 1.2.0; the release tag distinguishes this experimental build.
- Reboot before switching builds. Bundled-library ownership logic restores official AMPR if the installed copy is identified as a prior bundled copy. Preserve game caches and saves; do not delete manually installed custom libraries indiscriminately.
- Compare cold/warm launches, mount success, loading screens, saves, gameplay and rest mode against stable v1.2. No measured performance gains are claimed.
- [Stable v1.2](https://github.com/gnarly-todd/TOAD-CORE/releases/tag/v1.2) remains the recommended fallback.

Credits: devchmod, quer3q, EchoStretch, drakmor and all upstream contributors. See CREDITS.md and upstream licenses.
