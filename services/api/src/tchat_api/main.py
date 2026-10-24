"""Punto de entrada de la API."""

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from tchat_api import __version__
from tchat_api.api.health import router as health_router
from tchat_api.api.v1.router import router as v1_router
from tchat_api.core.config import settings
from tchat_api.db.session import engine


@asynccontextmanager
async def lifespan(_: FastAPI) -> AsyncIterator[None]:
    yield
    await engine.dispose()


def create_app() -> FastAPI:
    app = FastAPI(
        title=settings.app_name,
        version=__version__,
        description="API REST y WebSocket de TChat.",
        lifespan=lifespan,
        openapi_tags=[
            {"name": "salud", "description": "Sondas de liveness y readiness."},
            {"name": "meta", "description": "Información pública del servicio."},
        ],
    )
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.cors_origin_list,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    app.include_router(health_router)
    app.include_router(v1_router, prefix="/api/v1")
    return app


app = create_app()
