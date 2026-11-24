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
EXPECTED_NAME = "kstuff1.13-fpkg-dr-test8-rest-fast.elf"
EXPECTED_SHA = "221fa41c43597677c1153d5224174be9062105c0c9a3379e96cb45d278aa52fc"
EXPECTED_SIZE = 1845320
OLD_NAME = "kstuff1.13-fpkg-dr-test8-rest.elf"
OLD_SHA = "127332652374157b8ee0f5eb1128976f9055b88df5da89c0c54183e9c326fc7f"
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



FAST_VERSION = "1.13-fpkg-dr-test8-rest-fast"
BASE_VERSION = "1.13-fpkg-dr-test8-rest"
APP_VERSION = "1.1.1-beta.1"


def update_one(path: str, before: str, after: str) -> None:
    p = ROOT / path
    data = p.read_text(encoding="utf-8")
    count = data.count(before)
    if count != 1:
        raise ValueError(f"Expected one {before!r} in {path}, got {count}")
    p.write_text(data.replace(before, after), encoding="utf-8")


def run_checks() -> None:
    subprocess.run(["python3", "review/verify-unified-payloads.py"], cwd=ROOT, check=True)
    subprocess.run(["python3", "review/sync-component-readme.py", "--check"], cwd=ROOT, check=True)
    for folder in ("vendor/unified", "vendor/ampr-emu-ng",
                   "vendor/dlc_emu", "vendor/pgo_stub"):
        subprocess.run(["sha256sum", "-c", "--strict", "SHA256SUMS"],
                       cwd=ROOT / folder, check=True)


def main() -> None:
    data = checked_elf(archive_download())
    if b"PPR FAST" not in data or BASE_VERSION.encode() not in data:
        raise ValueError("Fast/PPR string or embedded version missing")
    parent = ROOT / "vendor/unified"
    old_path = parent / OLD_NAME
    if not old_path.is_file():
        raise ValueError("Original kstuff bundle is missing")
    if hashlib.sha256(old_path.read_bytes()).hexdigest() != OLD_SHA:
        raise ValueError("Current bundle checksum differs from stable v1.1")
    (parent / EXPECTED_NAME).write_bytes(data)

    # Replace only selected kstuff references; all other payloads stay pinned.
    for path in ("src/stack.rs", "review/verify-unified-payloads.py",
                 "vendor/unified/SHA256SUMS", "vendor/unified/sources/README.md",
                 "vendor/unified/PROVENANCE.md"):
        p = ROOT / path
        text = p.read_text(encoding="utf-8")
        if OLD_NAME not in text:
            raise ValueError(f"Previous kstuff filename missing in {path}")
        p.write_text(text.replace(OLD_NAME, EXPECTED_NAME), encoding="utf-8")
    for path in ("review/verify-unified-payloads.py",
                 "vendor/unified/SHA256SUMS", "vendor/unified/PROVENANCE.md"):
        update_one(path, OLD_SHA, EXPECTED_SHA)

    update_one("src/stack.rs", '"kstuff_bundle":"1.13-fpkg-dr-test8-rest"',
               f'"kstuff_bundle":"{FAST_VERSION}"')
    update_one("src/stack.rs", 'kstuff 1.13-fpkg-dr-test8-rest\\n"',
               f'kstuff {FAST_VERSION}\\n"')
    update_one("src/stack.rs", 'Starting bundled kstuff 1.13-fpkg-dr-test8-rest"',
               f'Starting bundled kstuff {FAST_VERSION}"')

    manifest_file = parent / "MANIFEST.json"
    manifest = json.loads(manifest_file.read_text())
    entry = [c for c in manifest["components"] if c["name"] == "kstuff-fpkg"]
    if len(entry) != 1 or entry[0]["file"] != OLD_NAME or entry[0]["sha256"] != OLD_SHA:
        raise ValueError("Unexpected existing kstuff manifest")
    entry[0].update({
        "file": EXPECTED_NAME,
        "sha256": EXPECTED_SHA,
        "embedded_version": BASE_VERSION,
        "display_version": FAST_VERSION,
        "variant": "fast",
        "provenance": ("The user's fast ELF (2026-11-23) matched the imported archive "
                       "byte-for-byte by SHA-256. Embedded version string omits fast; "
                       "upstream matching source revision is unverified.")
    })
    manifest["retrieved"] = "2026-11-23"
    manifest_file.write_text(json.dumps(manifest, indent=2) + "\n")

    prov_path = parent / "PROVENANCE.md"
    prov = prov_path.read_text(encoding="utf-8")
    marker = "Embedded strings report "
    if marker not in prov:
        raise ValueError("Provenance version identity missing")
    prov = prov.replace(marker, "Fast/PPR variant. " + marker, 1)
    prov += ("\nThe Fast/PPR variant retains the original internal welcome version "
             "without a fast suffix, while filename and SHA-256 identify the distinct ELF. "
             "Real PS5 timing/stability has not been measured.\n")
    prov_path.write_text(prov, encoding="utf-8")

    update_one("CREDITS.md", "kstuff 1.13-fpkg-dr-test8-rest |",
               "kstuff 1.13-fpkg-dr-test8-rest-fast |")
    update_one("Cargo.toml", 'name = "titan-core"\nversion = "1.1.0"',
               f'name = "titan-core"\nversion = "{APP_VERSION}"')
    update_one("Cargo.lock", 'name = "titan-core"\nversion = "1.1.0"',
               f'name = "titan-core"\nversion = "{APP_VERSION}"')
    update_one("README.md", "## Version 1.1\n",
               "## Version 1.1b (Fast Kstuff variant)\n")

    notes = ROOT / "RELEASE-NOTES.md"
    text = notes.read_text()
    if not text.startswith("# TITAN CORE v1.1 (first public release"):
        raise ValueError("Unexpected release notes header")
    notes.write_text(
        "# TITAN CORE v1.1b - Fast Kstuff pre-release\n\n"
        "This optional pre-release replaces only the checksum-pinned kstuff test8-rest "
        "ELF with the user-provided test8-rest-fast variant. "
        "Runtime process name stays titan-core.elf. App SemVer: "
        f"{APP_VERSION}.\n\n"
        "The internal version string remains " + BASE_VERSION + " but the selected "
        "binary and SHA-256 identify the Fast/PPR variant. Speed and console "
        "stability have not been benchmarked.\n\n---\n\n" + text
    )

    # Generate the README version table from the exact new manifest.
    subprocess.run(["python3", "review/sync-component-readme.py", "--write"],
                   cwd=ROOT, check=True)
    old_path.unlink()
    run_checks()
    print(f"Imported verified Fast Kstuff ELF: {EXPECTED_SHA}", flush=True)


if __name__ == "__main__":
    main()
