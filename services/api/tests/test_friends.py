"""Amistades."""

import uuid

from httpx import AsyncClient

from tchat_api.realtime.manager import manager
from tests.helpers import befriend, register


class FakeSocket:
    def __init__(self) -> None:
        self.messages: list[str] = []

    async def send_text(self, message: str) -> None:
        self.messages.append(message)


async def test_friend_request_and_accept(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    bruno = await register(client, act_as, "bruno", name="Bruno")
    assert bruno["username"] == "bruno"

    act_as("ana")
    sent = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    assert sent.status_code == 201
    body = sent.json()
    assert body["status"] == "pending"
    assert body["user"]["username"] == "bruno"

    act_as("bruno")
    requests = await client.get("/api/v1/friends/requests")
    assert requests.status_code == 200
    incoming = requests.json()["incoming"]
    assert len(incoming) == 1
    assert incoming[0]["user"]["username"] == "ana"

    accepted = await client.post(f"/api/v1/friends/requests/{incoming[0]['id']}/accept")
    assert accepted.status_code == 200
    assert accepted.json()["status"] == "accepted"

    friends = await client.get("/api/v1/friends")
    assert [friend["username"] for friend in friends.json()] == ["ana"]


async def test_duplicate_request_is_rejected(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    await register(client, act_as, "bruno", name="Bruno")

    act_as("ana")
    first = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    second = await client.post("/api/v1/friends/requests", json={"username": "bruno"})

    assert first.status_code == 201
    assert second.status_code == 409
    assert second.json()["detail"] == "request_pending"


async def test_cannot_friend_yourself(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")

    act_as("ana")
    response = await client.post("/api/v1/friends/requests", json={"username": "ana"})

    assert response.status_code == 409
    assert response.json()["detail"] == "cannot_friend_yourself"


async def test_only_addressee_can_accept(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    await register(client, act_as, "bruno", name="Bruno")
    await register(client, act_as, "carla", name="Carla")

    act_as("ana")
    sent = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    friendship_id = sent.json()["id"]

    act_as("carla")
    response = await client.post(f"/api/v1/friends/requests/{friendship_id}/accept")

    assert response.status_code == 403
    assert response.json()["detail"] == "not_your_request"


async def test_reject_and_remove(client: AsyncClient, act_as) -> None:
    ana, bruno = await befriend(client, act_as, "ana", "Ana", "bruno", "Bruno")

    act_as("ana")
    removed = await client.delete(f"/api/v1/friends/{bruno['id']}")
    assert removed.status_code == 204

    friends = await client.get("/api/v1/friends")
    assert friends.json() == []

    act_as("bruno")
    sent = await client.post("/api/v1/friends/requests", json={"username": "ana"})
    assert sent.status_code == 201

    act_as("ana")
    rejected = await client.post(f"/api/v1/friends/requests/{sent.json()['id']}/reject")
    assert rejected.status_code == 200
    assert rejected.json()["status"] == "rejected"

    # Tras rechazar, se puede volver a intentar.
    act_as("bruno")
    again = await client.post("/api/v1/friends/requests", json={"username": "ana"})
    assert again.status_code == 201
    assert ana["username"] == "ana"


async def test_cancel_request(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    await register(client, act_as, "bruno", name="Bruno")

    act_as("ana")
    sent = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    assert sent.status_code == 201
    friendship_id = sent.json()["id"]

    act_as("bruno")
    incoming = await client.get("/api/v1/friends/requests")
    assert len(incoming.json()["incoming"]) == 1

    act_as("ana")
    cancelled = await client.delete(f"/api/v1/friends/requests/{friendship_id}")
    assert cancelled.status_code == 204

    act_as("bruno")
    after = await client.get("/api/v1/friends/requests")
    assert after.json()["incoming"] == []


async def test_only_requester_can_cancel(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    await register(client, act_as, "bruno", name="Bruno")

    act_as("ana")
    sent = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    friendship_id = sent.json()["id"]

    act_as("bruno")
    denied = await client.delete(f"/api/v1/friends/requests/{friendship_id}")
    assert denied.status_code == 403
    assert denied.json()["detail"] == "not_your_request"


async def test_friend_request_notifies_addressee(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    bruno = await register(client, act_as, "bruno", name="Bruno")

    socket = FakeSocket()
    await manager.connect(uuid.UUID(bruno["id"]), socket)  # type: ignore[arg-type]
    try:
        act_as("ana")
        await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    finally:
        await manager.disconnect(uuid.UUID(bruno["id"]), socket)  # type: ignore[arg-type]

    assert len(socket.messages) == 1
    assert '"friend.request.received"' in socket.messages[0]
    assert '"username": "ana"' in socket.messages[0]


async def test_accept_notifies_requester(client: AsyncClient, act_as) -> None:
    ana = await register(client, act_as, "ana", name="Ana")
    await register(client, act_as, "bruno", name="Bruno")

    act_as("ana")
    sent = await client.post("/api/v1/friends/requests", json={"username": "bruno"})
    friendship_id = sent.json()["id"]

    socket = FakeSocket()
    await manager.connect(uuid.UUID(ana["id"]), socket)  # type: ignore[arg-type]
    try:
        act_as("bruno")
        await client.post(f"/api/v1/friends/requests/{friendship_id}/accept")
    finally:
        await manager.disconnect(uuid.UUID(ana["id"]), socket)  # type: ignore[arg-type]

    assert len(socket.messages) == 1
    assert '"friend.request.accepted"' in socket.messages[0]
    assert '"username": "bruno"' in socket.messages[0]
