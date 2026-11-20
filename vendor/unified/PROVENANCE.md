# Unified payload provenance — 2026-11-20

This TOAD-CORE fork embeds the owner's two supplied ELF files in place of the original TITAN CORE bundled runtime payloads:

- `kstuff-fpkg-1.13-dr-test6.elf` — SHA-256 `b1f60c1ec1685904d396da16710dc9d5676bf0880abf74c65459c886010b9443`, 1,818,680 bytes. Embedded strings report `Welcome To Kstuff Lite 1.13-fpkg-dr-test6`.
- `shadowmountplus.elf` — SHA-256 `8755172ec9bef1bff5453ea75606e0caf7c41b5963f94f5a1087e7e8c6d9b1bd`, 3,626,576 bytes. Embedded strings report `1.7beta12`.

The Git blob object identifiers of both uploaded files match the Git blob hashes of the supplied local attachments. These are user-provided binaries, **not** binaries verified against upstream tagged-release digests or matching source revisions. Do not treat the older included source snapshots as exact source for these ELFs. The original `vendor/unified/sources/` archives and upstream notices are retained for historical provenance and licensing. Corresponding source for these specific test binaries should be obtained from their authors if distributed.

`src/stack.rs` embeds these exact filenames and writes them to new versioned paths under `/data/titan-core/payloads` so an older unpacked ELF is not silently reused. Startup remains a one-attempt-per-boot sequence: ShadowMount first creates the automount sentinel, then kstuff can start. Existing resident processes are not terminated or reloaded, and different files at an installed destination are not overwritten. The bundled ShadowMount readiness check now requires the observed `1.7beta12` version string rather than accepting any 1.7 build.

The read-only KEKCALL_CHECK probe in `src/stack.rs` was initially based on ShadowMountPlus commit `a8e400487effa1a63c81943ad16fef2d9ea83cfb` (`src/sm_kstuff.c` and `include/sm_kstuff.h`); the read-only process enumeration also follows that historical code's layout. **These contracts have not been independently validated against the new user-supplied binaries**. Firmware 13.60 is the intended target, not confirmed runtime compatibility. No game launch or kernel payload execution was performed in this workspace.

`ps5/build.sh` verifies the two uploaded SHA-256 values via `vendor/unified/SHA256SUMS` when `TITAN_UNIFIED_STACK=1`. Keep upstream credits, licenses, and third-party notices intact.
