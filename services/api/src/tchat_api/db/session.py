"""Motor y sesiones de base de datos."""

from collections.abc import AsyncIterator
from typing import Any

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.pool import NullPool

from tchat_api.core.config import settings

engine_kwargs: dict[str, Any] = {"pool_pre_ping": True}
if settings.environment == "test":
    # Los tests corren en varios bucles de eventos; sin pool no se reutilizan conexiones.
    engine_kwargs["poolclass"] = NullPool

engine = create_async_engine(settings.database_url, **engine_kwargs)
session_factory = async_sessionmaker(engine, expire_on_commit=False)


async def get_session() -> AsyncIterator[AsyncSession]:
    """Dependencia de FastAPI que entrega una sesión por petición."""
    async with session_factory() as session:
        yield session
