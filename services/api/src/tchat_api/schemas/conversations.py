"""Esquemas de conversaciones."""

import uuid
from datetime import datetime
from typing import Annotated, Literal

from pydantic import BaseModel, Field

from tchat_api.schemas.messages import MessagePublic
from tchat_api.schemas.users import UserPublic


class DirectConversationCreate(BaseModel):
    type: Literal["direct"]
    user_id: uuid.UUID


class GroupConversationCreate(BaseModel):
    type: Literal["group"]
    title: str = Field(min_length=1, max_length=64)
    member_ids: list[uuid.UUID] = Field(min_length=1, max_length=64)


ConversationCreate = Annotated[
    DirectConversationCreate | GroupConversationCreate,
    Field(discriminator="type"),
]


class ConversationPublic(BaseModel):
    id: uuid.UUID
    type: Literal["direct", "group"]
    title: str | None
    created_at: datetime
    updated_at: datetime
    members: list[UserPublic]
    last_message: MessagePublic | None
    unread_count: int


class ReadRequest(BaseModel):
    message_id: uuid.UUID
