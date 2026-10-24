"""Generación de identificadores."""

import uuid

from tchat_api.core.ids import uuid7


def test_uuid7_version_y_variante() -> None:
    value = uuid7()

    assert value.version == 7
    assert value.variant == uuid.RFC_4122


def test_uuid7_es_estrictamente_creciente() -> None:
    values = [uuid7() for _ in range(50)]

    assert values == sorted(values)
    assert len(set(values)) == 50
