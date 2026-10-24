"""Sobre común y cargas útiles del protocolo WebSocket."""

import json
import uuid
from datetime import UTC, datetime
from typing import Any

from pydantic import BaseModel, Field

from tchat_api.core.ids import uuid7


class IncomingEvent(BaseModel):
    type: str = Field(max_length=40)
    id: str | None = Field(default=None, max_length=64)
    payload: dict[str, Any] = Field(default_factory=dict)


class MessageSendPayload(BaseModel):
    conversation_id: uuid.UUID
    body: str = Field(min_length=1, max_length=4000)
    client_id: str | None = Field(default=None, max_length=64)


class ReadPayload(BaseModel):
    conversation_id: uuid.UUID
    message_id: uuid.UUID


class TypingPayload(BaseModel):
    conversation_id: uuid.UUID


def envelope(event_type: str, payload: dict[str, Any], *, event_id: str | None = None) -> str:
    return json.dumps(
        {
            "type": event_type,
            "id": event_id or str(uuid7()),
            "ts": datetime.now(UTC).isoformat(),
            "payload": payload,
        },
        default=str,
        ensure_ascii=False,
    )
