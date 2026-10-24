"""Dependencias compartidas de la capa HTTP."""

from typing import Annotated

from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.core.security import AuthError, Identity, resolve_identity
from tchat_api.db.session import get_session
from tchat_api.models.user import User

bearer_scheme = HTTPBearer(auto_error=False)


def get_identity(
    credentials: Annotated[HTTPAuthorizationCredentials | None, Depends(bearer_scheme)],
) -> Identity:
    if credentials is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="missing_token",
            headers={"WWW-Authenticate": "Bearer"},
        )

    try:
        return resolve_identity(credentials.credentials)
    except AuthError as exc:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=exc.code,
            headers={"WWW-Authenticate": "Bearer"},
        ) from exc


async def get_current_user(
    identity: Annotated[Identity, Depends(get_identity)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> User:
    user = await session.scalar(select(User).where(User.firebase_uid == identity.uid))
    if user is None:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="user_not_synced")
    return user
