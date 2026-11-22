#!/usr/bin/env python3
"""Synchronize README bundled component versions from checked-in production inputs.

The versions shown are pinned local build inputs, not unverified upstream latest
versions. This script only writes its marked README section.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
START = "<!-- BEGIN GENERATED TITAN COMPONENTS: review/sync-component-readme.py -->"
END = "<!-- END GENERATED TITAN COMPONENTS -->"


def checked_hash(path: Path, expected: str) -> None:
    if not path.is_file() or path.is_symlink():
        raise ValueError(f"Missing bundled component: {path.relative_to(ROOT)}")
    with path.open("rb") as f:
        digest = hashlib.file_digest(f, "sha256").hexdigest()
    if digest != expected:
        raise ValueError(f"Bundled component SHA-256 differs: {path.relative_to(ROOT)}")


def render() -> str:
    manifest = json.loads((ROOT / "vendor/unified/MANIFEST.json").read_text())
    components = {c["name"]: c for c in manifest["components"]}
    if "kstuff-fpkg" not in components or "ShadowMountPlus" not in components:
        raise ValueError("Both resident components must appear in the manifest")
    kstuff = components["kstuff-fpkg"]
    shadow = components["ShadowMountPlus"]
    for item in (kstuff, shadow):
        checked_hash(ROOT / "vendor/unified" / item["file"], item["sha256"])
        if not re.fullmatch(r"[A-Za-z0-9_.+-]+", item["embedded_version"].split()[0]):
            raise ValueError(f"Unrecognized component version for {item['name']}")
    checksum_list = dict(
        line.split(maxsplit=1) for line in
        (ROOT / "vendor/unified/SHA256SUMS").read_text().splitlines() if line.strip()
    )
    for item in (kstuff, shadow):
        if checksum_list.get(item["file"]) != item["sha256"]:
            raise ValueError(f"Mismatch in SHA256SUMS: {item['file']}")

    # Read the real active bundle version from the runtime's version file.
    ampr = (ROOT / "vendor/ampr-emu-ng/VERSION").read_text().strip()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:[-._][A-Za-z0-9]+)*", ampr):
        raise ValueError(f"Unexpected AMPR version {ampr!r}")
    ampr_path = ROOT / "vendor/ampr-emu-ng/libSceAmpr.sprx"
    ampr_sums = dict(line.split(maxsplit=1) for line in
        (ROOT / "vendor/ampr-emu-ng/SHA256SUMS").read_text().splitlines() if line.strip())
    checked_hash(ampr_path, ampr_sums["libSceAmpr.sprx"])

    # This identifies the reference release for the selective source port.
    provenance = (ROOT / "vendor/kraken-mount/PROVENANCE.md").read_text()
    match = re.search(r"https://github\.com/quer3q/kraken-mount/releases/tag/v([0-9][A-Za-z0-9.-]*)", provenance)
    if not match:
        raise ValueError("Kraken version missing in provenance")
    kraken = match.group(1)
    archive = ROOT / "vendor/kraken-mount" / f"upstream-v{kraken}.tar.gz"
    if not archive.is_file():
        raise ValueError(f"Missing pinned Kraken source archive: {archive}")

    # This is what Payload Manager should see, independent of the artifact name.
    entry = (ROOT / "ps5/entry.c").read_text()
    process = re.search(r'SYS_thr_set_name\s*,\s*-1\s*,\s*"([^"]+)"', entry)
    if process is None:
        raise ValueError("PS5 process-name syscall missing")
    process_name = process.group(1)

    # Cross-check the two actual embedded payloads and on-console install paths.
    stack = (ROOT / "src/stack.rs").read_text()
    for sym, item in (("KSTUFF", kstuff), ("SHADOW", shadow)):
        path = item["file"]
        if f'const {sym}: &[u8] = include_bytes!("../vendor/unified/{path}");' not in stack:
            raise ValueError(f"{sym} is not the actual embedded file")
        if f'/data/titan-core/payloads/{path}' not in stack:
            raise ValueError(f"{sym} runtime path does not match the manifest")

    kversion = kstuff["embedded_version"].split()[0]
    sversion = shadow["embedded_version"].split()[0]
    return f"""## Included component versions

| Component | Version | Integration |
|---|---|---|
| **[ShadowMountPlus](https://github.com/drakmor/ShadowMountPlus)** | \`{sversion}\` | Bundled resident ELF |
| **[AMPR-EMU-NG](https://github.com/quer3q/ampr-emu-ng/releases/tag/v{ampr})** | \`{ampr}\` | Bundled official upstream SPRX |
| **[Kstuff Lite / FPKG](https://github.com/EchoStretch/kstuff-lite)** | \`{kversion}\` | Bundled resident ELF (user-supplied) |
| **[Kraken Mount](https://github.com/quer3q/kraken-mount/releases/tag/v{kraken})** | \`{kraken}\` | Selected upstream source changes; not a separate executable |

**Kraken Mount distinction:** TITAN CORE incorporates selected features from Kraken Mount v{kraken} source, including one-level nested game discovery, faster buffered metadata copying, per-title PlayGo stub exclusions via \`playgo_off\`, and batch registration on supported firmware. It does **not** bundle or launch a separate Kraken Mount executable or include every upstream feature. TITAN CORE keeps its own installer, original-path package handling, and ShadowMount coordination.

**Process identity:** \`{process_name}\` in the PS5 Active Processes list, as requested by the startup shim; actual display requires console confirmation. The downloadable executable remains \`TITAN-CORE.elf\` in the GitHub Actions \`payload\` artifact.

**Version scope:** These are the currently pinned *repository/build* versions, **not** promises that no newer upstream releases exist. They describe the binaries and source-port changes configured for a production build. Only a successful build of the corresponding commit confirms a finished ELF; actual hardware compatibility requires PS5 testing.

*This section is generated automatically from the checked-in component manifest, runtime version, Kraken source provenance, and PS5 process-name code. Updates are synchronized by GitHub Actions and the component import workflows. Do not edit this section manually.*"""


def sync(text: str, block: str) -> str:
    if START in text and END in text:
        prefix, rest = text.split(START, 1)
        _, suffix = rest.split(END, 1)
        return prefix + START + "\n" + block + "\n" + END + suffix
    if START in text or END in text:
        raise ValueError("Only one of the README section markers is present")
    anchor = "## Download the updated TITAN CORE build"
    if text.count(anchor) != 1:
        raise ValueError("README insertion anchor must occur exactly once")
    return text.replace(anchor, START + "\n" + block + "\n" + END + "\n\n" + anchor)


def main() -> None:
    ap = argparse.ArgumentParser()
    group = ap.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Fail if README is stale")
    group.add_argument("--write", action="store_true", help="Update generated section")
    args = ap.parse_args()
    before = README.read_text()
    after = sync(before, render())
    if args.check:
        if after != before:
            raise SystemExit("README component versions are out of sync; run python3 review/sync-component-readme.py --write")
        print("README component versions are synchronized.")
    elif after != before:
        README.write_text(after)
        print("Updated generated README component versions.")
    else:
        print("README component versions already current.")


if __name__ == "__main__":
    main()
