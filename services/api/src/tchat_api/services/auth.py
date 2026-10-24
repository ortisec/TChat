"""Flujo de autorización de dispositivos y tokens de sesión."""

import hashlib
import secrets
from datetime import UTC, datetime, timedelta

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.core.config import settings
from tchat_api.core.security import create_access_token
from tchat_api.models.device import DeviceCode, DeviceCodeStatus, DeviceToken
from tchat_api.models.user import User
from tchat_api.services.errors import ConflictError, NotFoundError, ServiceError

USER_CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"


def hash_token(token: str) -> str:
    return hashlib.sha256(token.encode("utf-8")).hexdigest()


def _generate_user_code() -> str:
    characters = "".join(secrets.choice(USER_CODE_ALPHABET) for _ in range(8))
    return f"{characters[:4]}-{characters[4:]}"


async def start_device(session: AsyncSession) -> tuple[DeviceCode, str]:
    """Crea la solicitud y devuelve también el device code en claro (solo se guarda el hash)."""
    device_code = secrets.token_urlsafe(32)
    user_code = _generate_user_code()

    for _ in range(5):
        existing = await session.scalar(
            select(DeviceCode.id).where(DeviceCode.user_code == user_code)
        )
        if existing is None:
            break
        user_code = _generate_user_code()

    now = datetime.now(UTC)
    code = DeviceCode(
        user_code=user_code,
        device_code_hash=hash_token(device_code),
        status=DeviceCodeStatus.PENDING,
        expires_at=now + timedelta(seconds=settings.device_code_ttl_seconds),
    )
    session.add(code)
    await session.commit()
    await session.refresh(code)
    return code, device_code


async def approve(session: AsyncSession, user: User, user_code: str) -> None:
    code = await session.scalar(
        select(DeviceCode).where(DeviceCode.user_code == user_code.strip().upper())
    )
    if code is None:
        raise NotFoundError("invalid_code")

    now = datetime.now(UTC)
    if code.expires_at <= now:
        raise ServiceError("expired_code")
    if code.status != DeviceCodeStatus.PENDING:
        raise ConflictError("code_already_used")

    code.status = DeviceCodeStatus.APPROVED
    code.user_id = user.id
    code.approved_at = now
    await session.commit()


async def poll(
    session: AsyncSession, device_code: str, *, name: str | None = None
) -> tuple[str, str]:
    """Devuelve (access_token, refresh_token) o lanza el error del protocolo."""
    code = await session.scalar(
        select(DeviceCode).where(DeviceCode.device_code_hash == hash_token(device_code))
    )
    if code is None or code.status == DeviceCodeStatus.CONSUMED:
        raise ServiceError("invalid_grant")

    now = datetime.now(UTC)
    if code.expires_at <= now:
        raise ServiceError("expired_token")

    if code.status == DeviceCodeStatus.PENDING:
        if (
            code.last_polled_at is not None
            and (now - code.last_polled_at).total_seconds() < settings.device_poll_interval_seconds
        ):
            code.last_polled_at = now
            await session.commit()
            raise ServiceError("slow_down")

        code.last_polled_at = now
        await session.commit()
        raise ServiceError("authorization_pending")

    if code.user_id is None:
        raise ServiceError("invalid_grant")

    user = await session.get(User, code.user_id)
    if user is None:
        raise ServiceError("invalid_grant")

    refresh_token = secrets.token_urlsafe(48)
    session.add(
        DeviceToken(
            user_id=user.id,
            token_hash=hash_token(refresh_token),
            name=(name or "terminal")[:64],
        )
    )
    code.status = DeviceCodeStatus.CONSUMED
    code.consumed_at = now
    await session.commit()

    return create_access_token(user), refresh_token


async def refresh(session: AsyncSession, refresh_token: str) -> tuple[str, str]:
    """Rota el refresh token y emite un acceso nuevo."""
    row = await session.scalar(
        select(DeviceToken).where(DeviceToken.token_hash == hash_token(refresh_token))
    )
    if row is None or row.revoked_at is not None:
        raise ServiceError("invalid_grant", status_code=401)

    user = await session.get(User, row.user_id)
    if user is None:
        raise ServiceError("invalid_grant", status_code=401)

    now = datetime.now(UTC)
    row.revoked_at = now
    row.last_used_at = now

    new_refresh = secrets.token_urlsafe(48)
    session.add(
        DeviceToken(
            user_id=user.id,
            token_hash=hash_token(new_refresh),
            name=row.name,
        )
    )
    await session.commit()

    return create_access_token(user), new_refresh


async def logout(session: AsyncSession, user: User, refresh_token: str) -> None:
    row = await session.scalar(
        select(DeviceToken).where(
            DeviceToken.token_hash == hash_token(refresh_token),
            DeviceToken.user_id == user.id,
        )
    )
    if row is None or row.revoked_at is not None:
        return

    row.revoked_at = datetime.now(UTC)
    await session.commit()
