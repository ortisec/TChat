"""Esquemas del flujo de autenticación del terminal."""

from typing import Literal

from pydantic import BaseModel, Field


class DeviceStartResponse(BaseModel):
    user_code: str = Field(description="Código corto que el usuario escribe o ve en el navegador")
    device_code: str = Field(description="Secreto que conserva el CLI para hacer polling")
    verification_url: str
    expires_in: int
    interval: int


class DeviceApproveRequest(BaseModel):
    user_code: str = Field(min_length=4, max_length=16)


class DeviceApproveResponse(BaseModel):
    status: Literal["approved"] = "approved"


class DeviceTokenRequest(BaseModel):
    device_code: str = Field(min_length=16, max_length=200)
    name: str | None = Field(default=None, max_length=64)


class RefreshTokenRequest(BaseModel):
    refresh_token: str = Field(min_length=16, max_length=200)


class TokenResponse(BaseModel):
    access_token: str
    refresh_token: str
    token_type: Literal["Bearer"] = "Bearer"
    expires_in: int
