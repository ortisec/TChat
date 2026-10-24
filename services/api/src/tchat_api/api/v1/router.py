"""Router de la versión 1."""

from fastapi import APIRouter

from tchat_api.api.v1 import auth, conversations, friends, meta, users

router = APIRouter()
router.include_router(auth.router)
router.include_router(users.router)
router.include_router(friends.router)
router.include_router(conversations.router)
router.include_router(meta.router)
