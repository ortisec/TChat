"""Esquemas de amistades."""

import uuid
from datetime import datetime
from typing import Literal

from pydantic import BaseModel, Field

from tchat_api.schemas.users import UserPublic


class FriendRequestCreate(BaseModel):
    username: str = Field(min_length=1, max_length=32)


class FriendshipPublic(BaseModel):
    id: uuid.UUID
    status: Literal["pending", "accepted", "rejected"]
    created_at: datetime
    responded_at: datetime | None
    user: UserPublic


class FriendRequestsPublic(BaseModel):
    incoming: list[FriendshipPublic]
    outgoing: list[FriendshipPublic]
