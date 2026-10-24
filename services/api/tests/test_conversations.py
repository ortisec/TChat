"""Conversaciones y mensajes."""

from httpx import AsyncClient

from tests.helpers import befriend, register


async def test_direct_conversation_requires_friendship(client: AsyncClient, act_as) -> None:
    await register(client, act_as, "ana", name="Ana")
    carla = await register(client, act_as, "carla", name="Carla")

    act_as("ana")
    response = await client.post(
        "/api/v1/conversations",
        json={"type": "direct", "user_id": carla["id"]},
    )

    assert response.status_code == 403
    assert response.json()["detail"] == "not_friends"


async def test_message_flow_and_unread(client: AsyncClient, act_as) -> None:
    ana, bruno = await befriend(client, act_as, "ana", "Ana", "bruno", "Bruno")

    act_as("ana")
    created = await client.post(
        "/api/v1/conversations",
        json={"type": "direct", "user_id": bruno["id"]},
    )
    assert created.status_code == 201
    conversation = created.json()
    assert conversation["type"] == "direct"
    assert {member["username"] for member in conversation["members"]} == {
        "ana",
        "bruno",
    }
    conversation_id = conversation["id"]

    for body in ["uno", "dos", "tres"]:
        sent = await client.post(
            f"/api/v1/conversations/{conversation_id}/messages",
            json={"body": body},
        )
        assert sent.status_code == 201

    act_as("bruno")
    listing = await client.get("/api/v1/conversations")
    assert listing.status_code == 200
    current = next(item for item in listing.json() if item["id"] == conversation_id)
    assert current["unread_count"] == 3
    assert current["last_message"]["body"] == "tres"

    history = await client.get(f"/api/v1/conversations/{conversation_id}/messages?limit=2")
    assert [message["body"] for message in history.json()] == ["dos", "tres"]

    last_message_id = history.json()[-1]["id"]
    marked = await client.post(
        f"/api/v1/conversations/{conversation_id}/read",
        json={"message_id": last_message_id},
    )
    assert marked.status_code == 204

    listing = await client.get("/api/v1/conversations")
    current = next(item for item in listing.json() if item["id"] == conversation_id)
    assert current["unread_count"] == 0
    assert ana["username"] == "ana"


async def test_non_member_cannot_read_conversation(client: AsyncClient, act_as) -> None:
    _, bruno = await befriend(client, act_as, "ana", "Ana", "bruno", "Bruno")
    await register(client, act_as, "carla", name="Carla")

    act_as("ana")
    created = await client.post(
        "/api/v1/conversations",
        json={"type": "direct", "user_id": bruno["id"]},
    )
    conversation_id = created.json()["id"]

    act_as("carla")
    detail = await client.get(f"/api/v1/conversations/{conversation_id}")
    send = await client.post(
        f"/api/v1/conversations/{conversation_id}/messages",
        json={"body": "hola"},
    )

    assert detail.status_code == 404
    assert send.status_code == 404


async def test_group_requires_friends(client: AsyncClient, act_as) -> None:
    _, bruno = await befriend(client, act_as, "ana", "Ana", "bruno", "Bruno")
    carla = await register(client, act_as, "carla", name="Carla")

    act_as("ana")
    created = await client.post(
        "/api/v1/conversations",
        json={
            "type": "group",
            "title": "Equipo",
            "member_ids": [bruno["id"]],
        },
    )
    assert created.status_code == 201
    assert created.json()["title"] == "Equipo"
    assert len(created.json()["members"]) == 2

    rejected = await client.post(
        "/api/v1/conversations",
        json={
            "type": "group",
            "title": "Solo amigos",
            "member_ids": [carla["id"]],
        },
    )
    assert rejected.status_code == 403
    assert rejected.json()["detail"] == "not_friends"


async def test_blank_message_is_rejected(client: AsyncClient, act_as) -> None:
    _, bruno = await befriend(client, act_as, "ana", "Ana", "bruno", "Bruno")

    act_as("ana")
    created = await client.post(
        "/api/v1/conversations",
        json={"type": "direct", "user_id": bruno["id"]},
    )

    response = await client.post(
        f"/api/v1/conversations/{created.json()['id']}/messages",
        json={"body": "   "},
    )

    assert response.status_code == 400
    assert response.json()["detail"] == "empty_message"
