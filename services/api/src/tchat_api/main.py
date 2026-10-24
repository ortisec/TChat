"""Punto de entrada de la API."""

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from tchat_api import __version__
from tchat_api.api.health import router as health_router
from tchat_api.api.v1.router import router as v1_router
from tchat_api.core.config import settings
from tchat_api.db.session import engine
from tchat_api.realtime.ws import router as ws_router
from tchat_api.services.errors import ServiceError


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
            {"name": "autenticación", "description": "Sincronización y tokens del terminal."},
            {"name": "usuarios", "description": "Perfiles y búsqueda."},
            {"name": "amistades", "description": "Solicitudes y lista de amigos."},
            {"name": "conversaciones", "description": "Conversaciones y mensajes."},
        ],
    )
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.cors_origin_list,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    @app.exception_handler(ServiceError)
    async def service_error_handler(_: Request, exc: ServiceError) -> JSONResponse:
        return JSONResponse(status_code=exc.status_code, content={"detail": exc.code})

    app.include_router(health_router)
    app.include_router(v1_router, prefix="/api/v1")
    app.include_router(ws_router)
    return app


app = create_app()
