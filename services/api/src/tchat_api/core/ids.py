"""UUID v7 monótonos: ordenables por tiempo incluso dentro del mismo milisegundo."""

import secrets
import threading
import time
import uuid

_lock = threading.Lock()
_last_ms = 0
_counter = 0


def uuid7() -> uuid.UUID:
    """Genera un UUID v7 (RFC 9562) estrictamente creciente en este proceso."""
    global _last_ms, _counter

    with _lock:
        now_ms = time.time_ns() // 1_000_000
        if now_ms > _last_ms:
            _last_ms = now_ms
            _counter = 0
        else:
            _counter += 1
            if _counter > 0xFFF:
                _last_ms += 1
                _counter = 0

        timestamp = _last_ms
        rand_a = _counter

    rand_b = secrets.randbits(62)

    value = (timestamp & 0xFFFF_FFFF_FFFF) << 80
    value |= 0x7 << 76
    value |= rand_a << 64
    value |= 0b10 << 62
    value |= rand_b & 0x3FFF_FFFF_FFFF_FFFF
    return uuid.UUID(int=value)
