"""Endpoint WebSocket."""

import asyncio
import uuid
from typing import Any

from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from pydantic import ValidationError
from sqlalchemy import select

from tchat_api.api.serializers import message_public
from tchat_api.core.security import AuthError, resolve_identity
from tchat_api.db.session import session_factory
from tchat_api.models.user import User
from tchat_api.realtime.manager import manager
from tchat_api.realtime.protocol import (
    IncomingEvent,
    MessageSendPayload,
    ReadPayload,
    TypingPayload,
    envelope,
)
from tchat_api.services import conversations as conversations_service
from tchat_api.services import messages as messages_service
from tchat_api.services.errors import ServiceError

router = APIRouter()


def _bearer_token(header: str | None) -> str | None:
    if not header:
        return None
    scheme, _, token = header.partition(" ")
    if scheme.lower() != "bearer" or not token:
        return None
    return token


async def _error(websocket: WebSocket, code: str, message: str, *, ref: str | None = None) -> None:
    await websocket.send_text(
        envelope(
            "error",
            {"code": code, "message": message, "ref": ref},
            event_id=ref,
        )
    )


async def _dispatch(raw: str, websocket: WebSocket, user_id: uuid.UUID) -> None:
    try:
        event = IncomingEvent.model_validate_json(raw)
    except ValidationError:
        await _error(websocket, "invalid_payload", "El evento no es JSON válido.")
        return

    try:
        match event.type:
            case "ping":
                await websocket.send_text(envelope("pong", {}))
            case "message.send":
                await _on_message_send(event, websocket, user_id)
            case "message.read":
                await _on_read(event, user_id)
            case "typing.start":
                await _on_typing(event, user_id, active=True)
            case "typing.stop":
                await _on_typing(event, user_id, active=False)
            case _:
                await _error(
                    websocket,
                    "unknown_event",
                    f"Evento desconocido: {event.type}",
                    ref=event.id,
                )
    except ValidationError:
        await _error(websocket, "invalid_payload", "Cuerpo fuera de contrato.", ref=event.id)
    except ServiceError as exc:
        await _error(websocket, exc.code, exc.code, ref=event.id)


async def _on_message_send(event: IncomingEvent, websocket: WebSocket, user_id: uuid.UUID) -> None:
    payload = MessageSendPayload.model_validate(event.payload)

    async with session_factory() as session:
        me = await session.get(User, user_id)
        if me is None:
            raise ServiceError("user_not_found", status_code=404)

        message, sender = await messages_service.send(
            session, me, payload.conversation_id, payload.body
        )
        recipients = await conversations_service.member_ids(session, payload.conversation_id)
        data = message_public(message, sender).model_dump(mode="json")
        created_at = message.created_at.isoformat()
        message_id = str(message.id)

    await websocket.send_text(
        envelope(
            "message.ack",
            {
                "client_id": payload.client_id,
                "message_id": message_id,
                "created_at": created_at,
            },
            event_id=event.id,
        )
    )
    await manager.broadcast(envelope("message.new", {"message": data}), user_ids=recipients)


async def _on_read(event: IncomingEvent, user_id: uuid.UUID) -> None:
    payload = ReadPayload.model_validate(event.payload)

    async with session_factory() as session:
        me = await session.get(User, user_id)
        if me is None:
            raise ServiceError("user_not_found", status_code=404)
        await messages_service.mark_read(session, me, payload.conversation_id, payload.message_id)


async def _on_typing(event: IncomingEvent, user_id: uuid.UUID, *, active: bool) -> None:
    payload = TypingPayload.model_validate(event.payload)

    async with session_factory() as session:
        await conversations_service.require_membership(session, payload.conversation_id, user_id)
        recipients = await conversations_service.member_ids(session, payload.conversation_id)

    data: dict[str, Any] = {
        "conversation_id": str(payload.conversation_id),
        "user_id": str(user_id),
        "active": active,
    }
    for recipient in recipients:
        if recipient != user_id:
            await manager.send_to_user(recipient, envelope("typing.update", data))


@router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket) -> None:
    token = _bearer_token(websocket.headers.get("authorization"))
    if token is None:
        await websocket.close(code=4401)
        return

    try:
        identity = await asyncio.to_thread(resolve_identity, token)
    except AuthError:
        await websocket.close(code=4401)
        return

    async with session_factory() as session:
        user = await session.scalar(select(User).where(User.firebase_uid == identity.uid))
        if user is None:
            await websocket.close(code=4403)
            return
        user_id = user.id

    await websocket.accept()
    first_connection = await manager.connect(user_id, websocket)

    if first_connection:
        await manager.broadcast(
            envelope(
                "presence.update",
                {"user_id": str(user_id), "status": "online"},
            )
        )

    try:
        while True:
            raw = await websocket.receive_text()
            await _dispatch(raw, websocket, user_id)
    except WebSocketDisconnect:
        pass
    finally:
        last_connection = await manager.disconnect(user_id, websocket)
        if last_connection:
            await manager.broadcast(
                envelope(
                    "presence.update",
                    {"user_id": str(user_id), "status": "offline"},
                )
            )
