"""Conversaciones y mensajes."""

import uuid
from typing import Annotated

from fastapi import APIRouter, Depends, Query, Response, status
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.api.deps import get_current_user
from tchat_api.api.serializers import conversation_public, message_public
from tchat_api.db.session import get_session
from tchat_api.models.user import User
from tchat_api.schemas.conversations import (
    ConversationCreate,
    ConversationPublic,
    DirectConversationCreate,
    ReadRequest,
)
from tchat_api.schemas.messages import MessageCreate, MessagePublic
from tchat_api.services import conversations as conversations_service
from tchat_api.services import messages as messages_service

router = APIRouter(prefix="/conversations", tags=["conversaciones"])


@router.get("", response_model=list[ConversationPublic], summary="Mis conversaciones")
async def list_conversations(
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> list[ConversationPublic]:
    bundles = await conversations_service.list_bundles(session, user)
    return [conversation_public(bundle) for bundle in bundles]


@router.post(
    "",
    response_model=ConversationPublic,
    status_code=status.HTTP_201_CREATED,
    summary="Crea una conversación",
)
async def create_conversation(
    payload: ConversationCreate,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> ConversationPublic:
    if isinstance(payload, DirectConversationCreate):
        conversation = await conversations_service.create_direct(session, user, payload.user_id)
    else:
        conversation = await conversations_service.create_group(
            session, user, payload.title, payload.member_ids
        )

    bundle = await conversations_service.get_bundle(session, conversation.id, user.id)
    return conversation_public(bundle)


@router.get(
    "/{conversation_id}",
    response_model=ConversationPublic,
    summary="Detalle de una conversación",
)
async def get_conversation(
    conversation_id: uuid.UUID,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> ConversationPublic:
    bundle = await conversations_service.get_bundle(session, conversation_id, user.id)
    return conversation_public(bundle)


@router.get(
    "/{conversation_id}/messages",
    response_model=list[MessagePublic],
    summary="Historial de mensajes",
)
async def list_messages(
    conversation_id: uuid.UUID,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
    before_id: uuid.UUID | None = None,
    limit: Annotated[int, Query(ge=1, le=100)] = 50,
) -> list[MessagePublic]:
    rows = await messages_service.list_messages(
        session,
        user,
        conversation_id,
        before_id=before_id,
        limit=limit,
    )
    return [message_public(message, sender) for message, sender in rows]


@router.post(
    "/{conversation_id}/messages",
    response_model=MessagePublic,
    status_code=status.HTTP_201_CREATED,
    summary="Envía un mensaje (alternativa REST al WebSocket)",
)
async def send_message(
    conversation_id: uuid.UUID,
    payload: MessageCreate,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> MessagePublic:
    message, sender = await messages_service.send(session, user, conversation_id, payload.body)
    return message_public(message, sender)


@router.post(
    "/{conversation_id}/read",
    status_code=status.HTTP_204_NO_CONTENT,
    response_class=Response,
    summary="Marca la conversación como leída",
)
async def mark_read(
    conversation_id: uuid.UUID,
    payload: ReadRequest,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> Response:
    await messages_service.mark_read(session, user, conversation_id, payload.message_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
