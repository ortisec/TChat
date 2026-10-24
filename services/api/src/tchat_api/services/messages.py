"""Mensajes: envío, historial y lectura."""

import uuid

from sqlalchemy import and_, or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.models.message import Message
from tchat_api.models.user import User
from tchat_api.services import conversations as conversations_service
from tchat_api.services.errors import NotFoundError, ServiceError


async def send(
    session: AsyncSession, me: User, conversation_id: uuid.UUID, body: str
) -> tuple[Message, User]:
    text = body.strip()
    if not text:
        raise ServiceError("empty_message")

    await conversations_service.require_membership(session, conversation_id, me.id)

    message = Message(
        conversation_id=conversation_id,
        sender_id=me.id,
        body=text,
    )
    session.add(message)
    await conversations_service.touch(session, conversation_id)
    await session.commit()
    await session.refresh(message)

    return message, me


async def list_messages(
    session: AsyncSession,
    me: User,
    conversation_id: uuid.UUID,
    *,
    before_id: uuid.UUID | None = None,
    limit: int = 50,
) -> list[tuple[Message, User]]:
    await conversations_service.require_membership(session, conversation_id, me.id)

    statement = (
        select(Message, User)
        .join(User, User.id == Message.sender_id)
        .where(Message.conversation_id == conversation_id)
    )

    if before_id is not None:
        anchor = await session.get(Message, before_id)
        if anchor is None or anchor.conversation_id != conversation_id:
            raise NotFoundError("message_not_found")
        statement = statement.where(
            or_(
                Message.created_at < anchor.created_at,
                and_(
                    Message.created_at == anchor.created_at,
                    Message.id < anchor.id,
                ),
            )
        )

    statement = statement.order_by(Message.created_at.desc(), Message.id.desc()).limit(limit)

    rows = await session.execute(statement)
    messages = [(row[0], row[1]) for row in rows.all()]
    messages.reverse()
    return messages


async def mark_read(
    session: AsyncSession,
    me: User,
    conversation_id: uuid.UUID,
    message_id: uuid.UUID,
) -> None:
    member = await conversations_service.require_membership(session, conversation_id, me.id)

    message = await session.get(Message, message_id)
    if message is None or message.conversation_id != conversation_id:
        raise NotFoundError("message_not_found")

    member.last_read_at = message.created_at
    await session.commit()
