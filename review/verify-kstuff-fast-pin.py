#!/usr/bin/env python3
"""Fail closed unless the exact owner-approved Fast Kstuff binary is bundled.

Only update this policy when the owner explicitly supplies a new version.
"""
from pathlib import Path
import hashlib
import json

root=Path(__file__).resolve().parent.parent
filename="kstuff1.13-fpkg-dr-test8-rest-fast.elf"
version="1.13-fpkg-dr-test8-rest-fast"
digest="221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc"
path=root/"vendor/unified"/filename
if not path.is_file() or path.is_symlink():
    raise SystemExit("Owner-approved Fast Kstuff ELF not found")
actual=hashlib.sha256(path.read_bytes()).hexdigest()
if actual!=digest:
    raise SystemExit(f"Fast Kstuff SHA mismatch: {actual}")
manifest=json.loads((root/"vendor/unified/MANIFEST.json").read_text())
matching=[x for x in manifest["components"] if x["name"]=="kstuff-fpkg"]
if len(matching)!=1 or (matching[0]["file"],matching[0]["sha256"],
                          matching[0].get("display_version"))!=(filename,digest,version):
    raise SystemExit("Kstuff manifest is no longer pinned to Fast test8")
if (digest+"  "+filename) not in (root/"vendor/unified/SHA256SUMS").read_text().splitlines():
    raise SystemExit("Kstuff checksum list was changed")
src=(root/"src/stack.rs").read_text()
required=(f'include_bytes!("../vendor/unified/{filename}")',
          f'"/data/titan-core/payloads/{filename}"',
          f'"kstuff_bundle":"{version}"')
for marker in required:
    if marker not in src:
        raise SystemExit("Bundled Fast Kstuff source reference changed: "+marker)
print(f"Verified persistent Fast Kstuff pin: {filename} {digest}")
