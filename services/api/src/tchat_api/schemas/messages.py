"""Esquemas de mensajes."""

import uuid
from datetime import datetime

from pydantic import BaseModel, Field

from tchat_api.schemas.users import UserPublic


class MessageCreate(BaseModel):
    body: str = Field(min_length=1, max_length=4000)
    client_id: str | None = Field(default=None, max_length=64)


class MessagePublic(BaseModel):
    id: uuid.UUID
    conversation_id: uuid.UUID
    body: str
    created_at: datetime
    edited_at: datetime | None
    sender: UserPublic
