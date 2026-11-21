# AMPR runtime provenance (TITAN CORE)

The active `libSceAmpr.sprx` is the **unmodified upstream ampr-emu-ng v0.5.0 stable release** from [quer3q/ampr-emu-ng](https://github.com/quer3q/ampr-emu-ng/releases/tag/v0.5.0).

- Active binary SHA-256: `7379e49953d8f14f26b45ff06a863ecea56aa90a60ba3d5a375f61c52618a174`
- Pinned upstream source commit: `d20210e224871864bee0888193c7c1062890639c`
- Complete source snapshot: `upstream-v0.5.0.tar.gz` (verified tag, git archive)
- Validation: the production PS5 build verifies `vendor/ampr-emu-ng/SHA256SUMS` before compiling, and the GitHub source check verifies it too.

The historical v0.4.0 AMPR binary, custom PLAY NO LIMIT source and owner-provided older v0.4.1 release are not active in this build. Their history and notices are preserved without misrepresenting the active release. No custom branding was injected into v0.5.0.

TITAN CORE replaces its own previously bundled AMPR library on PS5 when the on-console ownership marker confirms that the file came from the bundle. It does not overwrite manually installed custom AMPR libraries. Console gameplay compatibility must still be verified.
