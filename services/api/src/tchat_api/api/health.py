"""Sondas de salud."""

from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy import text
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api import __version__
from tchat_api.db.session import get_session
from tchat_api.schemas.common import HealthResponse, ReadinessResponse

router = APIRouter(tags=["salud"])


@router.get("/health", response_model=HealthResponse, summary="Liveness")
async def health() -> HealthResponse:
    """Responde si el proceso está vivo. No toca la base de datos."""
    return HealthResponse(status="ok", version=__version__)


@router.get(
    "/health/ready",
    response_model=ReadinessResponse,
    summary="Readiness",
    responses={status.HTTP_503_SERVICE_UNAVAILABLE: {"description": "Dependencia caída"}},
)
async def readiness(
    session: Annotated[AsyncSession, Depends(get_session)],
) -> ReadinessResponse:
    """Comprueba que la base de datos responde."""
    try:
        await session.execute(text("SELECT 1"))
    except SQLAlchemyError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="base de datos no disponible",
        ) from exc
    return ReadinessResponse(status="ok", database="up")
