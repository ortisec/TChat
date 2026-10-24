"""Generación de identificadores."""

import time
import uuid

from tchat_api.core.ids import uuid7


def test_uuid7_version_y_variante() -> None:
    value = uuid7()

    assert value.version == 7
    assert value.variant == uuid.RFC_4122


def test_uuid7_ordenable_por_tiempo() -> None:
    primero = uuid7()
    time.sleep(0.002)
    segundo = uuid7()

    assert primero.int < segundo.int
