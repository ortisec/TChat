"""Identificadores ordenables por tiempo."""

import secrets
import time
import uuid


def uuid7() -> uuid.UUID:
    """Genera un UUID v7 (RFC 9562): los 48 bits altos son el timestamp en ms."""
    unix_ts_ms = time.time_ns() // 1_000_000
    rand_a = secrets.randbits(12)
    rand_b = secrets.randbits(62)

    value = (unix_ts_ms & 0xFFFF_FFFF_FFFF) << 80
    value |= 0x7 << 76
    value |= rand_a << 64
    value |= 0b10 << 62
    value |= rand_b & 0x3FFF_FFFF_FFFF_FFFF
    return uuid.UUID(int=value)
