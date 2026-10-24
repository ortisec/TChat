"""Esquemas de usuarios."""

import uuid
from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field


class UserPublic(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: uuid.UUID
    username: str
    display_name: str | None
    email: str | None
    created_at: datetime


class UserUpdate(BaseModel):
    display_name: str | None = Field(default=None, min_length=1, max_length=64)
