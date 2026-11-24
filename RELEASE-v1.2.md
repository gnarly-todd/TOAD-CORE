# TITAN CORE v1.2 (AMPR-EMU-NG 0.5.1)

Stable public release. The exact AMPR runtime matches the SPRX supplied for this update, as confirmed against the official quer3q release.

## Changes

- Updated the bundled official, non-debug AMPR-EMU-NG library from 0.5.0 to **0.5.1**.
- AMPR upstream reports faster read batching and an expanded disk-read pool, plus fixes for save icon handling, shared save mounts, and packed/on-disk directory listings. These are upstream-reported changes; physical-console performance and game compatibility have not been independently measured for this combined build.
- The supplied 706,976-byte SPRX and the official release binary have the same SHA-256: 7ca61afc444f0b44d2f005506dde16cb43e74dae4b1e045323b291b8db1c3d5d.
- Updated version tracking, build checksum, and upstream source provenance.

## Included components

| Component | Version | Integration |
|---|---|---|
| AMPR-EMU-NG | **0.5.1** | Official upstream SPRX |
| Kstuff Lite / FPKG | 1.13-fpkg-dr-test8-rest-fast | Resident ELF |
| ShadowMountPlus | 1.7beta12 | Resident ELF |
| Kraken Mount | 0.3.1 | Partial source integration; not a separate executable |

## Installation and caveats

Download TITAN-CORE.elf and verify it using SHA256SUMS. Reboot to clear prior resident services; use your normal jailbreak and ELF loader, then visit http://<PS5-IP>:8097. Application version is 1.2.0 and the process identity remains titan-core.elf.

The current installer and original-path package support are preserved. Custom AMPR libraries installed manually are not overwritten unless ownership identifies a previous bundled copy. Debug FPKG pkg support remains experimental. The build checks are not a hardware acceptance test and do not guarantee every game, save or firmware works. No exploit or encrypted retail PKG support is provided.

Credits: devchmod (original TITAN CORE), quer3q (AMPR/Kraken), EchoStretch and relevant kstuff contributors, drakmor (ShadowMountPlus), and other upstream authors. See the repository README/CREDITS.md and upstream releases.
