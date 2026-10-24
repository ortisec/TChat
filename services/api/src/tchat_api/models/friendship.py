"""Amistades entre usuarios."""

import uuid
from datetime import datetime
from enum import StrEnum

from sqlalchemy import CheckConstraint, DateTime, ForeignKey, String, UniqueConstraint, Uuid, func
from sqlalchemy.orm import Mapped, mapped_column

from tchat_api.core.ids import uuid7
from tchat_api.db.base import Base


class FriendshipStatus(StrEnum):
    PENDING = "pending"
    ACCEPTED = "accepted"
    REJECTED = "rejected"


class Friendship(Base):
    """Solicitud de amistad en una sola dirección."""

    __tablename__ = "friendships"
    __table_args__ = (
        CheckConstraint("requester_id <> addressee_id", name="ck_friendships_distinct"),
        CheckConstraint(
            "status IN ('pending', 'accepted', 'rejected')",
            name="ck_friendships_status",
        ),
        UniqueConstraint("requester_id", "addressee_id", name="uq_friendships_pair"),
    )

    id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid7)
    requester_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("users.id", ondelete="CASCADE"), index=True
    )
    addressee_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("users.id", ondelete="CASCADE"), index=True
    )
    status: Mapped[str] = mapped_column(String(16), default=FriendshipStatus.PENDING)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    responded_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
