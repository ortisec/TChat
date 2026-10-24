"""Utilidades compartidas por los tests."""

from collections.abc import Callable

from httpx import AsyncClient

from tchat_api.core.security import Identity


async def register(
    client: AsyncClient,
    act_as: Callable[..., Identity],
    uid: str,
    *,
    name: str,
) -> dict[str, object]:
    act_as(uid, name=name)
    response = await client.post("/api/v1/auth/sync")
    assert response.status_code == 200, response.text
    return response.json()


async def befriend(
    client: AsyncClient,
    act_as: Callable[..., Identity],
    first_uid: str,
    first_name: str,
    second_uid: str,
    second_name: str,
) -> tuple[dict[str, object], dict[str, object]]:
    first = await register(client, act_as, first_uid, name=first_name)
    second = await register(client, act_as, second_uid, name=second_name)

    act_as(first_uid)
    sent = await client.post("/api/v1/friends/requests", json={"username": second["username"]})
    assert sent.status_code == 201, sent.text

    act_as(second_uid)
    accepted = await client.post(f"/api/v1/friends/requests/{sent.json()['id']}/accept")
    assert accepted.status_code == 200, accepted.text

    return first, second
