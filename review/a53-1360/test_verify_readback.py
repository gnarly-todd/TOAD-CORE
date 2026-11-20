"""Host-only synthetic fixtures. These are NOT PS5 hardware validation."""
import hashlib
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_readback import (
    HEADER_SIZE, READ_ACCESS_TYPE, READ_HEADER, InvalidRead,
    verify_readback,
)

class ReadbackTests(unittest.TestCase):
    ADDRESS = 0x57C14

    def captured(self, *, address=None, requested=4, transferred=4, status=0,
                 access_type=READ_ACCESS_TYPE, padding=0, data=None):
        if address is None:
            address = self.ADDRESS
        if data is None:
            data = bytes.fromhex('444dff97')
        return b'prefix' + READ_HEADER.pack(
            HEADER_SIZE, access_type, address, requested, transferred, status, padding,
        ) + data

    def verify(self, capture, **kwargs):
        return verify_readback(capture, offset=6, address=self.ADDRESS, size=4, **kwargs)

    def test_valid_stock_word_is_not_execution_proof(self):
        result = self.verify(self.captured(), stock_word_address=self.ADDRESS)
        self.assertEqual(result['matched_stock_site'], '0x57c14')
        self.assertEqual(result['sha256'], hashlib.sha256(bytes.fromhex('444dff97')).hexdigest())
        self.assertFalse(result['a53_code_execution_verified'])
        self.assertFalse(result['native_fpkg_enabled'])
        self.assertEqual(result['pkg_route'], 'ampr-emu-ng')

    def test_zero_transfer_is_a_failure_even_with_success_status(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(transferred=0))

    def test_status_0x20_is_a_failure_even_with_a_valid_echo(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(status=0x20, transferred=0))

    def test_address_echo_mismatch(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(address=self.ADDRESS + 4))

    def test_wrong_type_or_padding(self):
        for sample in [self.captured(access_type=0), self.captured(padding=1)]:
            with self.assertRaises(InvalidRead):
                self.verify(sample)

    def test_truncated_payload(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured()[:-1])

    def test_out_of_window_and_zero_length(self):
        with self.assertRaises(InvalidRead):
            verify_readback(self.captured(), offset=6, address=0xBFFFF, size=4)
        with self.assertRaises(InvalidRead):
            verify_readback(self.captured(), offset=6, address=self.ADDRESS, size=0)

    def test_wrong_hash(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(), expected_sha256='0' * 64)

    def test_wrong_stock_instruction(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(data=b'\0\0\0\0'), stock_word_address=self.ADDRESS)

    def test_read_not_covering_stock_site(self):
        with self.assertRaises(InvalidRead):
            self.verify(self.captured(), stock_word_address=0x5BC00)

if __name__ == '__main__':
    unittest.main()
