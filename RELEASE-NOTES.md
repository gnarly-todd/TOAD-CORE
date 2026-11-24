# TITAN CORE v1.1b - Fast Kstuff pre-release

This optional pre-release replaces only the checksum-pinned kstuff test8-rest ELF with the user-provided test8-rest-fast variant. Runtime process name stays titan-core.elf. App SemVer: 1.1.1-beta.1.

The internal version string remains 1.13-fpkg-dr-test8-rest but the selected binary and SHA-256 identify the Fast/PPR variant. Speed and console stability have not been benchmarked.

---

# TITAN CORE v1.1 (first public release of this fork)

Release: https://github.com/gnarly-todd/TOAD-CORE/releases/tag/v1.1

This fork's first published release bundles user-supplied Kstuff Lite / FPKG `1.13-fpkg-dr-test8-rest`, ShadowMountPlus `1.7beta12`, official AMPR-EMU-NG `0.5.0`, and selected Kraken Mount `0.3.1` integration.

The GitHub release tag is `v1.1`, and the application reports version `1.1.0`. The uploaded ELF remains `TITAN-CORE.elf` and requests `titan-core.elf` as its process identity. Repository naming remains `gnarly-todd/TOAD-CORE`.

Kraken Mount is **not** bundled as a separate ELF. Selective source-port changes include nested game discovery, buffered metadata staging, a per-title `playgo_off` setting and firmware-12+ batch registration. TITAN's original-path PKG workflow, installer and ShadowMount ownership rules are preserved.

**Validation:** GitHub Actions verifies all resident binary hashes, bundled AMPR and emulator hashes, and optimized PS5 ELF compilation. This does not constitute a hardware compatibility guarantee for every game or PS5 firmware. Restart the PS5 to clear old resident service versions before injecting the updated ELF.

**Attribution:** TITAN CORE originates from [devchmod](https://github.com/devchmod/TITAN-CORE). Kraken Mount / AMPR are attributed to quer3q; kstuff to EchoStretch and relevant contributors; ShadowMountPlus to drakmor. See `CREDITS.md` and third-party licenses.

---

## Previous upstream release (historical)

# TITAN CORE v1.0

## ⬇️ تحميل ELF / Download ELF

**[تحميل ملف التشغيل مباشرة — Download TITAN-CORE-v1.0.elf](https://github.com/devchmod/TITAN-CORE/releases/download/v1.0.0/TITAN-CORE-v1.0.elf)**

بالعربية: حمّل ملف **TITAN-CORE-v1.0.elf** من الرابط أعلاه أو من **Assets** بالأسفل. أغلق الألعاب ثم أرسله باستعمال أداة elfldr/payload المعتادة. افتح `http://<PS5-IP>:8097`. ملفات Source code للمطورين، وليست ملف التشغيل.

English: download **TITAN-CORE-v1.0.elf** above or expand **Assets** below. Close games, load the ELF through your existing elfldr/payload workflow, then open `http://<PS5-IP>:8097`. Source-code archives are for developers. Verify **SHA256SUMS**.

## What's included / الجديد

- Supported installed internal debug PKGs stay at `/user/app/<TITLE_ID>/app.pkg`, using the existing native cache/launch pipeline.
- Install from connected USB/extended disks to Internal or a selected external destination. TITAN creates `TITAN-CORE/Games/<TITLE_ID>/` automatically.
- Original installer PKGs stay archived, even on the same external disk. Copies are byte-verified; destination space checks include the complete package.
- Raw external PKGs remain Installer-only until installed; plugging a disk in does not auto-mount archive originals.
- Valid external installations show **External installed**, the actual drive/package size and internal cache separately.
- The fork now bundles kstuff 1.13 FPKG DR Test 6 and ShadowMountPlus 1.7beta12; original v1.0 included the PLAY NO LIMIT runtime notice, multilingual/RTL interface, settings, diagnostics/klog and update checks.

الداخلية تبقى في مسار النظام. الخارجية تُثبّت داخل مجلد TITAN مع الاحتفاظ بملف الأرشيف الأصلي. ملفات الأرشيف لا تعمل mount تلقائياً. تعرض الواجهة حالة التثبيت الخارجي والحجم والقرص والذاكرة المؤقتة بشكل منفصل.

## Before loading / قبل التشغيل

TITAN bundles the resident services. Disable separate kstuff/ShadowMount autoload entries for this setup; keep jailbreak/elfldr, game files and TITAN's bundled components. If standalone copies are already running, close games and begin a fresh console session using your normal jailbreak/elfldr workflow.

يتضمن TITAN الخدمات المدمجة. عطّل التشغيل المنفصل لـkstuff وShadowMount في autoload؛ لا تحذف ملفات الجيلبريك أو elfldr أو الألعاب. إذا كانت الخدمات المنفصلة تعمل، ابدأ جلسة جديدة بالطريقة المعتادة قبل تحميل TITAN.

## Validation and limits

104 synthetic Rust tests and both optimized unified ELF build stages passed. This release retains the0.6.5 behavior; the maintainer has reported successful use on the development console, but no new agent-run v1.0 hardware playtest or universal firmware compatibility pass was performed.

**Full external package size in PS5 System storage remains unverified and is not changed by this release.** TITAN displays the measured external package and internal cache separately. Cache needs internal space. Generic retail/encrypted PKGs, power-loss resume and update/DLC merging are not added. Extra console locales retain documented partial translation/English fallback.

**ظهور الحجم الكامل للخارجية داخل Stockage ديال PS5 ما زال غير محقق.** النسخة تعرض الحجم الصحيح داخل TITAN؛ لا تعني v1.0 توافق كل الألعاب أو جميع الأجهزة.

Full source snapshot, validation and checksums are attached. See [README](https://github.com/devchmod/TITAN-CORE#readme) and [credits/licenses](https://github.com/devchmod/TITAN-CORE/blob/main/CREDITS.md). Thanks to quer3q, EchoStretch, drakmor, SDK contributors and the homebrew community; included upstream licenses/source notices are preserved.
