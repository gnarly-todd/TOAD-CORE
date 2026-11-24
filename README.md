<p align="center"><img src="ps5/icon0.png" width="96" alt="TITAN CORE"></p>
<h1 align="center">TITAN CORE</h1>
<p align="center">A PS5 game workspace with a native FPKG engine, external-drive installer and live diagnostics.</p>

## Version 1.2

**Kstuff pinning:** This and all subsequent builds use the user-supplied Fast Kstuff file kstuff1.13-fpkg-dr-test8-rest-fast.elf, SHA-256 221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc, until you explicitly supply a replacement. Production build checks reject any different kstuff binary, including the non-fast test8-rest variant.

Internal installed PKGs stay in their system path. External installation keeps your archive original, creates its game folder automatically, and shows the installed external package size separately from internal cache. Connecting a USB disk does not automatically mount raw installer PKGs. The current bundled AMPR runtime version is listed in the table below; the earlier PLAY NO LIMIT custom notice is retained only in historical source archives.

**Known limit:** full external package size in PS5 System storage is not established. The v1.2 version number does not imply universal firmware/game compatibility or a new full hardware acceptance pass. See [release validation](docs/V1_0_VALIDATION.md).


<!-- BEGIN GENERATED TITAN COMPONENTS: review/sync-component-readme.py -->
## Included component versions

| Component | Version | Integration |
|---|---|---|
| **[ShadowMountPlus](https://github.com/drakmor/ShadowMountPlus)** | `1.7beta12` | Bundled resident ELF |
| **[AMPR-EMU-NG](https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.1)** | `0.5.1` | Bundled official upstream SPRX |
| **[Kstuff Lite / FPKG](https://github.com/EchoStretch/kstuff-lite)** | `1.13-fpkg-dr-test8-rest-fast` | Bundled resident ELF (user-supplied) |
| **[Kraken Mount](https://github.com/quer3q/kraken-mount/releases/tag/v0.3.1)** | `0.3.1` | Selected upstream source changes; not a separate executable |

**Kraken Mount distinction:** TITAN CORE incorporates selected features from Kraken Mount v0.3.1 source, including one-level nested game discovery, faster buffered metadata copying, per-title PlayGo stub exclusions via `playgo_off`, and batch registration on supported firmware. It does **not** bundle or launch a separate Kraken Mount executable or include every upstream feature. TITAN CORE keeps its own installer, original-path package handling, and ShadowMount coordination.

**Process identity:** `titan-core.elf` in the PS5 Active Processes list, as requested by the startup shim; actual display requires console confirmation. The downloadable executable remains `TITAN-CORE.elf` in the GitHub Actions `payload` artifact.

**Version scope:** These are the currently pinned *repository/build* versions, **not** promises that no newer upstream releases exist. They describe the binaries and source-port changes configured for a production build. Only a successful build of the corresponding commit confirms a finished ELF; actual hardware compatibility requires PS5 testing.

*This section is generated automatically from the checked-in component manifest, runtime version, Kraken source provenance, and PS5 process-name code. Updates are synchronized by GitHub Actions and the component import workflows. Do not edit this section manually.*
<!-- END GENERATED TITAN COMPONENTS -->

## Download the updated TITAN CORE build

This fork bundles the versioned kstuff and ShadowMountPlus ELF files listed above. The upstream TITAN CORE v1.0 release ELF **does not contain** these replacements.

Open [TITAN CORE build workflow](https://github.com/gnarly-todd/TOAD-CORE/actions/workflows/build.yml), select a **successful** run for `main`, and download the `payload` artifact. The generated `TITAN-CORE.elf` is the fork's downloadable ELF. It explicitly names its main process `titan-core.elf` at startup, independent of upload filename, so the Active Processes list can identify it. No PS5 console acceptance test has been performed for the supplied binaries.

Close games and reboot the console before switching resident payload versions. Then use your existing jailbroken console/elfldr workflow to load the updated ELF and access `http://<PS5-IP>:8097`. The source tree does not include a jailbreak exploit.

## Before loading TITAN CORE

**TITAN CORE fork payload update:** This fork uses the owner's test ELFs, rather than the upstream TITAN CORE v1.0 bundle. The bundled binaries are checksum-verified at build time but have **not** been validated on PS5 hardware. A reboot is required before loading the updated TITAN CORE ELF if older standalone/bundled kstuff or ShadowMount processes are resident. Source/binary provenance: [vendor/unified/PROVENANCE.md](vendor/unified/PROVENANCE.md).

**This TITAN CORE fork bundles the user-supplied kstuff and ShadowMountPlus versions shown in the component table and starts them automatically.**

### Loading instructions

- If you previously loaded **standalone kstuff or ShadowMount payloads**, disable their automatic loading or remove their entries from your **autoload / Payload Manager** list for this setup. Load **TITAN CORE only**, instead of loading all three separately.
- Remove the **separate launch entries**; you do not need to delete jailbreak, elfldr or game files. Keep TITAN CORE's bundled components intact.
- If standalone services are already running, close games and restart the console for a fresh session. Apply your normal jailbreak, make elfldr available, then load the TITAN CORE ELF once.
- TITAN CORE does not include a jailbreak exploit. You still need your usual jailbreak and ELF loader. Check service status in Dashboard and Diagnostics.

Developed and maintained by **[devchmod](https://github.com/devchmod)** following six months of independent research, TITAN CORE brings game browsing, USB installation and diagnostics into one PS5 workspace. It integrates the native mounting workflow from Kraken Mount and the user-supplied kstuff and ShadowMountPlus binaries shown above, with TITAN CORE interface, installer and integration work. It targets jailbroken PS5 consoles with elfldr. The development console uses firmware 13.60; wider hardware compatibility has not been established. This project does not include a console exploit.

[Releases](https://github.com/devchmod/TITAN-CORE/releases) · [Source](https://github.com/devchmod/TITAN-CORE) · [Credits](CREDITS.md)

## Updates and installed-game management

The header displays update status beside the language selector. It checks up to 100 recent public releases from the official repository, compares semantic versions (including beta versions), ignores drafts/unpublished entries and requires an uploaded ELF with an official repository download URL. Automatic checks run after console identification and at most once per 30 minutes per page session, with a manual retry option. Network/rate-limit failures remain distinct from a successful check that finds no newer release. Downloads remain a user action; no payload is automatically loaded or replaced. Checking requires browser access to GitHub. Current public releases may lag local test builds.

Manage installed game opens TITAN CORE's own panel. It retains package/cache details, queues a library rescan, keeps the existing confirmed cache-rebuild action, and navigates to Diagnostics. Installed-game management no longer opens the ShadowMount web UI. Uninstallation guidance points to PS5 System Settings; no new game-deletion API was introduced. Image-only tools retain their separate existing behavior. Rebuild is disabled while a game is running. A successful ready-path mount/ownership check now clears a stale earlier error.

## Prepare supported installed PKGs in their original system location

With **Prepare installed PKGs in place automatically** enabled, TITAN CORE discovers eligible debug FPKGs under `/user/app/<TITLE_ID>/app.pkg` and builds its native launch cache without moving the package to homebrew. Read access is prepared for the game's read-only sandbox mount; original permissions are recorded first. The package remains a single physical file in the system location.

Receipt-proven internal packages moved by older TITAN CORE builds can be returned to their original installed path. The package title and recorded size must match. Pre-existing different `app.pkg` files are refused. Restoration journals and exclusively reserves the absent destination, verifies that reservation, then uses rename with a verified `/user/data` alias when required; there is no full-copy fallback. Do not install, update or uninstall the same title concurrently: the SDK does not expose an atomic no-replace rename primitive. A missing title directory can be recreated from matching cached registration metadata. Unsupported restoration leaves the old managed source intact. Manually relocated packages without a valid receipt are not moved automatically.

The verified rename-based restoration replaces the earlier unsupported hard-link approach and preserves the large metadata-snapshot fix. Cache still occupies additional storage. The maintainer confirmed original-path launch and full internal System storage sizes on the development console with 0.5.7; later installer/UI changes have local tests and require their documented hardware checks. This uses the native cache/mount engine for supported debug FPKGs.

## Kraken Mount integration (partial)

This TITAN CORE fork bundles the upstream AMPR version shown above, the DLC emulator 0.3, and PlayGo stub 0.6. The native cache code incorporates Kraken's normal-game save-module exclusion, mandatory AMPR for PKG sources, conditional emulators, and preservation of newer game-supplied AMPR. The current Kraken source port also adopts one-level nested scan folders, faster 1 MiB buffered registration metadata copies, a per-title `playgo_off` setting, resilient configuration-array parsing, and batch home-screen registration when the FW12+ per-title install API is unavailable. **This is a selective source port, not a full merge of the upstream application.** Unported features include Kraken's per-game PlayGo interface control, title sound/localization, auto-removal of missing games, and USB unmount. The TITAN installer, original-path package handling, and ShadowMount coordination remain independently maintained. Full console testing is still required. The custom TITAN installer, original-path installations, and ShadowMount coordination remain unchanged.

For the exact source tags, pinned release hashes and current integration limits, see [upstream integration details](docs/KRAKEN_031_AMPR_051.md). The complete upstream sources are archived under `vendor/kraken-mount/` and `vendor/ampr-emu-ng/`. These are attribution/reference archives, not additional resident payloads.

## Features

- Dashboard and game library for supported debug FPKGs, folders and image sources, including exFAT.
- USB/extended-drive browser, internal or connected external FPKG destination, progress, byte-for-byte verification and cancellation.
- **Original-path preparation for supported Debug Settings/Overdrive installs:** keep `app.pkg` under `/user/app/<TITLE_ID>/`, with verified recovery of earlier internal imports.
- Reported system/payload CPU and memory, storage, live logs, kernel-log viewer and current alerts.
- Browser-local CPU history, read-only reported health checks, JSON/CSV exports and print/Save PDF reports.
- 31 console-language/regional choices, flag cues, Arabic RTL, themes and controller-friendly navigation.

![TITAN CORE diagnostics](docs/screenshots/monitor.png)
*Interface preview with synthetic data. Unsupported sensors remain unavailable on the actual console.*

## Use

1. Close games before loading a new payload.
2. Send the release ELF through your existing elfldr workflow.
3. Open `http://<PS5-IP>:8097` from the console or a browser on your Mac/PC/phone.
4. Browse Library or open Installer to choose a supported PS5 debug FPKG from a mounted USB/extended drive.

Installer copies the PKG to the selected internal or external destination and keeps the original, even on the same external disk. External installation creates TITAN-CORE/Games/<TITLE_ID>/ automatically. Raw external PKGs remain Installer-only until installed. The native engine then builds its cache, registers the game and mounts its content at launch. This is the homebrew/debug-FPKG workflow, not a general retail/encrypted PKG installer. Cache takes extra space and can be substantial for incompatible block layouts. Supported installed packages are prepared at their original system path. Legacy recovery refuses collisions and unavailable same-inode moves, leaving the managed source intact without creating another full package.

Do not disconnect a source drive during installation. Ordinary cancellation/errors clean the transaction's partial copy; power loss may leave a reserved `.part` directory that needs inspection before retrying. Resume and update/DLC merging are not implemented.

## Diagnostics and localization

Metrics are reported by the console, with payload-only fallbacks labeled explicitly. Temperature/GPU/I/O rate are not exposed by this backend and remain unknown. CPU history starts when the page opens; it is not historical console telemetry. Diagnostic checks inspect reported status only and are not hardware stress tests. Reports/history are handled in the browser.

English, French, Arabic, Simplified Chinese and German have full current catalogs. Additional languages currently translate 50 core labels and use English fallback for remaining technical text. See [coverage](Localizations/coverage.json).

## Technology

[![Rust](https://img.shields.io/badge/Rust-native_backend-DEA584?logo=rust)](https://github.com/devchmod/TITAN-CORE/search?l=rust) [![HTML](https://img.shields.io/badge/HTML-bundled_interface-E34F26?logo=html5&logoColor=white)](https://github.com/devchmod/TITAN-CORE/search?l=html) [![C](https://img.shields.io/badge/C-PS5_shims-555555?logo=c)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5/shim) [![Shell](https://img.shields.io/badge/Shell-build_scripts-89E051?logo=gnubash&logoColor=white)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5) [![Assembly](https://img.shields.io/badge/Assembly-startup-6E4C13)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5) [![Docker](https://img.shields.io/badge/Docker-build_environment-2496ED?logo=docker&logoColor=white)](https://github.com/devchmod/TITAN-CORE/blob/main/ps5/Dockerfile) [![Python](https://img.shields.io/badge/Python-tooling-3572A5?logo=python&logoColor=white)](https://github.com/devchmod/TITAN-CORE/tree/main/review)

| Layer | Implementation |
|---|---|
| Console backend | Rust: package discovery, cache, registration, mounting, installer and HTTP service |
| Console integration | C and Assembly: PS5 shims, startup and linking support |
| Browser interface | HTML, CSS and JavaScript: library, settings, localization and diagnostics |
| Build and tooling | Shell, Dockerfile and Python: payload builds, UI embedding and review tools |

GitHub's language percentages measure classified source bytes; they do not measure engineering effort or ownership. See the repository's Languages panel for its current calculation.

## Build

The source uses Rust 1.99.0 and PS5 payload SDK 0.43 in the Docker build. Keep compiler parallelism at two jobs.

**Active process name:** `titan-core.elf`. The PS5 startup shim uses `thr_set_name` on the main thread before Rust starts, the same mechanism used by PS5 Payload Manager. This is independent of the GitHub artifact name or Cargo crate name. The internal saved payload path remains `/data/titan-core/titan-core.elf` for compatibility, but loading that stage also sets the visible name. Fully stop an existing TITAN CORE process or reboot before injecting the updated binary, or both processes could be listed.


**Production/research isolation:** The A53 research tools under `review/a53-1360/` are separate and are not linked into the production ELF. The production build does not run those diagnostic scripts or embed the diagnostic ELFs. It continues using the pinned kstuff-fpkg, ShadowMountPlus, AMPR and conditional emulator components. A53 code execution and native A53 FPKG handling on firmware 13.60 have not been demonstrated by these diagnostics.

The downloadable GitHub Actions artifact is named `payload` and contains `TITAN-CORE.elf` and `SHA256SUMS`. If GitHub Actions does not start a run, build locally with the command below; editing `build.yml` alone does not create a new ELF.


```sh
TITAN_UNIFIED_STACK=1 bash ps5/build.sh
```

The build verifies bundled runtime/service hashes and builds both ELF stages. The packages are `titan-core` and `crates/titan-core-pkg`; the Docker toolchain image is `titan-core-ps5` and the output is `target/ps5/titan-core.elf`. Upstream runtime protocol filenames remain compatible. See [architecture](DESIGN.md), [validation](docs/VALIDATION.md) and [credits](CREDITS.md).

## License and credits

GPL-3.0-or-later. TITAN CORE is a derivative/aggregate project, with original authorship and upstream notices retained. The bundled runtime is now the upstream ampr-emu-ng v0.4.1 binary (the previous custom AMPR 0.4.0 PLAY NO LIMIT branding remains historical only). Bundled component source snapshots, modified runtime source and notices are included under `vendor/`.

See [CREDITS.md](CREDITS.md), [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).

## External destination choice

In Installer, choose Internal or a connected USB/extended destination before selecting a PKG. TITAN creates `<drive>/TITAN-CORE/Games/<TITLE_ID>/` automatically. Starting with 0.6.3, installation always keeps the original download and verifies a separate installed copy, including on the same external drive. Allow space for the full copy plus working reserve. Existing installed system PKGs remain in place.

Keep the destination connected while playing. Native cache still occupies internal storage and can be substantial. External source size in TITAN is not a promise of full PS5 System storage accounting. The maintainer reports successful managed external-path use on the development console; broader hardware compatibility and full external PS5 System storage accounting remain unverified. See [validation](docs/EXTERNAL_DESTINATION_VALIDATION.md).

## Corrected Internal destination — 0.6.1-beta.1

The Installer Internal choice now commits the new package to `/user/app/<TITLE_ID>/app.pkg` and uses the existing native system-path preparation/registration workflow. This corrects the 0.6.0 homebrew destination. External destinations remain `<drive>/TITAN-CORE/Games/`. Existing legacy homebrew packages remain discoverable without automatic migration. Download originals are retained for internal copies. See [validation](docs/INTERNAL_DESTINATION_VALIDATION.md); new hardware validation is still required.

## Launch notice and game-folder help — 0.6.2-beta.1

The historical bundled runtime introduced a custom **PLAY NO LIMIT** notice; current TITAN CORE builds use the upstream AMPR release identified in the component table instead. Settings uses **Game folders** with corrected system/external paths and an automatic-discovery placeholder. Close games before upgrading; the new notice applies on the next game start with the bundled runtime. Native installer/game behavior remains the 0.6.1 baseline. See [validation](docs/PLAY_NO_LIMIT_VALIDATION.md).

## Preserve your PKG archive — 0.6.3-beta.1

Installing to an external disk keeps the original PKG exactly where it was, even if the source and destination are the same disk. TITAN creates a separate verified copy in `TITAN-CORE/Games/<TITLE_ID>/`. This needs additional space for the complete package plus working reserve; insufficient space stops installation without moving or deleting the original. Existing internal `/user/app` games stay in place. See [validation](docs/PRESERVE_EXTERNAL_SOURCE_VALIDATION.md).


## External PKGs require installation — 0.6.4-beta.1

Connecting an external drive no longer auto-registers or mounts its download PKGs. Browse those originals in Installer and choose a destination. Only receipt-proven TITAN external installations (`TITAN-CORE/Games/<TITLE_ID>/<TITLE_ID>.pkg`) or recognized system-installed PKGs enter native game preparation. A folder with that name alone does not prove installation. Originals stay archived and are excluded from playable-library duplicates, even with custom scan folders. Internal PKGs, external extracted-game folders and image handling keep their existing behavior. Existing archives are not deleted. See [validation](docs/EXTERNAL_INSTALL_ONLY_VALIDATION.md).


## External installed label and measured size — 0.6.5-beta.1

Receipt-proven external installations now show **PKG · External installed** with the actual drive, package size and internal cache listed separately. Original external installer archives remain Installer-only. System-installed/internal package behavior is unchanged. This display update does not change PS5 System storage accounting; full external accounting remains unverified. See [validation and investigation](docs/EXTERNAL_STATUS_VALIDATION.md).

