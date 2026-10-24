"""Conversión de modelos a esquemas públicos."""

from typing import Literal, cast

from tchat_api.models.friendship import Friendship
from tchat_api.models.message import Message
from tchat_api.models.user import User
from tchat_api.schemas.conversations import ConversationPublic
from tchat_api.schemas.friends import FriendshipPublic
from tchat_api.schemas.messages import MessagePublic
from tchat_api.schemas.users import UserPublic
from tchat_api.services.conversations import ConversationBundle


def user_public(user: User) -> UserPublic:
    return UserPublic.model_validate(user)


def message_public(message: Message, sender: User) -> MessagePublic:
    return MessagePublic(
        id=message.id,
        conversation_id=message.conversation_id,
        body=message.body,
        created_at=message.created_at,
        edited_at=message.edited_at,
        sender=user_public(sender),
    )


def friendship_public(friendship: Friendship, counterpart: User) -> FriendshipPublic:
    return FriendshipPublic(
        id=friendship.id,
        status=cast(Literal["pending", "accepted", "rejected"], friendship.status),
        created_at=friendship.created_at,
        responded_at=friendship.responded_at,
        user=user_public(counterpart),
    )


def conversation_public(bundle: ConversationBundle) -> ConversationPublic:
    last_message = (
        message_public(bundle.last_message, bundle.last_sender)
        if bundle.last_message is not None and bundle.last_sender is not None
        else None
    )
    return ConversationPublic(
        id=bundle.conversation.id,
        type=cast(Literal["direct", "group"], bundle.conversation.type),
        title=bundle.conversation.title,
        created_at=bundle.conversation.created_at,
        updated_at=bundle.conversation.updated_at,
        members=[user_public(member) for member in bundle.members],
        last_message=last_message,
        unread_count=bundle.unread_count,
    )
