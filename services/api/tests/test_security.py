"""Tokens propios."""

import pytest

from tchat_api.core.ids import uuid7
from tchat_api.core.security import AuthError, create_access_token, decode_access_token
from tchat_api.models.user import User


def _user() -> User:
    return User(
        id=uuid7(),
        firebase_uid="uid-ana",
        username="ana",
        email="ana@example.com",
        display_name="Ana",
    )


def test_roundtrip() -> None:
    identity = decode_access_token(create_access_token(_user()))

    assert identity.uid == "uid-ana"
    assert identity.email == "ana@example.com"
    assert identity.provider == "device"


def test_expired_token_is_rejected() -> None:
    with pytest.raises(AuthError):
        decode_access_token(create_access_token(_user(), ttl_seconds=-10))


def test_tampered_token_is_rejected() -> None:
    token = create_access_token(_user())

    with pytest.raises(AuthError):
        decode_access_token(f"{token}x")
