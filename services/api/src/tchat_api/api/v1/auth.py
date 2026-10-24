"""Autenticación: sincronización, device flow y tokens."""

from typing import Annotated

from fastapi import APIRouter, Depends, Response, status
from sqlalchemy.ext.asyncio import AsyncSession

from tchat_api.api.deps import get_current_user, get_identity
from tchat_api.api.serializers import user_public
from tchat_api.core.config import settings
from tchat_api.core.security import Identity
from tchat_api.db.session import get_session
from tchat_api.models.user import User
from tchat_api.schemas.auth import (
    DeviceApproveRequest,
    DeviceApproveResponse,
    DeviceStartResponse,
    DeviceTokenRequest,
    RefreshTokenRequest,
    TokenResponse,
)
from tchat_api.schemas.users import UserPublic
from tchat_api.services import auth as auth_service
from tchat_api.services import users as users_service

router = APIRouter(prefix="/auth", tags=["autenticación"])


@router.post("/sync", response_model=UserPublic, summary="Crea o actualiza el usuario")
async def sync(
    identity: Annotated[Identity, Depends(get_identity)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> UserPublic:
    user = await users_service.sync_user(session, identity)
    return user_public(user)


@router.post(
    "/device/start",
    response_model=DeviceStartResponse,
    status_code=status.HTTP_201_CREATED,
    summary="Inicia el flujo de autorización del terminal",
)
async def device_start(
    session: Annotated[AsyncSession, Depends(get_session)],
) -> DeviceStartResponse:
    code, device_code = await auth_service.start_device(session)
    return DeviceStartResponse(
        user_code=code.user_code,
        device_code=device_code,
        verification_url=f"{settings.public_account_url}/cli-auth?code={code.user_code}",
        expires_in=settings.device_code_ttl_seconds,
        interval=settings.device_poll_interval_seconds,
    )


@router.post(
    "/device/approve",
    response_model=DeviceApproveResponse,
    summary="Aprueba el código desde la web",
)
async def device_approve(
    payload: DeviceApproveRequest,
    identity: Annotated[Identity, Depends(get_identity)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> DeviceApproveResponse:
    user = await users_service.sync_user(session, identity)
    await auth_service.approve(session, user, payload.user_code)
    return DeviceApproveResponse()


@router.post(
    "/device/token",
    response_model=TokenResponse,
    summary="Canjea el device code por tokens",
)
async def device_token(
    payload: DeviceTokenRequest,
    session: Annotated[AsyncSession, Depends(get_session)],
) -> TokenResponse:
    access_token, refresh_token = await auth_service.poll(
        session, payload.device_code, name=payload.name
    )
    return TokenResponse(
        access_token=access_token,
        refresh_token=refresh_token,
        expires_in=settings.access_token_ttl_seconds,
    )


@router.post("/token", response_model=TokenResponse, summary="Renueva el token de acceso")
async def refresh_token(
    payload: RefreshTokenRequest,
    session: Annotated[AsyncSession, Depends(get_session)],
) -> TokenResponse:
    access_token, new_refresh = await auth_service.refresh(session, payload.refresh_token)
    return TokenResponse(
        access_token=access_token,
        refresh_token=new_refresh,
        expires_in=settings.access_token_ttl_seconds,
    )


@router.post(
    "/logout",
    status_code=status.HTTP_204_NO_CONTENT,
    response_class=Response,
    summary="Revoca el refresh token del dispositivo",
)
async def logout(
    payload: RefreshTokenRequest,
    user: Annotated[User, Depends(get_current_user)],
    session: Annotated[AsyncSession, Depends(get_session)],
) -> Response:
    await auth_service.logout(session, user, payload.refresh_token)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
