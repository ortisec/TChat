"""Reglas de usuarios y nombres de usuario."""

import re
import unicodedata
import uuid

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.core.security import Identity
from tchat_api.models.user import User
from tchat_api.services.errors import NotFoundError


def slugify(value: str) -> str:
    normalized = unicodedata.normalize("NFKD", value)
    ascii_value = normalized.encode("ascii", "ignore").decode("ascii")
    cleaned = re.sub(r"[^a-zA-Z0-9]+", "-", ascii_value).strip("-").lower()
    return cleaned[:20] or "user"


async def _username_taken(session: AsyncSession, username: str) -> bool:
    return (await session.scalar(select(User.id).where(User.username == username))) is not None


async def _generate_username(session: AsyncSession, identity: Identity) -> str:
    base = slugify(identity.display_name or identity.email or identity.uid)
    candidate = base

    for suffix in range(2, 100):
        if not await _username_taken(session, candidate):
            return candidate
        candidate = f"{base[:17]}-{suffix}"

    return f"user-{uuid.uuid4().hex[:8]}"


async def sync_user(session: AsyncSession, identity: Identity) -> User:
    """Crea o actualiza el usuario a partir de la identidad verificada."""
    user = await session.scalar(select(User).where(User.firebase_uid == identity.uid))

    if user is None:
        username = await _generate_username(session, identity)
        user = User(
            firebase_uid=identity.uid,
            username=username,
            email=identity.email,
            display_name=identity.display_name,
        )
        session.add(user)
        try:
            await session.commit()
        except IntegrityError:
            # Carrera con otra petición del mismo usuario o nombre ya tomado.
            await session.rollback()
            user = await session.scalar(select(User).where(User.firebase_uid == identity.uid))
            if user is None:
                raise
        else:
            await session.refresh(user)
            return user

    changed = False
    if identity.email and user.email != identity.email:
        user.email = identity.email
        changed = True
    if identity.display_name and user.display_name != identity.display_name:
        user.display_name = identity.display_name
        changed = True

    if changed:
        await session.commit()
        await session.refresh(user)

    return user


async def get_by_username(session: AsyncSession, username: str) -> User:
    user = await session.scalar(select(User).where(User.username == username.strip().lower()))
    if user is None:
        raise NotFoundError("user_not_found")
    return user


async def get_by_id(session: AsyncSession, user_id: uuid.UUID) -> User:
    user = await session.get(User, user_id)
    if user is None:
        raise NotFoundError("user_not_found")
    return user


async def search_users(session: AsyncSession, query: str, *, limit: int = 10) -> list[User]:
    pattern = f"{query.strip().lower()}%"
    result = await session.scalars(
        select(User).where(User.username.like(pattern)).order_by(User.username).limit(limit)
    )
    return list(result)


async def update_display_name(session: AsyncSession, user: User, display_name: str) -> User:
    user.display_name = display_name
    await session.commit()
    await session.refresh(user)
    return user
