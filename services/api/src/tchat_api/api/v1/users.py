"""Usuarios."""

from typing import Annotated

from fastapi import APIRouter, Depends, Query
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.api.deps import get_current_user
from tchat_api.api.serializers import user_public
from tchat_api.db.session import get_session
from tchat_api.models.user import User
from tchat_api.schemas.users import UserPublic, UserUpdate
from tchat_api.services import users as users_service

router = APIRouter(prefix="/users", tags=["usuarios"])


@router.get("/me", response_model=UserPublic, summary="Mi perfil")
async def me(user: Annotated[User, Depends(get_current_user)]) -> UserPublic:
    return user_public(user)


@router.patch("/me", response_model=UserPublic, summary="Actualiza mi perfil")
async def update_me(
    payload: UserUpdate,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> UserPublic:
    if payload.display_name is None:
        return user_public(user)

    updated = await users_service.update_display_name(session, user, payload.display_name.strip())
    return user_public(updated)


@router.get("/search", response_model=list[UserPublic], summary="Busca usuarios")
async def search(
    q: Annotated[str, Query(min_length=2, max_length=32)],
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> list[UserPublic]:
    found = await users_service.search_users(session, q)
    return [user_public(item) for item in found if item.id != user.id]


@router.get("/{username}", response_model=UserPublic, summary="Perfil público")
async def by_username(
    username: str,
    _: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> UserPublic:
    found = await users_service.get_by_username(session, username)
    return user_public(found)
