"""Identidad: verificación de tokens de Firebase y emisión de tokens propios."""

from dataclasses import dataclass
from datetime import UTC, datetime, timedelta

import jwt
from firebase_admin import auth as firebase_auth

from tchat_api.core.config import settings
from tchat_api.core.firebase import get_firebase_app
from tchat_api.models.user import User


class AuthError(Exception):
    """Token ausente, inválido o expirado."""

    def __init__(self, code: str = "invalid_token") -> None:
        super().__init__(code)
        self.code = code


@dataclass(frozen=True, slots=True)
class Identity:
    uid: str
    email: str | None = None
    display_name: str | None = None
    provider: str = "password"


def create_access_token(user: User, *, ttl_seconds: int | None = None) -> str:
    """Firma un token de acceso para el cliente de terminal."""
    now = datetime.now(UTC)
    ttl = settings.access_token_ttl_seconds if ttl_seconds is None else ttl_seconds
    claims = {
        "iss": settings.auth_jwt_issuer,
        "sub": str(user.id),
        "fuid": user.firebase_uid,
        "email": user.email,
        "name": user.display_name,
        "typ": "access",
        "iat": now,
        "exp": now + timedelta(seconds=ttl),
    }
    return jwt.encode(claims, settings.auth_jwt_secret, algorithm="HS256")


def decode_access_token(token: str) -> Identity:
    try:
        payload = jwt.decode(
            token,
            settings.auth_jwt_secret,
            algorithms=["HS256"],
            issuer=settings.auth_jwt_issuer,
            options={"require": ["exp", "iat", "sub", "fuid"]},
        )
    except jwt.InvalidTokenError as exc:
        raise AuthError() from exc

    if payload.get("typ") != "access":
        raise AuthError()

    return Identity(
        uid=str(payload["fuid"]),
        email=payload.get("email"),
        display_name=payload.get("name"),
        provider="device",
    )


def verify_firebase_token(token: str) -> Identity:
    try:
        claims = firebase_auth.verify_id_token(
            token,
            app=get_firebase_app(),
            check_revoked=False,
        )
    except Exception as exc:
        raise AuthError() from exc

    provider = claims.get("firebase", {}).get("sign_in_provider", "password")
    return Identity(
        uid=str(claims["uid"]),
        email=claims.get("email"),
        display_name=claims.get("name"),
        provider="google" if provider == "google.com" else "password",
    )


def resolve_identity(token: str) -> Identity:
    """Acepta tokens propios (CLI) y tokens de Firebase (web)."""
    try:
        return decode_access_token(token)
    except AuthError:
        return verify_firebase_token(token)
