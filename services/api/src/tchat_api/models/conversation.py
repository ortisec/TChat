"""Conversaciones y sus miembros."""

import uuid
from datetime import datetime
from enum import StrEnum

from sqlalchemy import CheckConstraint, DateTime, ForeignKey, String, Uuid, func
from sqlalchemy.orm import Mapped, mapped_column

from tchat_api.core.ids import uuid7
from tchat_api.db.base import Base


class ConversationType(StrEnum):
    DIRECT = "direct"
    GROUP = "group"


class Conversation(Base):
    __tablename__ = "conversations"
    __table_args__ = (CheckConstraint("type IN ('direct', 'group')", name="ck_conversations_type"),)

    id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid7)
    type: Mapped[str] = mapped_column(String(16), default=ConversationType.DIRECT)
    title: Mapped[str | None] = mapped_column(String(64), default=None)
    created_by: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("users.id", ondelete="CASCADE"), index=True
    )
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), index=True
    )


class ConversationMember(Base):
    __tablename__ = "conversation_members"

    conversation_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("conversations.id", ondelete="CASCADE"), primary_key=True
    )
    user_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("users.id", ondelete="CASCADE"), primary_key=True, index=True
    )
    joined_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    last_read_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
