"""Conversaciones: creación, consulta y estado de lectura."""

import uuid
from dataclasses import dataclass
from datetime import UTC, datetime

from sqlalchemy import and_, func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import aliased

from tchat_api.models.conversation import Conversation, ConversationMember, ConversationType
from tchat_api.models.message import Message
from tchat_api.models.user import User
from tchat_api.services import friends as friends_service
from tchat_api.services import users as users_service
from tchat_api.services.errors import ConflictError, ForbiddenError, NotFoundError


@dataclass(slots=True)
class ConversationBundle:
    conversation: Conversation
    members: list[User]
    last_message: Message | None
    last_sender: User | None
    unread_count: int


async def require_membership(
    session: AsyncSession, conversation_id: uuid.UUID, user_id: uuid.UUID
) -> ConversationMember:
    member = await session.get(ConversationMember, (conversation_id, user_id))
    if member is None:
        # No revelamos si la conversación existe o simplemente no pertenece al usuario.
        raise NotFoundError("conversation_not_found")
    return member


async def _find_direct(
    session: AsyncSession, first_id: uuid.UUID, second_id: uuid.UUID
) -> Conversation | None:
    first_member = aliased(ConversationMember)
    second_member = aliased(ConversationMember)

    return await session.scalar(
        select(Conversation)
        .join(first_member, first_member.conversation_id == Conversation.id)
        .join(second_member, second_member.conversation_id == Conversation.id)
        .where(
            Conversation.type == ConversationType.DIRECT,
            first_member.user_id == first_id,
            second_member.user_id == second_id,
        )
        .limit(1)
    )


async def create_direct(session: AsyncSession, me: User, other_id: uuid.UUID) -> Conversation:
    if other_id == me.id:
        raise ConflictError("cannot_message_yourself")

    await users_service.get_by_id(session, other_id)

    if not await friends_service.are_friends(session, me.id, other_id):
        raise ForbiddenError("not_friends")

    existing = await _find_direct(session, me.id, other_id)
    if existing is not None:
        return existing

    conversation = Conversation(type=ConversationType.DIRECT, created_by=me.id)
    session.add(conversation)
    await session.flush()

    session.add_all(
        [
            ConversationMember(conversation_id=conversation.id, user_id=me.id),
            ConversationMember(conversation_id=conversation.id, user_id=other_id),
        ]
    )
    await session.commit()
    await session.refresh(conversation)
    return conversation


async def create_group(
    session: AsyncSession,
    me: User,
    title: str,
    member_ids: list[uuid.UUID],
) -> Conversation:
    unique_ids = [user_id for user_id in dict.fromkeys(member_ids) if user_id != me.id]
    if not unique_ids:
        raise ConflictError("group_needs_members")

    for member_id in unique_ids:
        await users_service.get_by_id(session, member_id)
        if not await friends_service.are_friends(session, me.id, member_id):
            raise ForbiddenError("not_friends")

    conversation = Conversation(
        type=ConversationType.GROUP,
        title=title.strip(),
        created_by=me.id,
    )
    session.add(conversation)
    await session.flush()

    session.add_all(
        [
            ConversationMember(conversation_id=conversation.id, user_id=user_id)
            for user_id in [me.id, *unique_ids]
        ]
    )
    await session.commit()
    await session.refresh(conversation)
    return conversation


async def get_bundle(
    session: AsyncSession, conversation_id: uuid.UUID, viewer_id: uuid.UUID
) -> ConversationBundle:
    member = await require_membership(session, conversation_id, viewer_id)
    conversation = await session.get(Conversation, conversation_id)
    if conversation is None:
        raise NotFoundError("conversation_not_found")

    members = await _members_for(session, [conversation_id])
    last_row = await session.execute(
        select(Message, User)
        .join(User, User.id == Message.sender_id)
        .where(Message.conversation_id == conversation_id)
        .order_by(Message.created_at.desc(), Message.id.desc())
        .limit(1)
    )
    row = last_row.first()
    last_message = row[0] if row else None
    last_sender = row[1] if row else None

    unread = await _unread_count(session, conversation_id, viewer_id, member)

    return ConversationBundle(
        conversation=conversation,
        members=members.get(conversation_id, []),
        last_message=last_message,
        last_sender=last_sender,
        unread_count=unread,
    )


async def list_bundles(
    session: AsyncSession, me: User, *, limit: int = 50
) -> list[ConversationBundle]:
    conversations = list(
        await session.scalars(
            select(Conversation)
            .join(
                ConversationMember,
                ConversationMember.conversation_id == Conversation.id,
            )
            .where(ConversationMember.user_id == me.id)
            .order_by(Conversation.updated_at.desc())
            .limit(limit)
        )
    )
    if not conversations:
        return []

    ids = [conversation.id for conversation in conversations]
    members = await _members_for(session, ids)

    message_rows = await session.execute(
        select(Message, User)
        .join(User, User.id == Message.sender_id)
        .where(Message.conversation_id.in_(ids))
        .order_by(Message.created_at.desc(), Message.id.desc())
    )
    latest: dict[uuid.UUID, tuple[Message, User]] = {}
    for message, sender in message_rows.all():
        latest.setdefault(message.conversation_id, (message, sender))

    unread_rows = await session.execute(
        select(Message.conversation_id, func.count())
        .join(
            ConversationMember,
            and_(
                ConversationMember.conversation_id == Message.conversation_id,
                ConversationMember.user_id == me.id,
            ),
        )
        .where(
            Message.conversation_id.in_(ids),
            Message.sender_id != me.id,
            or_(
                ConversationMember.last_read_at.is_(None),
                Message.created_at > ConversationMember.last_read_at,
            ),
        )
        .group_by(Message.conversation_id)
    )
    unread = {conversation_id: count for conversation_id, count in unread_rows.all()}

    bundles: list[ConversationBundle] = []
    for conversation in conversations:
        pair = latest.get(conversation.id)
        bundles.append(
            ConversationBundle(
                conversation=conversation,
                members=members.get(conversation.id, []),
                last_message=pair[0] if pair else None,
                last_sender=pair[1] if pair else None,
                unread_count=unread.get(conversation.id, 0),
            )
        )
    return bundles


async def _members_for(
    session: AsyncSession, conversation_ids: list[uuid.UUID]
) -> dict[uuid.UUID, list[User]]:
    rows = await session.execute(
        select(ConversationMember.conversation_id, User)
        .join(User, User.id == ConversationMember.user_id)
        .where(ConversationMember.conversation_id.in_(conversation_ids))
        .order_by(User.username)
    )
    grouped: dict[uuid.UUID, list[User]] = {}
    for conversation_id, user in rows.all():
        grouped.setdefault(conversation_id, []).append(user)
    return grouped


async def _unread_count(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    viewer_id: uuid.UUID,
    member: ConversationMember,
) -> int:
    conditions = [
        Message.conversation_id == conversation_id,
        Message.sender_id != viewer_id,
    ]
    if member.last_read_at is not None:
        conditions.append(Message.created_at > member.last_read_at)

    count = await session.scalar(select(func.count()).select_from(Message).where(*conditions))
    return int(count or 0)


async def member_ids(session: AsyncSession, conversation_id: uuid.UUID) -> list[uuid.UUID]:
    result = await session.scalars(
        select(ConversationMember.user_id).where(
            ConversationMember.conversation_id == conversation_id
        )
    )
    return list(result)


async def touch(session: AsyncSession, conversation_id: uuid.UUID) -> None:
    conversation = await session.get(Conversation, conversation_id)
    if conversation is not None:
        conversation.updated_at = datetime.now(UTC)
