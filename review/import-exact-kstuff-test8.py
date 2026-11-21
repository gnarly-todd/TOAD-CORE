#!/usr/bin/env python3
"""Import only the exact kstuff ELF supplied by the TITAN CORE owner.

The forum archive is merely a transport: reject it unless the selected entry
matches the owner's locally computed full SHA-256 and size exactly.
"""
from __future__ import annotations

import hashlib
import html
import io
import json
import re
import subprocess
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXPECTED_NAME = "kstuff1.13-fpkg-dr-test8-rest.elf"
EXPECTED_SHA = "127332652374157b8ee0f5eb1128976f9055b88df5da89c0c54183e9c326fc7f"
EXPECTED_SIZE = 1845320
OLD_NAME = "kstuff-fpkg-1.13-dr-test6.elf"
OLD_SHA = "b1f60c1ec1685904d396da16710dc9d5676bf0880abf74c65459c886010b9443"
ZIP_LANDING = "https://www.mediafire.com/file/t9ioa2ltqp6xqi6/kstuff.zip/file"
API_LINK = ("https://www.mediafire.com/api/1.5/file/get_links.php"
            "?quick_key=t9ioa2ltqp6xqi6&response_format=json")
HEADERS = {"User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36"}


def get(url: str, limit: int) -> bytes:
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=60) as response:
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f"Download exceeds {limit} bytes; refusing")
    return data


def archive_download() -> bytes:
    candidates = []
    # MediaFire's official links API may offer a normal_download link.
    try:
        doc = json.loads(get(API_LINK, 1 << 20))
        links = doc.get("response", {}).get("links", [])
        if isinstance(links, list):
            for link in links:
                candidates.extend(link[k] for k in ("normal_download", "direct_download")
                                  if isinstance(link.get(k), str))
        else:
            candidates.extend(links[k] for k in ("normal_download", "direct_download")
                              if isinstance(links.get(k), str))
    except Exception as exc:
        print(f"MediaFire links API unavailable: {exc}", flush=True)

    page = get(ZIP_LANDING, 4 << 20).decode("utf-8", "replace")
    # Literal direct download anchor / escaped JS link. No arbitrary code is executed.
    found = re.findall(r'https?:(?:/|\\/){2}download[^"<> \t\r\n]+', page)
    candidates.extend(html.unescape(s).replace("\\/", "/").replace("\\u0026", "&")
                      for s in found)
    anchor = re.search(r'<a[^>]*id=["\x27]downloadButton["\x27][^>]*>', page)
    if anchor:
        href = re.search(r'href=["\x27]([^"\x27]+)', anchor.group(0))
        if href:
            candidates.insert(0, html.unescape(href.group(1)))
    # Some mirrors use the www.mediafire.com/file_premium/download?... endpoint.
    seen = set()
    for candidate in candidates:
        candidate = candidate.split('"')[0].split("'")[0].replace("&amp;", "&")
        if not candidate.startswith("https://") or candidate in seen:
            continue
        seen.add(candidate)
        try:
            blob = get(candidate, 120 << 20)
            if zipfile.is_zipfile(io.BytesIO(blob)):
                print("Downloaded a ZIP archive; verifying selected ELF", flush=True)
                return blob
            print(f"Rejected a non-ZIP response from {candidate.split('/')[2]}", flush=True)
        except Exception as exc:
            print(f"Mirror download failed: {exc}", flush=True)
    raise RuntimeError("No working ZIP download. Nothing in the repository was replaced.")


def checked_elf(zipped: bytes) -> bytes:
    with zipfile.ZipFile(io.BytesIO(zipped)) as z:
        entries = [entry for entry in z.infolist()
                   if Path(entry.filename).name == EXPECTED_NAME and not entry.is_dir()]
        if len(entries) != 1:
            raise ValueError(f"Expected one exact {EXPECTED_NAME}, found {len(entries)}")
        entry = entries[0]
        if entry.file_size != EXPECTED_SIZE:
            raise ValueError(f"ELF length mismatch: {entry.file_size} != {EXPECTED_SIZE}")
        data = z.read(entry)
    digest = hashlib.sha256(data).hexdigest()
    if digest != EXPECTED_SHA:
        raise ValueError(f"ELF SHA-256 mismatch: {digest} != owner's {EXPECTED_SHA}")
    if not data.startswith(b"\x7fELF\x02\x01") or data[18:20] != b"\x3e\x00":
        raise ValueError("ELF format is not x86-64 ELF64")
    print(f"Exact uploaded ELF verified: {EXPECTED_NAME}, {len(data)} bytes, sha256={digest}", flush=True)
    return data


def rewrite(path: str, old: str, new: str, expect: bool = True) -> None:
    p = ROOT / path
    text = p.read_text()
    if expect and old not in text:
        raise ValueError(f"Expected reference missing from {path}: {old}")
    text = text.replace(old, new)
    p.write_text(text)


def main() -> None:
    data = checked_elf(archive_download())
    parent = ROOT / "vendor/unified"
    (parent / EXPECTED_NAME).write_bytes(data)

    # Do not rename other embedded services, install policy, or process identifier.
    rewrite("src/stack.rs", OLD_NAME, EXPECTED_NAME)
    rewrite("src/stack.rs", "1.13-fpkg-dr-test6", "1.13-fpkg-dr-test8-rest")
    rewrite("review/verify-unified-payloads.py", OLD_NAME, EXPECTED_NAME)
    rewrite("review/verify-unified-payloads.py", OLD_SHA, EXPECTED_SHA)
    rewrite("vendor/unified/SHA256SUMS", OLD_NAME, EXPECTED_NAME)
    rewrite("vendor/unified/SHA256SUMS", OLD_SHA, EXPECTED_SHA)

    manifest_path = ROOT / "vendor/unified/MANIFEST.json"
    manifest = json.loads(manifest_path.read_text())
    entries = [x for x in manifest["components"] if x["name"] == "kstuff-fpkg"]
    if len(entries) != 1 or entries[0]["sha256"] != OLD_SHA:
        raise ValueError("Unexpected existing kstuff manifest")
    entry = entries[0]
    entry["file"] = EXPECTED_NAME
    entry["sha256"] = EXPECTED_SHA
    entry["embedded_version"] = "1.13-fpkg-dr-test8-rest"
    entry["provenance"] = ("User-supplied ELF hash matched the pinned 2026-11-21 import; "
                            "matching source revision and upstream release digest not independently verified")
    manifest["retrieved"] = "2026-11-21"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    rewrite("README.md", OLD_NAME, EXPECTED_NAME)
    rewrite("README.md", "1.13-dr-test6", "1.13-fpkg-dr-test8-rest")
    rewrite("vendor/unified/sources/README.md", OLD_NAME, EXPECTED_NAME)
    rewrite("vendor/unified/PROVENANCE.md", OLD_NAME, EXPECTED_NAME)
    rewrite("vendor/unified/PROVENANCE.md", OLD_SHA, EXPECTED_SHA)
    rewrite("vendor/unified/PROVENANCE.md", "1,818,680 bytes", "1,845,320 bytes")
    rewrite("vendor/unified/PROVENANCE.md", "1.13-fpkg-dr-test6", "1.13-fpkg-dr-test8-rest")
    rewrite("vendor/unified/PROVENANCE.md",
            "The Git blob object identifiers of both uploaded files match the Git blob hashes of the supplied local attachments.",
            "The new kstuff ELF was recovered from a public archive and independently SHA-256 verified "
            "against the user's exact uploaded ELF; ShadowMountPlus is byte-for-byte unchanged.")
    rewrite("CREDITS.md", "kstuff-fpkg 1.13-dr-test6", "kstuff-fpkg 1.13-fpkg-dr-test8-rest")

    # The filename is versioned to prevent reusing a resident unpacked test6 file.
    old_elf = parent / OLD_NAME
    if not old_elf.exists() or hashlib.sha256(old_elf.read_bytes()).hexdigest() != OLD_SHA:
        raise ValueError("Previously bundled ELF differs from the verified test6 version")
    old_elf.unlink()
    subprocess.run(["python3", "review/verify-unified-payloads.py"], cwd=ROOT, check=True)
    subprocess.run(["bash", "-c", "cd vendor/unified && sha256sum -c --strict SHA256SUMS"],
                   cwd=ROOT, check=True)
    print("Prepared verified kstuff upgrade without changing other resident payloads.", flush=True)


if __name__ == "__main__":
    main()
