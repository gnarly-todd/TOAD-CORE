# Historical source snapshots (not active payloads)

These archives are retained for upstream attribution, license compliance, and reference.
They belong to the **previous** kstuff-lite v1.11 and ShadowMountPlus 1.7beta10 bundle.
They are **not** the exact matching source for the newer user-supplied ELF binaries.

**Do not extract or rebuild these archives into the active payload filenames.**
The unified build embeds only the root-level files:
- `vendor/unified/kstuff1.13-fpkg-dr-test8-rest-fast.elf`
- `vendor/unified/shadowmountplus.elf`

The build's `review/verify-unified-payloads.py` script pins both filenames, contents,
runtime paths, and manifest entries. On a mismatch it fails before compilation.
Historical archives remain untouched by the build.
