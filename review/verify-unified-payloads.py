#!/usr/bin/env python3
"""Fail closed if the unified build stops embedding the owner's pinned ELF uploads.

The historical archives in vendor/unified/sources/ are intentionally inert:
they must never be treated as the active payloads or substituted during build.
"""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UNIFIED = ROOT / "vendor" / "unified"
EXPECTED = {
    "KSTUFF": (
        "kstuff1.13-fpkg-dr-test8-rest.elf",
        "127332652374157b8ee0f5eb1128976f9055b88df5da89c0c54183e9c326fc7f",
        "kstuff-fpkg",
    ),
    "SHADOW": (
        "shadowmountplus.elf",
        "8755172ec9bef1bff5453ea75606e0caf7c41b5963f94f5a1087e7e8c6d9b1bd",
        "ShadowMountPlus",
    ),
}


def ensure(condition: bool, reason: str) -> None:
    if not condition:
        raise SystemExit(f"Unified payload verification FAILED: {reason}")


def main() -> None:
    stack = (ROOT / "src" / "stack.rs").read_text(encoding="utf-8")
    sums: dict[str, str] = {}
    for line in (UNIFIED / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
        pieces = line.split()
        ensure(len(pieces) == 2 and pieces[1] not in sums, "invalid or duplicate SHA256SUMS entry")
        sums[pieces[1]] = pieces[0].lower()

    manifest = json.loads((UNIFIED / "MANIFEST.json").read_text(encoding="utf-8"))
    components = {item["name"]: item for item in manifest["components"]}
    expected_names = {name for name, _, _ in EXPECTED.values()}
    ensure(set(sums) == expected_names, "SHA256SUMS does not exclusively pin the two uploaded ELFs")

    for symbol, (name, expected_hash, component_name) in EXPECTED.items():
        blob = UNIFIED / name
        ensure(blob.is_file() and not blob.is_symlink(), f"{name} missing or is a symlink")
        with blob.open("rb") as handle:
            actual_hash = hashlib.file_digest(handle, "sha256").hexdigest()
        ensure(actual_hash == expected_hash, f"{name} differs from the original uploaded ELF")
        ensure(sums[name] == expected_hash, f"SHA256SUMS does not pin {name}")
        entry = components.get(component_name)
        ensure(entry is not None and entry.get("file") == name and entry.get("sha256") == expected_hash,
               f"MANIFEST.json does not pin {name}")
        pattern = rf'const\s+{symbol}\s*:\s*&\[u8\]\s*=\s*include_bytes!\("\.\./vendor/unified/{re.escape(name)}"\);'
        ensure(len(re.findall(pattern, stack)) == 1, f"src/stack.rs no longer embeds {name} exactly once")
        runtime_path = rf'const\s+{symbol}_PATH\s*:\s*&str\s*=\s*"/data/titan-core/payloads/{re.escape(name)}";'
        ensure(len(re.findall(runtime_path, stack)) == 1, f"runtime file path no longer uses {name}")

    # Source snapshots are preserved for attribution, never executable build inputs.
    # Reject an accidental switch to importing an ELF/source archive from that folder.
    for folder, patterns in [(ROOT / "src", ["*.rs"]), (ROOT / "ps5", ["*.sh", "*.py"])]:
        for pattern in patterns:
            for code in folder.rglob(pattern):
                ensure("vendor/unified/sources/" not in code.read_text(encoding="utf-8"),
                       f"build/runtime code references historical source archives: {code.relative_to(ROOT)}")
    print("Unified payloads verified: only the two uploaded ELFs are pinned and embedded; historical sources are inert")


if __name__ == "__main__":
    main()
