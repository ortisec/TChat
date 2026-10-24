"""Router de la versión 1."""

from fastapi import APIRouter

from tchat_api.api.v1 import meta

router = APIRouter()
router.include_router(meta.router)
