"""Tests for Flipper file parsers."""

import sys
import os
import unittest

# Add python module to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src-tauri", "python"))

from parsers import parse_key_value, parse_sub, parse_ir, parse_nfc


class TestSubGhzFile(unittest.TestCase):
    SAMPLE = """Filetype: Flipper SubGhz Key File
Version: 1
Frequency: 433920000
Preset: FuriHalSubGhzPresetOok650Async
Protocol: Princeton
Bit: 24
Key: 001122334455"""

    def test_parse_kv(self):
        kvs = parse_key_value(self.SAMPLE)
        self.assertEqual(len(kvs), 7)
        self.assertEqual(kvs[0], ("Filetype", "Flipper SubGhz Key File"))

    def test_parse_sub(self):
        result = parse_sub(self.SAMPLE)
        self.assertEqual(result["file_type"], "subghz")
        freq = [f for f in result["fields"] if f["key"] == "frequency"]
        self.assertTrue(len(freq) > 0)
        self.assertEqual(freq[0]["value"], 433920000)

    def test_empty(self):
        self.assertEqual(len(parse_key_value("")), 0)

    def test_comments(self):
        kvs = parse_key_value("# comment\nKey: Value")
        self.assertEqual(len(kvs), 1)


class TestIRFile(unittest.TestCase):
    SAMPLE = """Filetype: IR
Version: 1
Protocol: NEC
Address: 0x00
Command: 0x45"""

    def test_parse_ir(self):
        result = parse_ir(self.SAMPLE)
        self.assertEqual(result["file_type"], "infrared")
        proto = [f for f in result["fields"] if f["key"] == "protocol"]
        self.assertEqual(len(proto), 1)
        self.assertEqual(proto[0]["value"], "NEC")


class TestNFCFile(unittest.TestCase):
    SAMPLE = """Filetype: Flipper NFC Key
Version: 1
Device Type: Mifare Classic
UID: 04:1E:23:4A:5B:6C
ATQA: 00 44
SAK: 0x08"""

    def test_parse_nfc(self):
        result = parse_nfc(self.SAMPLE)
        self.assertEqual(result["file_type"], "nfc")
        uid = [f for f in result["fields"] if f["key"] == "uid"]
        self.assertEqual(len(uid), 1)
        self.assertEqual(uid[0]["value"], "04:1E:23:4A:5B:6C")


if __name__ == "__main__":
    unittest.main(verbosity=2)
