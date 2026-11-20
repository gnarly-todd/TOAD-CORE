# Development, thanks and licenses

## TITAN CORE — devchmod

TITAN CORE is maintained by **[devchmod](https://github.com/devchmod)**, following six months of independent research. Project work includes the TITAN CORE interface and branding, multilingual navigation, USB FPKG installer, installed-package preparation and legacy import recovery, diagnostics and reporting, and integration of the resident services.

**Installed-PKG launch handling is a TITAN CORE addition:** the earlier workflow brought supported Debug Settings/Overdrive installs into a playable homebrew library without retaining another complete PKG, with successful console use reported by the maintainer on firmware 13.60. Current versions keep eligible packages at their original system path and recover receipt-proven earlier imports, while retaining the native cache and launch engine. The maintainer confirmed original-path launch and full System storage sizes on the development console with 0.5.7-beta.1; see docs/USER_HARDWARE_CONFIRMATION.md. Native cache uses additional space.

Development also addresses package discovery and duplicate library filters. Build checks and documented tests are included in this repository; see [validation](docs/VALIDATION.md) for their scope.

## Thanks to the upstream developers

Thank you to **quer3q**, **EchoStretch**, **drakmor**, the **PS5 payload SDK contributors**, and the wider homebrew community for sharing their code, tools and research. Their contributions support the components listed below. TITAN CORE development and integration are described above; each upstream project retains credit for its own code.

## Included components

| Component | Upstream | Role |
|---|---|---|
| Kraken Mount | [quer3q/kraken-mount](https://github.com/quer3q/kraken-mount) | Native FPKG/folder mounting, cache, registration and launch code used and adapted in TITAN CORE |
| ampr-emu-ng 0.4.0 | [quer3q/ampr-emu-ng](https://github.com/quer3q/ampr-emu-ng/tree/v0.4.0) | Packed-game runtime; custom launch notice `PLAY NO LIMIT` (component UI label `chmod v0.1`) |
| kstuff-fpkg 1.13-dr-test6 | [EchoStretch/kstuff-lite](https://github.com/EchoStretch/kstuff-lite) (historical project reference) | User-supplied resident ELF; exact binary/source revision not verified |
| ShadowMountPlus 1.7beta12 | [drakmor/ShadowMountPlus](https://github.com/drakmor/ShadowMountPlus) | User-supplied resident ELF; exact binary/source revision not verified |
| PS5 Dump Forge | [quer3q/ps5-dump-forge](https://github.com/quer3q/ps5-dump-forge) | Toolchain/workflow references and attributed sparse-format reader reference |
| ps5-payload-sdk | [ps5-payload-dev/sdk](https://github.com/ps5-payload-dev/sdk) | Payload build toolchain and PS5 libraries |

## Source and notices

- Root license: GPL-3.0-or-later; see [LICENSE](LICENSE).
- Existing dependency notices: [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).
- Current test service binary versions and hashes, retained third-party dependency source snapshots, component licenses and notices: [vendor/unified](vendor/unified). Exact matching source for the new test ELFs has not been verified.
- Original AMPR 0.4.0 binary/source and custom display-only runtime source: [vendor/ampr-emu-ng](vendor/ampr-emu-ng), including [branding explanation](vendor/ampr-emu-ng/TITAN_BRANDING.md).
- Sparse-format port attribution and exact source reference: [vendor/format-reference](vendor/format-reference).
- TITAN CORE artwork provenance: [docs/branding/PROVENANCE.md](docs/branding/PROVENANCE.md).

The current custom AMPR source changes only the launch-notice text to PLAY NO LIMIT. Compatible filenames, exported runtime identity and underlying engine version remain upstream 0.4.0. Upstream credits in source archives are preserved. Additional third-party code retains its own license terms; consult the included notices/source for each component.

## Reader and decoder source references

The renamed `crates/titan-core-pkg/src/layout.rs` is the existing layout reader previously named `kraken.rs`. It ports the upstream `VF/kraken_image.rs:728-1036` describe logic and `FP/mount.rs:35-283` block/layout logic. Its sparse handling references `vendor/ps5upload-fpkg/src/kraken_image.rs` (GPL-3.0-or-later). The decoder ports the Kraken block decoder vendored by ps5-dump-forge: `kraken.rs` (bit reader, arrays and LZ), `kraken/huff.rs` (Huffman helpers) and `kraken_image.rs::decode_described`. Neutral local names do not change the compression format or its authorship. AMPR codec-2 index encoding follows the upstream `ng/KRAKEN.md` contract.

The superseded service binaries and their old release/source metadata were removed from the current TOAD-CORE tree. Exact corresponding source for the user's replacement ELF builds remains unverified, and the upstream project authors retain their respective credits and licenses.
