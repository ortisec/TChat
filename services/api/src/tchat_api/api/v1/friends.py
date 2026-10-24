"""Amistades."""

import uuid
from typing import Annotated

from fastapi import APIRouter, Depends, Response, status
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.api.deps import get_current_user
from tchat_api.api.serializers import friendship_public, user_public
from tchat_api.db.session import get_session
from tchat_api.models.user import User
from tchat_api.schemas.friends import (
    FriendRequestCreate,
    FriendRequestsPublic,
    FriendshipPublic,
)
from tchat_api.schemas.users import UserPublic
from tchat_api.services import friends as friends_service
from tchat_api.services import users as users_service

router = APIRouter(prefix="/friends", tags=["amistades"])


@router.get("", response_model=list[UserPublic], summary="Mis amigos")
async def list_friends(
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> list[UserPublic]:
    friends = await friends_service.list_friends(session, user)
    return [user_public(friend) for friend in friends]


@router.get(
    "/requests",
    response_model=FriendRequestsPublic,
    summary="Solicitudes pendientes",
)
async def list_requests(
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> FriendRequestsPublic:
    incoming, outgoing = await friends_service.list_requests(session, user)
    return FriendRequestsPublic(
        incoming=[
            friendship_public(friendship, counterpart) for friendship, counterpart in incoming
        ],
        outgoing=[
            friendship_public(friendship, counterpart) for friendship, counterpart in outgoing
        ],
    )


@router.post(
    "/requests",
    response_model=FriendshipPublic,
    status_code=status.HTTP_201_CREATED,
    summary="Envía una solicitud de amistad",
)
async def send_request(
    payload: FriendRequestCreate,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> FriendshipPublic:
    friendship = await friends_service.send_request(session, user, payload.username)
    counterpart = await users_service.get_by_id(session, friendship.addressee_id)
    return friendship_public(friendship, counterpart)


@router.post(
    "/requests/{friendship_id}/accept",
    response_model=FriendshipPublic,
    summary="Acepta una solicitud",
)
async def accept_request(
    friendship_id: uuid.UUID,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> FriendshipPublic:
    friendship = await friends_service.respond(session, user, friendship_id, accept=True)
    counterpart = await users_service.get_by_id(session, friendship.requester_id)
    return friendship_public(friendship, counterpart)


@router.post(
    "/requests/{friendship_id}/reject",
    response_model=FriendshipPublic,
    summary="Rechaza una solicitud",
)
async def reject_request(
    friendship_id: uuid.UUID,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> FriendshipPublic:
    friendship = await friends_service.respond(session, user, friendship_id, accept=False)
    counterpart = await users_service.get_by_id(session, friendship.requester_id)
    return friendship_public(friendship, counterpart)


@router.delete(
    "/{user_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    response_class=Response,
    summary="Elimina una amistad",
)
async def remove_friend(
    user_id: uuid.UUID,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> Response:
    await friends_service.remove_friend(session, user, user_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
