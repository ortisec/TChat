"""Dispositivos autorizados y códigos de vinculación."""

import uuid
from datetime import datetime
from enum import StrEnum

from sqlalchemy import CheckConstraint, DateTime, ForeignKey, String, Uuid, func
from sqlalchemy.orm import Mapped, mapped_column

from tchat_api.core.ids import uuid7
from tchat_api.db.base import Base


class DeviceCodeStatus(StrEnum):
    PENDING = "pending"
    APPROVED = "approved"
    CONSUMED = "consumed"


class DeviceToken(Base):
    """Refresh token opaco de un terminal. Solo se guarda su hash."""

    __tablename__ = "device_tokens"

    id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid7)
    user_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("users.id", ondelete="CASCADE"), index=True
    )
    token_hash: Mapped[str] = mapped_column(String(64), unique=True, index=True)
    name: Mapped[str] = mapped_column(String(64), default="terminal")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    last_used_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
    revoked_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)


class DeviceCode(Base):
    """Solicitud de vinculación iniciada por `tchat login`."""

    __tablename__ = "device_codes"
    __table_args__ = (
        CheckConstraint(
            "status IN ('pending', 'approved', 'consumed')",
            name="ck_device_codes_status",
        ),
    )

    id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid7)
    user_code: Mapped[str] = mapped_column(String(9), unique=True, index=True)
    device_code_hash: Mapped[str] = mapped_column(String(64), unique=True, index=True)
    status: Mapped[str] = mapped_column(String(16), default=DeviceCodeStatus.PENDING)
    user_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("users.id", ondelete="SET NULL"), default=None, index=True
    )
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    approved_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
    consumed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
    last_polled_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), default=None)
