# TITAN CORE experimental performance audit

**Branch:** \`experimental/performance\`  
**Base:** released TITAN CORE v1.2 (source commit \`be94c9c164c89775d1129a916b15be7473187c0b\`)  
**Scope:** application startup, discovery, cache validation/build, title registration, launch watcher, resident services, HTTP UI, and user-triggered package installation. Rust runtime, PS5 shim, related native decoder crate, Python/JavaScript tooling and the production release workflow were inspected. Changes are intentionally isolated from \`main\`.

## Constraints and safety boundaries

- **No changes** to the owner-approved **Fast Kstuff test8-rest-fast**, SHA-256 \`221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc\`.
- AMPR-EMU-NG **0.5.1**, ShadowMountPlus **1.7beta12** and partial Kraken **0.3.1** remain unchanged. No byte patches to upstream binaries or research A53 payloads.
- No changes to package deletion, installation ownership, game save mounts, cache format, sandbox overlay order, kernel syscalls, startup sentinel, fallback registration policy or firmware gates.
- This branch is **experimental**. A clean compile does not prove faster game launch or PS5 compatibility. Do not merge until console-tested.

## Implemented optimizations (source-grounded, not benchmarked)

| Area | Original repeated work | Experimental change | Intended benefit |
| --- | --- | --- | --- |
| Backport discovery (\`src/scan.rs\`) | Each detected game enumerated, filtered and sorted each relevant \`backports/\` directory | Build a sorted backport-directory snapshot once per scan, preserve owner-root priority and exact-before-prefix matching | Fewer filesystem calls on internal/USB roots with many games |
| Drive detection (\`src/storage.rs\`, \`src/scan.rs\`) | \`drive_of()\` rechecked mounted USB/extended aliases via up to 10 \`statfs\` calls per title | Snapshot mounted aliases once per scan and reuse for per-source classification | Less per-title mount querying |
| Healthy scan processing (\`src/main.rs\`) | \`inputs_key()\` was eagerly evaluated for **every game** during failure memo comparison, even when no prior failure existed | Compute the potentially recursive input fingerprint **only when a failure record exists** | Avoid redundant backport walks, stats and JSON stamps on ready titles |
| Dashboard publication (\`src/main.rs\`) | Each new game linearly searched the entire prior game list for its status, and unstable-source reconciliation scanned current entries | Hash index prior \`(path,title_id)\` statuses, use a visible title set, retain first match | Linear-scale status updates without changing existing states |
| Title-set reconciliation (\`src/main.rs\`) | \`registered.retain()\` scanned all current games for each registered title | Precompute a set of available title ids once | Linear-scale reconciliation |
| Web page delivery (\`src/web.rs\`) | Every \`GET /\` rebuilt the ~1 MB embedded HTML by \`replace\`; AppCache hash was recomputed and static images were copied into owned response vectors | Store versioned HTML + AppCache once with \`OnceLock\`; borrow immutable static bytes using \`Cow\` for HTML, manifest, icons and background | Fewer allocations and CPU cycles when opening TITAN's UI |
| Library merge (\`src/library.rs\`) | API polls repeatedly scanned the ShadowMount list to establish title ownership and find matching paths | Index ShadowMount titles and \`(title_id,path)\` once per merge; index native output by path | Fewer comparisons as game count increases |
| External-install lookups (\`src/web.rs\`) | \`game_json()\` performed vector \`contains()\` for each source in every \`/api/state\` response | Build a hash set once per response | Cheaper library serialization |
| Diagnostics (\`src/main.rs\`) | No direct discovery/cache-build timing in the usual runtime log | When debug is enabled, log discovery elapsed milliseconds and each cache build elapsed milliseconds | Enables actual before/after measurement on PS5 without default log churn |
| Native ownership refresh (\`src/stack.rs\`) | Every 2-second ShadowMount snapshot rechecked all remote games for each native title | Compute a set of remote image/folder owners once per snapshot | Reduces repeated inventory comparisons |
| Launch watcher pending execs (\`src/launch.rs\`) | Every 10 ms pending-PID poll cloned the entire configured title set even if no PID had resolved | Only materialize the title set when at least one PID resolves | Lower CPU overhead while waiting for executable title identification without changing polling timing |

### Preserved behavior

- Backport lookup still prioritizes the game's own scan root, then configured roots, then \`/data/homebrew\`; an exact title directory wins over suffix directories within a root.
- An unstable copied package remains visible, existing errors keep their retry semantics, and a new scan picks up mount changes.
- Static HTTP bodies still use \`Cache-Control: no-cache\`; API responses retain \`no-store\`. The AppCache hash and version invalidation remain unchanged.
- Native FPKGs retain source precedence and never disappear merely because the image service is unavailable. Installed title delegation and duplicate-path ordering are retained.
- Launching a game keeps the existing kqueue sandbox watcher, unionfs ownership checks, and service readiness gate.

## Bottlenecks identified but intentionally not changed

1. **Actual game launch:** \`src/launch.rs\` uses the sandbox vnode event, bounded 1 ms \`common/lib\` readiness polling and NOTE_EXEC fallback. Adjusting these without timing can introduce missed overlays, launch failures, or stale mounts. Existing \`[launch <ms>]\` log lines report the mount and readiness path; measure them first.
2. **Resident dependency startup:** \`src/stack.rs\` waits for ShadowMount's no-automount sentinel in 250 ms increments and verifies service readiness every 500 ms. Reducing the waits is unsafe until there are boot measurements showing excess slack, not genuine dependency latency.
3. **Cache sizing:** \`cache::total_size()\` recursively sums cache files after scan passes, and \`/api/sizes/\` recursively walks game/save/DLC trees when requested. This can dominate I/O for very large caches. Consider a validated metadata cache with explicit invalidation and a periodic full reconciliation, not a guess-based cached count.
4. **First cache build:** \`src/cache.rs::build_inner()\` opens/decodes PKG metadata, extracts incompatible blocks and validates free space before writes. Profiling must separate extraction time from installation and game launch. Changes to packing/layout compatibility need unit fixtures and PS5 tests.
5. **Native AMPR read performance:** v0.5.1 is an upstream binary. Its chunk scheduling/read-batch optimizations cannot be tuned from TITAN CORE source without separately changing the upstream library. FPKG pkg mode remains experimental; storage device and archive layout affect observed loading.
6. **Status polling:** \`/api/state\` scans the final 64 KiB of the log, reads mount capacities and merges native/ShadowMount inventory each request. A further short-lived response cache might reduce CPU, but must not delay running-game status, errors or installation progress.
7. **Installer throughput:** \`src/installer_io.rs\` copies in 1 MiB blocks and verifies the complete source and copy, with frequent destination checks. Removing verification or reducing safety checks to gain speed risks corrupted or misplaced packages and is deliberately out of scope.
8. **UI transfer size:** \`src/ui.html\` is a large (~1 MB) embedded page. Eliminating server-side re-rendering helps but does not shrink the network transfer. Compression/code splitting needs explicit PS5 browser and AppCache testing.
9. **Decoder:** \`crates/titan-core-pkg\` uses release \`opt-level = 3\` for hot decoding and whole-program LTO. Algorithm changes need representative PKG fixture benchmarks; increasing compiler flags alone is not evidence of improvement.

## Validation requirements

GitHub Actions must pass both the FreeBSD \`cargo check --workspace --features unified-stack --locked\` job (warnings denied) and the two-stage PS5 ELF build, alongside pinned Kstuff/ShadowMount/AMPR/DLC/PlayGo checks and UI JavaScript syntax validation. Existing Rust regression tests and new backport, mount-classification and static-response tests should be run in a compatible host test harness where available.

**PS5 hardware measurement remains pending.** Use the same original PS5/firmware, storage devices, games and cache state to compare release v1.2 vs the experimental build. Repeat at least five times per condition and log median/p95:

- Cold start: injection to web interface responding, to full service readiness, and to first complete game inventory.
- Warm start: reopening the dashboard and the elapsed time for \`/api/state\` with 10, 50 and 100+ game entries.
- Native title discovery: initial scan and subsequent scan with unchanged files, especially with backports and slow USB drives.
- First launch: cache missing, separate initial extraction/indexing time from the game's launch screen.
- Warm launch: cache already fresh, from selecting the tile to first game frame. Compare repeated launches with no title/drive changes.
- Exit and rest mode: no missed fakelib mounts, status regressions or stale app0 paths; verify save/load in multiple games.

Enable TITAN CORE debug logging for \`perf: native discovery\` and \`perf: cache build\` messages, and inspect existing \`[launch ...]\` timings and \`titan-core.log\`. Report actual timings before claiming percentage improvements or merging this branch.

A small host-side probe is included: run \`python3 review/bench-web.py http://PS5-IP:8097\` on a PC in the same network. It records median/p95 wall-clock latency and bytes read for the dashboard HTML and JSON state endpoints. **This probes web/API responsiveness, not game launch-to-first-frame latency.**

## Release policy

This branch does not tag or publish a new release and does not edit the stable v1.2 assets. Retain Fast Kstuff as the production default until a new owner-supplied ELF is explicitly approved. Promote optimizations into \`main\` only after compiled artifact validation and hardware acceptance of game startup, USB replug, installation, saves and rest-mode behavior.
