# TITAN CORE v1.1

First public release of this TITAN CORE fork. Original TITAN CORE author and third-party contributors retain attribution.

## Included components

| Component | Version | How included |
|---|---|---|
| ShadowMountPlus | `1.7beta12` | Bundled resident ELF |
| AMPR-EMU-NG | `0.5.0` | Official upstream SPRX |
| Kstuff Lite / FPKG | `1.13-fpkg-dr-test8-rest` | User-supplied resident ELF |
| Kraken Mount | `0.3.1` | Selective source integration, not a separate executable |

The Kraken-derived changes include nested game discovery, buffered metadata copying, `playgo_off` per-title exclusions and batch game registration on supported firmware. TITAN CORE preserves its independent installer, in-place debug PKG support and ShadowMount coordination.

## How to use

Download **TITAN-CORE.elf** below, then verify it using **SHA256SUMS**. Close running games and reboot to clear prior resident kstuff/ShadowMount versions. Apply your existing PS5 jailbreak, load the ELF through your usual loader and visit `http://<PS5-IP>:8097`.

The embedded application version is **1.1.0**. The payload requests **titan-core.elf** as its active process name.

## Validation and limitations

The bundled binaries and the output checksum are verified by the build; host and PS5 compilation checks are automated. **No complete physical-console acceptance test was performed for this release.** The development target is firmware 13.60, but use depends on the active jailbreak, services and title. Native A53 code execution is not confirmed. This is for homebrew/debug FPKG workflows, not retail encrypted PKGs, and not all upstream Kraken Mount UI features are integrated.

**Credits:** [devchmod/TITAN-CORE](https://github.com/devchmod/TITAN-CORE), [quer3q](https://github.com/quer3q), [EchoStretch](https://github.com/EchoStretch), [drakmor](https://github.com/drakmor), and all upstream contributors. [Repository README](https://github.com/gnarly-todd/TOAD-CORE#included-component-versions) · [Credits and licenses](https://github.com/gnarly-todd/TOAD-CORE/blob/main/CREDITS.md)
