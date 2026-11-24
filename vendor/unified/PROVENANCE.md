# Unified payload provenance — 2026-11-20

This TITAN CORE fork embeds the owner's two supplied ELF files in place of the original TITAN CORE bundled runtime payloads:

- `kstuff1.13-fpkg-dr-test8-rest-fast.elf` — SHA-256 `221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc`, 1,845,320 bytes. Fast/PPR variant. Embedded strings report `Welcome To Kstuff Lite 1.13-fpkg-dr-test8-rest`.
- `shadowmountplus.elf` — SHA-256 `8755172ec9bef1bff5453ea75606e0caf7c41b5963f94f5a1087e7e8c6d9b1bd`, 3,626,576 bytes. Embedded strings report `1.7beta12`.

The new kstuff ELF was recovered from a public archive and independently SHA-256 verified against the user's exact uploaded ELF; ShadowMountPlus is byte-for-byte unchanged. These are user-provided binaries, **not** binaries verified against upstream tagged-release digests or matching source revisions. The old service source snapshots and release metadata have been removed from the current branch. Third-party dependency source archives and upstream license notices are retained; they do not establish matching source for the new ELFs. Corresponding source for these specific test binaries should be obtained from their authors if distributed.

`src/stack.rs` embeds these exact filenames and writes them to new versioned paths under `/data/titan-core/payloads` so an older unpacked ELF is not silently reused. Startup remains a one-attempt-per-boot sequence: ShadowMount first creates the automount sentinel, then kstuff can start. Existing resident processes are not terminated or reloaded, and different files at an installed destination are not overwritten. The bundled ShadowMount readiness check now requires the observed `1.7beta12` version string rather than accepting any 1.7 build.

The read-only KEKCALL_CHECK probe in `src/stack.rs` was derived from an earlier ShadowMountPlus implementation, and the read-only process enumeration likewise follows that implementation's layout. **These contracts have not been independently validated against the new user-supplied binaries**. Firmware 13.60 is the intended target, not confirmed runtime compatibility. No game launch or kernel payload execution was performed in this workspace.

`ps5/build.sh` verifies the two uploaded SHA-256 values via `vendor/unified/SHA256SUMS` when `TITAN_UNIFIED_STACK=1`. Keep upstream credits, licenses, and third-party notices intact.

The Fast/PPR variant retains the original internal welcome version without a fast suffix, while filename and SHA-256 identify the distinct ELF. Real PS5 timing/stability has not been measured.
