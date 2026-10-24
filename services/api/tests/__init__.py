"""Configuración temprana del entorno de pruebas.

Se ejecuta antes de importar `tchat_api` (pytest importa el paquete `tests` primero),
lo que permite apuntar la aplicación a una base de datos de pruebas.
"""

import os


def _test_database_url() -> str:
    """Deriva siempre una base *_test a partir de la URL configurada."""
    candidate = os.environ.get("TEST_DATABASE_URL") or os.environ.get("DATABASE_URL")
    if not candidate:
        return "postgresql+asyncpg://tchat:tchat@localhost:5432/tchat_test"
    base, separator, query = candidate.partition("?")
    if not base.endswith("_test"):
        base = f"{base}_test"
    return f"{base}{separator}{query}"


os.environ["DATABASE_URL"] = _test_database_url()
os.environ.setdefault("ENVIRONMENT", "test")
os.environ.setdefault("AUTH_JWT_SECRET", "test-secret-change-me-32-characters-long")
os.environ.setdefault("DEVICE_POLL_INTERVAL_SECONDS", "0")
