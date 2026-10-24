"""Fixtures compartidas por los tests.

La base de datos de pruebas se crea sola (`tchat_test`) y se reinicia por sesión.
No se necesita Firebase: la identidad se inyecta con `app.dependency_overrides`.
`tests/__init__.py` configura el entorno antes de importar la app.
"""

import asyncio
from collections.abc import AsyncIterator, Callable, Iterator

import asyncpg
import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import text
from sqlalchemy.engine import make_url

from tchat_api.api.deps import get_identity
from tchat_api.core.config import settings
from tchat_api.core.security import Identity
from tchat_api.db.base import Base
from tchat_api.db.session import engine
from tchat_api.main import app

TABLES = (
    "device_codes",
    "device_tokens",
    "messages",
    "conversation_members",
    "conversations",
    "friendships",
    "users",
)


async def _prepare_database() -> None:
    url = make_url(settings.database_url)
    assert url.database
    assert url.database.endswith("_test"), "Los tests solo pueden ejecutarse contra una base *_test"

    admin = await asyncpg.connect(
        host=url.host,
        port=url.port or 5432,
        user=url.username,
        password=url.password,
        database="postgres",
    )
    try:
        exists = await admin.fetchval("SELECT 1 FROM pg_database WHERE datname = $1", url.database)
        if not exists:
            await admin.execute(f'CREATE DATABASE "{url.database}"')
    finally:
        await admin.close()

    async with engine.begin() as connection:
        await connection.run_sync(Base.metadata.drop_all)
    async with engine.begin() as connection:
        await connection.run_sync(Base.metadata.create_all)


@pytest.fixture(scope="session", autouse=True)
def database() -> Iterator[None]:
    asyncio.run(_prepare_database())
    yield
    asyncio.run(engine.dispose())


@pytest.fixture(autouse=True)
async def clean_tables() -> AsyncIterator[None]:
    async with engine.begin() as connection:
        await connection.execute(text(f"TRUNCATE {', '.join(TABLES)} CASCADE"))
    yield


@pytest.fixture
async def client() -> AsyncIterator[AsyncClient]:
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as http_client:
        yield http_client


@pytest.fixture
def identity_holder() -> Iterator[dict[str, Identity]]:
    holder: dict[str, Identity] = {
        "identity": Identity(uid="test-user", email="test@example.com", display_name="Test")
    }

    def override() -> Identity:
        return holder["identity"]

    app.dependency_overrides[get_identity] = override
    yield holder
    app.dependency_overrides.pop(get_identity, None)


@pytest.fixture
def act_as(identity_holder: dict[str, Identity]) -> Callable[..., Identity]:
    """Cambia la identidad que verá la próxima petición."""

    def _act_as(uid: str, *, email: str | None = None, name: str | None = None) -> Identity:
        identity = Identity(
            uid=uid,
            email=email or f"{uid}@example.com",
            display_name=name or uid.replace("-", " ").title(),
        )
        identity_holder["identity"] = identity
        return identity

    return _act_as
