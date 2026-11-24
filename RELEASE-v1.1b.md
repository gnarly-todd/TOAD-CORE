# TITAN CORE v1.1b (Fast Kstuff)

**Fast-variant pre-release** following the stable TITAN CORE v1.1. The previously published v1.1 release and its assets remain unchanged.

## What's different

- Kstuff Lite / FPKG has been replaced with the **test8-rest-fast** ELF: `kstuff1.13-fpkg-dr-test8-rest-fast.elf`.
- The selected file is the distinct Fast/PPR build, not the standard test8-rest variant.
- Uploaded source ELF SHA-256: `221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc`.
- The kstuff internal version text remains `1.13-fpkg-dr-test8-rest`; the `-fast` distinction is verified by the actual binary hash, file name and Fast/PPR marker.
- The app version is `1.1.1-beta.1` for correct forward SemVer ordering. GitHub release tag: **v1.1b**.
- Active process identity remains `titan-core.elf`.

## Unchanged components

| Component | Version |
|---|---|
| ShadowMountPlus | `1.7beta12` |
| AMPR-EMU-NG | `0.5.0` |
| Kraken Mount | `0.3.1` (partial source integration) |

No separate Kraken ELF is bundled. TITAN CORE's original-path package handling, installer, and ShadowMount coordination remain unchanged.

## Install and limitations

Download `TITAN-CORE.elf` and its `SHA256SUMS` checksum from the assets. Reboot to clear previously running kstuff and ShadowMount processes before loading with your usual PS5 jailbreak and ELF loader. Open `http://<PS5-IP>:8097` afterward.

Automated integrity/build checks do not prove physical-console performance, rest-mode stability or compatibility. This Fast/PPR build has **not** been independently benchmarked or hardware-tested. It does not provide a jailbreak or retail encrypted PKG compatibility. The original TITAN CORE author [devchmod](https://github.com/devchmod/TITAN-CORE) and upstream contributors (including quer3q, EchoStretch, drakmor) retain attribution.
