"""Amistades: solicitudes, aceptación y listado."""

import uuid
from datetime import UTC, datetime

from sqlalchemy import and_, or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.sql.elements import ColumnElement

from tchat_api.models.friendship import Friendship, FriendshipStatus
from tchat_api.models.user import User
from tchat_api.services import users as users_service
from tchat_api.services.errors import ConflictError, ForbiddenError, NotFoundError


def _pair_clause(first: uuid.UUID, second: uuid.UUID) -> ColumnElement[bool]:
    return or_(
        and_(Friendship.requester_id == first, Friendship.addressee_id == second),
        and_(Friendship.requester_id == second, Friendship.addressee_id == first),
    )


async def send_request(session: AsyncSession, me: User, username: str) -> Friendship:
    target = await users_service.get_by_username(session, username)

    if target.id == me.id:
        raise ConflictError("cannot_friend_yourself")

    existing = await session.scalar(select(Friendship).where(_pair_clause(me.id, target.id)))

    if existing is not None:
        if existing.status == FriendshipStatus.ACCEPTED:
            raise ConflictError("already_friends")
        if existing.status == FriendshipStatus.PENDING:
            raise ConflictError("request_pending")

        existing.requester_id = me.id
        existing.addressee_id = target.id
        existing.status = FriendshipStatus.PENDING
        existing.responded_at = None
        await session.commit()
        await session.refresh(existing)
        return existing

    friendship = Friendship(
        requester_id=me.id,
        addressee_id=target.id,
        status=FriendshipStatus.PENDING,
    )
    session.add(friendship)
    await session.commit()
    await session.refresh(friendship)
    return friendship


async def respond(
    session: AsyncSession,
    me: User,
    friendship_id: uuid.UUID,
    *,
    accept: bool,
) -> Friendship:
    friendship = await session.get(Friendship, friendship_id)
    if friendship is None:
        raise NotFoundError("friendship_not_found")
    if friendship.addressee_id != me.id:
        raise ForbiddenError("not_your_request")
    if friendship.status != FriendshipStatus.PENDING:
        raise ConflictError("request_already_answered")

    friendship.status = FriendshipStatus.ACCEPTED if accept else FriendshipStatus.REJECTED
    friendship.responded_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(friendship)
    return friendship


async def list_friends(session: AsyncSession, me: User) -> list[User]:
    result = await session.scalars(
        select(User)
        .join(
            Friendship,
            or_(
                and_(
                    Friendship.requester_id == me.id,
                    Friendship.addressee_id == User.id,
                ),
                and_(
                    Friendship.addressee_id == me.id,
                    Friendship.requester_id == User.id,
                ),
            ),
        )
        .where(Friendship.status == FriendshipStatus.ACCEPTED)
        .order_by(User.username)
    )
    return list(result)


async def list_requests(
    session: AsyncSession, me: User
) -> tuple[list[tuple[Friendship, User]], list[tuple[Friendship, User]]]:
    incoming_rows = await session.execute(
        select(Friendship, User)
        .join(User, User.id == Friendship.requester_id)
        .where(
            Friendship.addressee_id == me.id,
            Friendship.status == FriendshipStatus.PENDING,
        )
        .order_by(Friendship.created_at.desc())
    )
    outgoing_rows = await session.execute(
        select(Friendship, User)
        .join(User, User.id == Friendship.addressee_id)
        .where(
            Friendship.requester_id == me.id,
            Friendship.status == FriendshipStatus.PENDING,
        )
        .order_by(Friendship.created_at.desc())
    )

    incoming = [(row[0], row[1]) for row in incoming_rows.all()]
    outgoing = [(row[0], row[1]) for row in outgoing_rows.all()]
    return incoming, outgoing


async def remove_friend(session: AsyncSession, me: User, other_id: uuid.UUID) -> None:
    friendship = await session.scalar(
        select(Friendship).where(
            Friendship.status == FriendshipStatus.ACCEPTED,
            _pair_clause(me.id, other_id),
        )
    )
    if friendship is None:
        raise NotFoundError("friendship_not_found")

    await session.delete(friendship)
    await session.commit()


async def are_friends(session: AsyncSession, first_id: uuid.UUID, second_id: uuid.UUID) -> bool:
    friendship = await session.scalar(
        select(Friendship.id).where(
            Friendship.status == FriendshipStatus.ACCEPTED,
            _pair_clause(first_id, second_id),
        )
    )
    return friendship is not None
