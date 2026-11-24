# AMPR v0.5.1 performance review (experimental)

Reviewed 2026-11-23 against `quer3q/ampr-emu-ng` release v0.5.1, commit `e1789c11bf36c2f4e9b67c5452935c1816674a2c`, and the TITAN CORE `experimental/performance` integration. This is **source review**, not a console performance claim.

## What upstream already solved

- v0.5.1 overlaps end-of-packet completion with the next lane's reads, raises the pread pool ceiling from 6 to 8, and fixes save and directory issues in PKG mode. Upstream reported roughly 10% faster read batches in tested PKG workloads; one badly queued zstd title improved from 98 to 5 ms per batch. Those figures are **upstream results**, not TITAN measurements.
- v0.5.0 accelerated index load and introduced compact/expandable index and cache structures. This TITAN CORE revision already pins v0.5.1, so no upstream version change is needed.
- Upstream `docs/read-path-tuning.md` reports that blanket read-ahead increased disk traffic and worst latency in sampled games. Increasing the pread limit to 32 also risked game thread creation. Do not enable either by default.

## Changes on experimental/performance

1. `src/ampr_index.rs`: Streamability pre-validation now visits chunk descriptors without allocating an encoded chunk vector; manifest generation still encodes those descriptors into the **same** output via one shared function. The former path allocated and discarded a temporary descriptor vector before allocating again for the final manifest. This can reduce allocation pressure on **first PKG cache preparation**, not in-game read latency. Build and hardware validation required.

No changes to the binary `vendor/ampr-emu-ng/libSceAmpr.sprx`, AMPR index on-disk formats, PKG chunk flags or the user-provided `1.13-fpkg-dr-test8-rest-fast` kstuff ELF.

## Remaining candidate improvements, in order

1. **Quantify where time goes on hardware**. Test the official v0.5.1 debug SPRX separately (never deploy it as the release default). Its `:9090/metrics` page exposes `index_mount_ns`, `job_total_ns/job_totals`, `job_first_issue_ns/job_first_issues`, `apr_read_ns/apr_reads`, `pack_job_ns/pack_jobs`, `pread_queue_ns/pread_reads`, `cache_miss_repeat/cache_misses`, `pack_disk_bytes`, `pool_alloc_failed` and thread-pressure counters. Compare cold/warm starts of the same test PKG and storage location, ideally five runs each. Note that debug metrics reset some maxima on each read.
2. **Detect missing AMPR index outputs in a fresh TITAN cache**. `src/cache.rs::check` verifies `kraken.image` for a PKG but does not explicitly require both `ampr_emu.index` and `ampr_assets.index`. Missing outputs may leave `.done` valid and defer failure to game launch. A future narrowly-scoped repair should check existence/cheap headers; avoid full-file CRC on every scan.
3. **Profile memory and allocation at first cache build**. `src/ampr_index.rs::build_manifest` creates complete file/chunk/string buffers, then a combined output buffer, and `build_index` similarly copies interim tables into the final vector. Direct assembly into pre-sized final buffers could reduce peak host/console RAM and CPU time, but requires byte-for-byte fixture comparisons before altering the writer.
4. **A/B cache protection only where repeat misses dominate**. The upstream decoded cache uses `cache_protect_hits=3` and `cache_protect_pct=50` by default. Debug-only trials may be warranted for a workload with high `cache_miss_repeat`, keeping exact cold/warm scenarios constant and watching eviction, read bytes, and thread failures. No blanket default modification justified yet.
5. **Avoid a speculative thread increase or read-ahead**. v0.5.1's 8-thread ceiling is already tuned; more workers consume the game's thread budget, and upstream measurements showed read-ahead can worsen tail latency.

## Safety and validation

- Do not alter a game while running, mounted writable files, user saves, manual AMPR files, or an owner-managed replacement.
- A GitHub Actions `cargo check` and two-stage PS5 payload build are the minimum validation; real PS5 startup and gameplay measurements are separate.
- Compare PKG mode against the existing v1.2 baseline. Improvements in initial cache build must not be presented as improvements in frame rate or read latency.
- Keep `main`, release v1.2, Kstuff fast Test 8, ShadowMount, and upstream AMPR binary unchanged pending tests.

Sources:
- https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.1
- https://github.com/quer3q/ampr-emu-ng/blob/v0.5.1/docs/read-path-tuning.md
- https://github.com/quer3q/ampr-emu-ng/blob/v0.5.1/docs/debug-build.md
- https://github.com/gnarly-todd/TOAD-CORE/blob/experimental/performance/src/ampr_index.rs

## Separate patched AMPR runtime (new experimental target)

The independent source patch at review/ampr-experimental/apply.py checks out
exactly upstream AMPR-EMU-NG v0.5.1 commit
e1789c11bf36c2f4e9b67c5452935c1816674a2c. This is not a byte patch
of the bundled official v0.5.1 binary; that official file stays unchanged
and continues to be checked by SHA-256 on every ordinary build.

AMPR runtime changes in the patched source:

1. pack/schedule.rs: when no jobs exist in a priority lane under PACK_M,
   do not recheck that same empty lane 2-4 times during one weighted
   scheduling pass. Retain the 4:2:1 cursor, FIFO order, latency/bulk
   reservation and priority decisions. Delete the redundant second sweep
   because the weighted pass already checks all three lanes. Include a
   regression test of the empty-queue cursor.
2. rt/tune.rs: experiment with decoded-cache protection at 2 hits
   (upstream 3) and 60% maximum protected share (upstream 50%).
   These are explicitly unmeasured settings and must be A/B compared.
   The debug runtime provides existing cache and I/O metrics and tuning.

The separate .github/workflows/experimental-ampr.yml workflow checks out
the source, confirms its git SHA, applies the patch, runs upstream host
tests, builds release and debug .sprx with upstream's official PS5 SDK
toolchain, and builds a TITAN CORE ELF with an explicit Cargo feature.
The official SPRX remains the default in normal builds; the alternate
TITAN_EXPERIMENTAL_AMPR=1 path is opt-in. The artifact is named
TITAN-CORE-experimental-AMPR and contains the embedded-test ELF, both
custom SPRXs, and SHA-256 checksums.

No speculative pread-thread increase, memory-cap increase, read-ahead,
Kraken decode format changes, live mount/unmount changes, or service
readiness reductions were introduced. Upstream v0.5.1 already provides
load-responsive worker growth, cache expansion from freed index memory,
Kraken scratch reuse and game-thread pressure recovery. The custom build
is not a performance or stability confirmation until PS5 comparisons
have run on representative PKG and folder games.
