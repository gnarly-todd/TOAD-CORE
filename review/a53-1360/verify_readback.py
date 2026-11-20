#!/usr/bin/env python3
"""Offline verifier for SDBGP A53 read-result records on PS5 firmware 13.60.

Research-only: this never contacts the console, changes memory, or attests code
execution. A valid readback does NOT authorize native FPKG routing.
"""
from __future__ import annotations
import argparse
import hashlib
import hmac
import json
import struct
from pathlib import Path

READ_HEADER = struct.Struct('<IIQQQII')  # 0x28 bytes
HEADER_SIZE = READ_HEADER.size
READ_ACCESS_TYPE = 0x02010001
MAX_READ = 0x40000
READ_WINDOW_END = 0xC0000
STOCK_WORDS_1360 = {
    0x4A120: 0x94004611,
    0x4B860: 0x94004041,
    0x57C14: 0x97FF4D44,
    0x57D4C: 0x97FF4CF6,
    0x5BC00: 0x97FFF3B3,
}

class InvalidRead(ValueError):
    """Invalid or inconclusive A53 read result."""

def verify_readback(
    capture: bytes,
    *,
    offset: int,
    address: int,
    size: int,
    expected_sha256: str | None = None,
    stock_word_address: int | None = None,
) -> dict[str, object]:
    """Validate one explicitly located SDBGP result header and its data.

    This verifies record consistency, not where the capture came from.
    Never scan for a header and infer execution from a coincidental match.
    """
    if offset < 0 or address < 0 or size < 1 or size > MAX_READ:
        raise InvalidRead('invalid offset, address, or length')
    if address + size > READ_WINDOW_END:
        raise InvalidRead('request outside measured firmware 13.60 read window')
    if offset + HEADER_SIZE > len(capture):
        raise InvalidRead('truncated SDBGP result header')

    self_size, access_type, echoed_address, requested, transferred, status, padding = READ_HEADER.unpack_from(capture, offset)
    if self_size != HEADER_SIZE or access_type != READ_ACCESS_TYPE or padding != 0:
        raise InvalidRead('unexpected read header layout/type')
    if echoed_address != address or requested != size:
        raise InvalidRead('address or requested length not echoed correctly')
    if status != 0 or transferred != requested:
        raise InvalidRead(f'failed A53 read: status=0x{status:x}, transferred={transferred}, requested={requested}')
    end = offset + HEADER_SIZE + size
    if end > len(capture):
        raise InvalidRead('truncated read data')
    data = capture[offset + HEADER_SIZE:end]
    digest = hashlib.sha256(data).hexdigest()
    if expected_sha256 is not None:
        if len(expected_sha256) != 64 or not hmac.compare_digest(digest, expected_sha256.lower()):
            raise InvalidRead('readback SHA-256 differs from the expected reference')

    matched_stock_site = None
    if stock_word_address is not None:
        expected_word = STOCK_WORDS_1360.get(stock_word_address)
        if expected_word is None:
            raise InvalidRead('stock site is not included in the documented 13.60 profile')
        if stock_word_address < address or stock_word_address + 4 > address + size:
            raise InvalidRead('stock site is not fully covered by the readback')
        actual_word = int.from_bytes(data[stock_word_address - address:stock_word_address - address + 4], 'little')
        if actual_word != expected_word:
            raise InvalidRead(f'stock word mismatch at 0x{stock_word_address:x}')
        matched_stock_site = f'0x{stock_word_address:x}'

    return {
        'result': 'record_consistent',
        'a53_address': f'0x{address:x}',
        'bytes_verified': size,
        'sha256': digest,
        'matched_stock_site': matched_stock_site,
        'source_provenance': 'not_attested',
        'a53_code_execution_verified': False,
        'native_fpkg_enabled': False,
        'pkg_route': 'ampr-emu-ng',
    }

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('capture', type=Path, help='local SDBGP result buffer (never an ELF/payload)')
    parser.add_argument('--offset', type=lambda x: int(x, 0), required=True, help='exact result-header offset in capture')
    parser.add_argument('--address', type=lambda x: int(x, 0), required=True, help='A53 physical address requested')
    parser.add_argument('--size', type=lambda x: int(x, 0), required=True, help='requested read length')
    parser.add_argument('--expected-sha256', help='optional known-good SHA-256 of read data')
    parser.add_argument('--stock-site', type=lambda x: int(x, 0), help='optional known 13.60 stock instruction address')
    parser.add_argument('--firmware', required=True, help='operator-declared console firmware, must be 13.60')
    args = parser.parse_args()
    if args.firmware != '13.60':
        parser.error('this prototype is restricted to firmware 13.60')
    try:
        result = verify_readback(
            args.capture.read_bytes(), offset=args.offset, address=args.address, size=args.size,
            expected_sha256=args.expected_sha256, stock_word_address=args.stock_site,
        )
    except (InvalidRead, OSError) as err:
        print(json.dumps({'result': 'rejected', 'reason': str(err), 'native_fpkg_enabled': False}, sort_keys=True))
        return 1
    result['firmware'] = '13.60 (operator-declared, not cryptographically attested)'
    print(json.dumps(result, sort_keys=True))
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
