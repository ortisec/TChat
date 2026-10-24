"""Registro de conexiones en memoria."""

import uuid

from tchat_api.realtime.manager import ConnectionManager
from tchat_api.realtime.protocol import envelope


class FakeSocket:
    def __init__(self) -> None:
        self.messages: list[str] = []

    async def send_text(self, message: str) -> None:
        self.messages.append(message)


async def test_first_and_last_connection() -> None:
    manager = ConnectionManager()
    user_id = uuid.uuid4()
    first_socket = FakeSocket()
    second_socket = FakeSocket()

    assert await manager.connect(user_id, first_socket) is True
    assert await manager.connect(user_id, second_socket) is False
    assert manager.is_online(user_id) is True

    assert await manager.disconnect(user_id, first_socket) is False
    assert await manager.disconnect(user_id, second_socket) is True
    assert manager.is_online(user_id) is False


async def test_broadcast_only_to_requested_members() -> None:
    manager = ConnectionManager()
    ana = uuid.uuid4()
    bruno = uuid.uuid4()
    ana_socket = FakeSocket()

    await manager.connect(ana, ana_socket)

    payload = envelope("message.new", {"body": "hola"})
    await manager.broadcast(payload, user_ids=[ana, bruno])

    assert len(ana_socket.messages) == 1
    assert '"message.new"' in ana_socket.messages[0]
    assert '"body": "hola"' in ana_socket.messages[0]


async def test_envelope_has_common_fields() -> None:
    raw = envelope("pong", {})

    assert '"type": "pong"' in raw
    assert '"id"' in raw
    assert '"ts"' in raw
    assert '"payload": {}' in raw
