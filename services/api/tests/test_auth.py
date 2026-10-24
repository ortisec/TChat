"""Autenticación: sincronización, tokens y device flow."""

from datetime import UTC, datetime, timedelta

from httpx import AsyncClient
from sqlalchemy import select, update

from tchat_api.api.deps import get_identity
from tchat_api.core.config import settings
from tchat_api.core.security import create_access_token
from tchat_api.db.session import session_factory
from tchat_api.main import app
from tchat_api.models.device import DeviceCode
from tchat_api.models.user import User


async def test_sync_creates_user(client: AsyncClient, act_as) -> None:
    act_as("firebase-ana", email="ana@example.com", name="Ana")

    response = await client.post("/api/v1/auth/sync")

    assert response.status_code == 200
    body = response.json()
    assert body["username"] == "ana"
    assert body["email"] == "ana@example.com"
    assert body["display_name"] == "Ana"


async def test_sync_generates_unique_usernames(client: AsyncClient, act_as) -> None:
    act_as("firebase-1", email="one@example.com", name="Ana")
    first = (await client.post("/api/v1/auth/sync")).json()

    act_as("firebase-2", email="two@example.com", name="Ana")
    second = (await client.post("/api/v1/auth/sync")).json()

    assert first["username"] == "ana"
    assert second["username"] == "ana-2"


async def test_sync_is_idempotent(client: AsyncClient, act_as) -> None:
    act_as("firebase-ana", email="ana@example.com", name="Ana")
    first = (await client.post("/api/v1/auth/sync")).json()
    second = (await client.post("/api/v1/auth/sync")).json()

    assert first["id"] == second["id"]


async def test_access_token_authorizes_requests(client: AsyncClient, act_as) -> None:
    act_as("firebase-ana", email="ana@example.com", name="Ana")
    await client.post("/api/v1/auth/sync")

    async with session_factory() as session:
        user = await session.scalar(select(User).where(User.firebase_uid == "firebase-ana"))
        assert user is not None
        token = create_access_token(user)

    app.dependency_overrides.pop(get_identity, None)

    response = await client.get("/api/v1/users/me", headers={"Authorization": f"Bearer {token}"})

    assert response.status_code == 200
    assert response.json()["username"] == "ana"


async def test_missing_token_is_rejected(client: AsyncClient) -> None:
    response = await client.get("/api/v1/users/me")

    assert response.status_code == 401
    assert response.json()["detail"] == "missing_token"


async def test_device_flow_end_to_end(client: AsyncClient, act_as) -> None:
    start = await client.post("/api/v1/auth/device/start")
    assert start.status_code == 201
    payload = start.json()
    assert payload["user_code"].count("-") == 1
    assert payload["verification_url"].endswith(f"/cli-auth?code={payload['user_code']}")

    pending = await client.post(
        "/api/v1/auth/device/token",
        json={"device_code": payload["device_code"]},
    )
    assert pending.status_code == 400
    assert pending.json()["detail"] == "authorization_pending"

    act_as("firebase-cli", email="cli@example.com", name="Cli User")
    approve = await client.post(
        "/api/v1/auth/device/approve", json={"user_code": payload["user_code"]}
    )
    assert approve.status_code == 200
    assert approve.json()["status"] == "approved"

    tokens = await client.post(
        "/api/v1/auth/device/token",
        json={"device_code": payload["device_code"], "name": "test"},
    )
    assert tokens.status_code == 200
    access = tokens.json()["access_token"]
    refresh = tokens.json()["refresh_token"]

    app.dependency_overrides.pop(get_identity, None)
    me = await client.get("/api/v1/users/me", headers={"Authorization": f"Bearer {access}"})
    assert me.status_code == 200
    assert me.json()["username"] == "cli-user"

    rotated = await client.post("/api/v1/auth/token", json={"refresh_token": refresh})
    assert rotated.status_code == 200
    new_refresh = rotated.json()["refresh_token"]
    assert new_refresh != refresh

    reused = await client.post("/api/v1/auth/token", json={"refresh_token": refresh})
    assert reused.status_code == 401

    logout = await client.post(
        "/api/v1/auth/logout",
        json={"refresh_token": new_refresh},
        headers={"Authorization": f"Bearer {access}"},
    )
    assert logout.status_code == 204

    after_logout = await client.post("/api/v1/auth/token", json={"refresh_token": new_refresh})
    assert after_logout.status_code == 401


async def test_device_code_expires(client: AsyncClient) -> None:
    start = await client.post("/api/v1/auth/device/start")
    payload = start.json()

    async with session_factory() as session:
        await session.execute(
            update(DeviceCode)
            .where(DeviceCode.user_code == payload["user_code"])
            .values(expires_at=datetime.now(UTC) - timedelta(seconds=1))
        )
        await session.commit()

    poll = await client.post(
        "/api/v1/auth/device/token",
        json={"device_code": payload["device_code"]},
    )

    assert poll.status_code == 400
    assert poll.json()["detail"] == "expired_token"


async def test_polling_too_fast(client: AsyncClient, monkeypatch) -> None:
    monkeypatch.setattr(settings, "device_poll_interval_seconds", 5)

    start = await client.post("/api/v1/auth/device/start")
    device_code = start.json()["device_code"]

    first = await client.post("/api/v1/auth/device/token", json={"device_code": device_code})
    second = await client.post("/api/v1/auth/device/token", json={"device_code": device_code})

    assert first.json()["detail"] == "authorization_pending"
    assert second.status_code == 400
    assert second.json()["detail"] == "slow_down"
