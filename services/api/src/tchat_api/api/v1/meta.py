"""Endpoints de metadatos."""

from fastapi import APIRouter

from tchat_api import __version__
from tchat_api.core.config import settings
from tchat_api.schemas.common import MetaResponse

router = APIRouter(tags=["meta"])


@router.get("/meta", response_model=MetaResponse, summary="Metadatos públicos de la API")
async def meta() -> MetaResponse:
    return MetaResponse(
        name=settings.app_name,
        version=__version__,
        environment=settings.environment,
        firebase_project_id=settings.firebase_project_id,
    )
