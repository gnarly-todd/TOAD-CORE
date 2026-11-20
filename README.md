<p align="center"><img src="ps5/icon0.png" width="96" alt="TITAN CORE"></p>
<h1 align="center">TITAN CORE</h1>
<p align="center">A PS5 game workspace with a native FPKG engine, external-drive installer and live diagnostics.</p>

## Version 1.0

Internal installed PKGs stay in their system path. External installation keeps your archive original, creates its game folder automatically, and shows the installed external package size separately from internal cache. Connecting a USB disk does not automatically mount raw installer PKGs. The current bundled runtime is unmodified **ampr-emu-ng 0.4.1**; the earlier PLAY NO LIMIT custom notice is retained only in historical source archives.

**Known limit:** full external package size in PS5 System storage is not established. The v1.0 version number does not imply universal firmware/game compatibility or a new full hardware acceptance pass. See [release validation](docs/V1_0_VALIDATION.md).

يبقى التثبيت الداخلي في مساره الأصلي. التثبيت الخارجي يحتفظ بملف الأرشيف ويعرض حجم اللعبة والذاكرة المؤقتة بشكل منفصل. ملفات PKG الأصلية على USB لا تعمل mount تلقائياً. ظهور الحجم الكامل للخارجية داخل تخزين نظام PS5 ما زال غير محقق.

## Download the updated TOAD-CORE build

This fork bundles `kstuff-fpkg-1.13-dr-test6.elf` and `shadowmountplus.elf` (1.7beta12). The upstream TITAN CORE v1.0 release ELF **does not contain** these replacements.

Open [TOAD-CORE build workflow](https://github.com/gnarly-todd/TOAD-CORE/actions/workflows/build.yml), select a **successful** run for `main`, and download the `payload` artifact. The generated `TITAN-CORE.elf` is the fork's build output. No PS5 console acceptance test has been performed for the supplied binaries.

Close games and reboot the console before switching resident payload versions. Then use your existing jailbroken console/elfldr workflow to load the updated ELF and access `http://<PS5-IP>:8097`. The source tree does not include a jailbreak exploit.

## ⚙️ قبل التشغيل / Before loading TITAN CORE

**TOAD-CORE fork payload update:** This fork uses the owner's test ELFs, rather than the upstream TITAN CORE v1.0 bundle. The bundled binaries are checksum-verified at build time but have **not** been validated on PS5 hardware. A reboot is required before loading the updated TITAN CORE ELF if older standalone/bundled kstuff or ShadowMount processes are resident. Source/binary provenance: [vendor/unified/PROVENANCE.md](vendor/unified/PROVENANCE.md).

**TITAN CORE يتضمن kstuff-fpkg وShadowMountPlus ويبدأ تشغيلهما تلقائياً. / This TOAD-CORE fork bundles user-supplied kstuff-fpkg 1.13-dr-test6 and ShadowMountPlus 1.7beta12 and starts them automatically.**

### بالعربية

- إذا كنت كتستعمل **kstuff أو ShadowMount كـpayloads مستقلّة**، عطّل تشغيلهما التلقائي أو احذف إدخالاتهما من لائحة **autoload / Payload Manager** الخاصة بهاد الإعداد. شغّل **TITAN CORE فقط** عوض تشغيل الحزم الثلاث كل واحدة بوحدها.
- المقصود هو إزالة **التشغيل المنفصل** لهذين المكوّنين؛ لا تحذف ملفات الجيلبريك أو elfldr أو ملفات الألعاب، ولا تحذف المكونات المضمّنة داخل TITAN CORE.
- إذا كانا مشغّلين من قبل، أغلق الألعاب وأعد تشغيل الجهاز لبدء جلسة جديدة، ثم فعّل الجيلبريك وelfldr بالطريقة المعتادة وحمّل ELF ديال TITAN CORE مرة واحدة.
- TITAN CORE لا يتضمن استغلال الجيلبريك نفسه؛ ما زلت تحتاج إلى الجيلبريك وأداة تحميل ELF. يمكن مراجعة حالة الخدمات في Dashboard وDiagnostics.

### English

- If you previously loaded **standalone kstuff or ShadowMount payloads**, disable their automatic loading or remove their entries from your **autoload / Payload Manager** list for this setup. Load **TITAN CORE only**, instead of loading all three separately.
- Remove the **separate launch entries**; you do not need to delete jailbreak, elfldr or game files. Keep TITAN CORE's bundled components intact.
- If standalone services are already running, close games and restart the console for a fresh session. Apply your normal jailbreak, make elfldr available, then load the TITAN CORE ELF once.
- TITAN CORE does not include a jailbreak exploit. You still need your usual jailbreak and ELF loader. Check service status in Dashboard and Diagnostics.

Developed and maintained by **[devchmod](https://github.com/devchmod)** following six months of independent research, TITAN CORE brings game browsing, USB installation and diagnostics into one PS5 workspace. It integrates the native mounting workflow from Kraken Mount, user-supplied kstuff-fpkg 1.13-dr-test6 and ShadowMountPlus 1.7beta12, with TITAN CORE interface, installer and integration work. It targets jailbroken PS5 consoles with elfldr. The development console uses firmware 13.60; wider hardware compatibility has not been established. This project does not include a console exploit.

[Releases](https://github.com/devchmod/TITAN-CORE/releases) · [Source](https://github.com/devchmod/TITAN-CORE) · [Credits](CREDITS.md)

## Updates and installed-game management

The header displays update status beside the language selector. It checks up to 100 recent public releases from the official repository, compares semantic versions (including beta versions), ignores drafts/unpublished entries and requires an uploaded ELF with an official repository download URL. Automatic checks run after console identification and at most once per 30 minutes per page session, with a manual retry option. Network/rate-limit failures remain distinct from a successful check that finds no newer release. Downloads remain a user action; no payload is automatically loaded or replaced. Checking requires browser access to GitHub. Current public releases may lag local test builds.

Manage installed game opens TITAN CORE's own panel. It retains package/cache details, queues a library rescan, keeps the existing confirmed cache-rebuild action, and navigates to Diagnostics. Installed-game management no longer opens the ShadowMount web UI. Uninstallation guidance points to PS5 System Settings; no new game-deletion API was introduced. Image-only tools retain their separate existing behavior. Rebuild is disabled while a game is running. A successful ready-path mount/ownership check now clears a stale earlier error.

## Prepare supported installed PKGs in their original system location

With **Prepare installed PKGs in place automatically** enabled, TITAN CORE discovers eligible debug FPKGs under `/user/app/<TITLE_ID>/app.pkg` and builds its native launch cache without moving the package to homebrew. Read access is prepared for the game's read-only sandbox mount; original permissions are recorded first. The package remains a single physical file in the system location.

Receipt-proven internal packages moved by older TITAN CORE builds can be returned to their original installed path. The package title and recorded size must match. Pre-existing different `app.pkg` files are refused. Restoration journals and exclusively reserves the absent destination, verifies that reservation, then uses rename with a verified `/user/data` alias when required; there is no full-copy fallback. Do not install, update or uninstall the same title concurrently: the SDK does not expose an atomic no-replace rename primitive. A missing title directory can be recreated from matching cached registration metadata. Unsupported restoration leaves the old managed source intact. Manually relocated packages without a valid receipt are not moved automatically.

The verified rename-based restoration replaces the earlier unsupported hard-link approach and preserves the large metadata-snapshot fix. Cache still occupies additional storage. The maintainer confirmed original-path launch and full internal System storage sizes on the development console with 0.5.7; later installer/UI changes have local tests and require their documented hardware checks. This uses the native cache/mount engine for supported debug FPKGs.

## Kraken Mount v0.3 integration (partial)

This TOAD-CORE fork now bundles upstream AMPR 0.4.1, the DLC emulator 0.3, and PlayGo stub 0.6. The native cache code incorporates Kraken v0.3's normal-game save-module exclusion, mandatory AMPR for PKG sources, conditional emulator injection, and preservation of newer game-supplied AMPR libraries. **This is not yet a full merge of every Kraken v0.3 feature**. Specifically, v0.3 scan stability/removal, title sound/localization, and USB unmount still require separate integration and console validation. The custom TOAD installer, original-path installations, and ShadowMount coordination remain unchanged.

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

The historical bundled runtime introduced a custom **PLAY NO LIMIT** notice; current TOAD-CORE builds use upstream AMPR 0.4.1 instead. Settings uses **Game folders / Dossiers des jeux** with corrected system/external paths and an automatic-discovery placeholder. Close games before upgrading; the new notice applies on the next game start with the bundled runtime. Native installer/game behavior remains the 0.6.1 baseline. See [validation](docs/PLAY_NO_LIMIT_VALIDATION.md).

## Preserve your PKG archive — 0.6.3-beta.1

Installing to an external disk keeps the original PKG exactly where it was, even if the source and destination are the same disk. TITAN creates a separate verified copy in `TITAN-CORE/Games/<TITLE_ID>/`. This needs additional space for the complete package plus working reserve; insufficient space stops installation without moving or deleting the original. Existing internal `/user/app` games stay in place. See [validation](docs/PRESERVE_EXTERNAL_SOURCE_VALIDATION.md).

يبقى ملف PKG الأصلي في مكانه بعد التثبيت، حتى على نفس القرص الخارجي. ينشئ TITAN نسخة للألعاب داخل `TITAN-CORE/Games/<TITLE_ID>/`؛ يجب توفير مساحة للنسخة الكاملة.

## External PKGs require installation — 0.6.4-beta.1

Connecting an external drive no longer auto-registers or mounts its download PKGs. Browse those originals in Installer and choose a destination. Only receipt-proven TITAN external installations (`TITAN-CORE/Games/<TITLE_ID>/<TITLE_ID>.pkg`) or recognized system-installed PKGs enter native game preparation. A folder with that name alone does not prove installation. Originals stay archived and are excluded from playable-library duplicates, even with custom scan folders. Internal PKGs, external extracted-game folders and image handling keep their existing behavior. Existing archives are not deleted. See [validation](docs/EXTERNAL_INSTALL_ONLY_VALIDATION.md).

ملفات PKG الأصلية في القرص الخارجي تظهر في المثبّت فقط، ولا يتم تجهيزها أو عمل mount لها بمجرد توصيل القرص. بعد التثبيت يعتمد TITAN نسخة اللعبة في مسار التثبيت، مع الاحتفاظ بملف الأرشيف الأصلي.

## External installed label and measured size — 0.6.5-beta.1

Receipt-proven external installations now show **PKG · External installed** with the actual drive, package size and internal cache listed separately. Original external installer archives remain Installer-only. System-installed/internal package behavior is unchanged. This display update does not change PS5 System storage accounting; full external accounting remains unverified. See [validation and investigation](docs/EXTERNAL_STATUS_VALIDATION.md).

تظهر النسخة الخارجية المعتمدة باسم «مثبّتة خارجياً»، مع القرص والحجم الحقيقي للحزمة وحجم الذاكرة المؤقتة الداخلية بشكل منفصل. حساب التخزين داخل نظام PS5 لم يتغيّر بهذه النسخة.
