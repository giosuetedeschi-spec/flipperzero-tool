"""Python reference implementation of parsers for testing."""

import json
from typing import List, Tuple


def parse_key_value(raw: str) -> List[Tuple[str, str]]:
    """Parse key:value lines, ignoring comments and empty lines."""
    result = []
    for line in raw.strip().split("\n"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(":", 1)
        if len(parts) == 2:
            result.append((parts[0].strip(), parts[1].strip()))
    return result


def parse_sub(raw: str):
    """Parse a Sub-GHz file."""
    kvs = parse_key_value(raw)
    fields = []
    for k, v in kvs:
        if k == "Frequency":
            try:
                fields.append({"key": "frequency", "value": int(v)})
            except ValueError:
                fields.append({"key": "frequency", "value": v})
        elif k == "Bit":
            try:
                fields.append({"key": "bit", "value": int(v)})
            except ValueError:
                fields.append({"key": "bit", "value": v})
        else:
            fields.append({"key": k.lower().replace(" ", "_"), "value": v})
    return {"file_type": "subghz", "fields": fields, "raw_preview": "\n".join(raw.split("\n")[:20])}


def parse_ir(raw: str):
    """Parse an IR file."""
    kvs = parse_key_value(raw)
    fields = []
    for k, v in kvs:
        fields.append({"key": k.lower().replace(" ", "_"), "value": v})
    return {"file_type": "infrared", "fields": fields, "raw_preview": "\n".join(raw.split("\n")[:20])}


def parse_nfc(raw: str):
    """Parse an NFC file."""
    kvs = parse_key_value(raw)
    fields = []
    for k, v in kvs:
        fields.append({"key": k.lower().replace(" ", "_"), "value": v})
    return {"file_type": "nfc", "fields": fields, "raw_preview": "\n".join(raw.split("\n")[:20])}
