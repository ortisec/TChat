"""amistades, conversaciones, mensajes y dispositivos

Revision ID: 0002_social
Revises: 0001_users
Create Date: 2026-10-24

"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

revision: str = "0002_social"
down_revision: str | None = "0001_users"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column("users", sa.Column("email", sa.String(length=255), nullable=True))

    op.create_table(
        "friendships",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("requester_id", sa.Uuid(), nullable=False),
        sa.Column("addressee_id", sa.Uuid(), nullable=False),
        sa.Column("status", sa.String(length=16), nullable=False),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("responded_at", sa.DateTime(timezone=True), nullable=True),
        sa.CheckConstraint("requester_id <> addressee_id", name="ck_friendships_distinct"),
        sa.CheckConstraint(
            "status IN ('pending', 'accepted', 'rejected')",
            name="ck_friendships_status",
        ),
        sa.ForeignKeyConstraint(["addressee_id"], ["users.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["requester_id"], ["users.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("requester_id", "addressee_id", name="uq_friendships_pair"),
    )
    op.create_index("ix_friendships_addressee_id", "friendships", ["addressee_id"])
    op.create_index("ix_friendships_requester_id", "friendships", ["requester_id"])

    op.create_table(
        "conversations",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("type", sa.String(length=16), nullable=False),
        sa.Column("title", sa.String(length=64), nullable=True),
        sa.Column("created_by", sa.Uuid(), nullable=False),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column(
            "updated_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.CheckConstraint("type IN ('direct', 'group')", name="ck_conversations_type"),
        sa.ForeignKeyConstraint(["created_by"], ["users.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_conversations_created_by", "conversations", ["created_by"])
    op.create_index("ix_conversations_updated_at", "conversations", ["updated_at"])

    op.create_table(
        "conversation_members",
        sa.Column("conversation_id", sa.Uuid(), nullable=False),
        sa.Column("user_id", sa.Uuid(), nullable=False),
        sa.Column(
            "joined_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("last_read_at", sa.DateTime(timezone=True), nullable=True),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["user_id"], ["users.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("conversation_id", "user_id"),
    )
    op.create_index("ix_conversation_members_user_id", "conversation_members", ["user_id"])

    op.create_table(
        "messages",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("conversation_id", sa.Uuid(), nullable=False),
        sa.Column("sender_id", sa.Uuid(), nullable=False),
        sa.Column("body", sa.String(length=4000), nullable=False),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("edited_at", sa.DateTime(timezone=True), nullable=True),
        sa.ForeignKeyConstraint(["conversation_id"], ["conversations.id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["sender_id"], ["users.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_messages_sender_id", "messages", ["sender_id"])
    op.create_index(
        "ix_messages_conversation_created",
        "messages",
        ["conversation_id", "created_at"],
    )

    op.create_table(
        "device_tokens",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("user_id", sa.Uuid(), nullable=False),
        sa.Column("token_hash", sa.String(length=64), nullable=False),
        sa.Column("name", sa.String(length=64), nullable=False),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("last_used_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("revoked_at", sa.DateTime(timezone=True), nullable=True),
        sa.ForeignKeyConstraint(["user_id"], ["users.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_device_tokens_user_id", "device_tokens", ["user_id"])
    op.create_index("ix_device_tokens_token_hash", "device_tokens", ["token_hash"], unique=True)

    op.create_table(
        "device_codes",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("user_code", sa.String(length=9), nullable=False),
        sa.Column("device_code_hash", sa.String(length=64), nullable=False),
        sa.Column("status", sa.String(length=16), nullable=False),
        sa.Column("user_id", sa.Uuid(), nullable=True),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("expires_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("approved_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("consumed_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("last_polled_at", sa.DateTime(timezone=True), nullable=True),
        sa.CheckConstraint(
            "status IN ('pending', 'approved', 'consumed')",
            name="ck_device_codes_status",
        ),
        sa.ForeignKeyConstraint(["user_id"], ["users.id"], ondelete="SET NULL"),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index("ix_device_codes_user_id", "device_codes", ["user_id"])
    op.create_index("ix_device_codes_user_code", "device_codes", ["user_code"], unique=True)
    op.create_index(
        "ix_device_codes_device_code_hash",
        "device_codes",
        ["device_code_hash"],
        unique=True,
    )


def downgrade() -> None:
    op.drop_index("ix_device_codes_device_code_hash", table_name="device_codes")
    op.drop_index("ix_device_codes_user_code", table_name="device_codes")
    op.drop_index("ix_device_codes_user_id", table_name="device_codes")
    op.drop_table("device_codes")

    op.drop_index("ix_device_tokens_token_hash", table_name="device_tokens")
    op.drop_index("ix_device_tokens_user_id", table_name="device_tokens")
    op.drop_table("device_tokens")

    op.drop_index("ix_messages_conversation_created", table_name="messages")
    op.drop_index("ix_messages_sender_id", table_name="messages")
    op.drop_table("messages")

    op.drop_index("ix_conversation_members_user_id", table_name="conversation_members")
    op.drop_table("conversation_members")

    op.drop_index("ix_conversations_updated_at", table_name="conversations")
    op.drop_index("ix_conversations_created_by", table_name="conversations")
    op.drop_table("conversations")

    op.drop_index("ix_friendships_requester_id", table_name="friendships")
    op.drop_index("ix_friendships_addressee_id", table_name="friendships")
    op.drop_table("friendships")

    op.drop_column("users", "email")
