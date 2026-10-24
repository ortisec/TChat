"""Conexiones WebSocket activas."""

import asyncio
import uuid
from collections.abc import Iterable

from fastapi import WebSocket


class ConnectionManager:
    """Registro en memoria de conexiones por usuario.

    El despliegue actual es un solo proceso; si se escala horizontalmente habrá que mover
    este estado a Redis y usar un pub/sub para la entrega entre instancias.
    """

    def __init__(self) -> None:
        self._connections: dict[uuid.UUID, set[WebSocket]] = {}
        self._lock = asyncio.Lock()

    async def connect(self, user_id: uuid.UUID, websocket: WebSocket) -> bool:
        """Registra la conexión y devuelve True si es la primera del usuario."""
        async with self._lock:
            sockets = self._connections.setdefault(user_id, set())
            first = not sockets
            sockets.add(websocket)
            return first

    async def disconnect(self, user_id: uuid.UUID, websocket: WebSocket) -> bool:
        """Elimina la conexión y devuelve True si el usuario se quedó sin ninguna."""
        async with self._lock:
            sockets = self._connections.get(user_id)
            if sockets is None:
                return False

            sockets.discard(websocket)
            if sockets:
                return False

            self._connections.pop(user_id, None)
            return True

    def is_online(self, user_id: uuid.UUID) -> bool:
        return bool(self._connections.get(user_id))

    async def send_to_user(self, user_id: uuid.UUID, message: str) -> None:
        for websocket in list(self._connections.get(user_id, ())):
            try:
                await websocket.send_text(message)
            except Exception:
                await self.disconnect(user_id, websocket)

    async def broadcast(self, message: str, *, user_ids: Iterable[uuid.UUID] | None = None) -> None:
        targets = list(self._connections) if user_ids is None else list(user_ids)
        for user_id in targets:
            await self.send_to_user(user_id, message)


manager = ConnectionManager()
