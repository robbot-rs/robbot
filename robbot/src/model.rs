use chrono::{DateTime, Utc};
use serenity::model::{
    channel::{
        Attachment, ChannelMention, Embed, MessageActivity, MessageApplication, MessageFlags,
        MessageReaction, MessageReference, MessageType, Sticker,
    },
    guild::PartialMember,
    id::{ChannelId, GuildId, MessageId, RoleId, WebhookId},
    user::User,
};
use std::convert::From;

#[derive(Clone, Debug)]
pub struct Message {
    pub id: MessageId,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: Option<GuildId>,
    pub kind: MessageType,
    pub member: Option<PartialMember>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mention_channels: Vec<ChannelMention>,
    pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub timestamp: DateTime<Utc>,
    pub tts: bool,
    pub webhook_id: Option<WebhookId>,
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub message_reference: Option<MessageReference>,
    pub flags: Option<MessageFlags>,
    pub stickers: Vec<Sticker>,
    pub referenced_message: Option<Box<Message>>,
}

impl AsRef<MessageId> for Message {
    fn as_ref(&self) -> &MessageId {
        &self.id
    }
}

impl AsRef<ChannelId> for Message {
    fn as_ref(&self) -> &ChannelId {
        &self.channel_id
    }
}

/// A [`Message`] is sent inside a guild. Guarantees some
/// fields to have a non `None` type compared to [`Message`].
#[derive(Clone, Debug)]
pub struct GuildMessage {
    pub id: MessageId,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: GuildId,
    pub kind: MessageType,
    pub member: PartialMember,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mention_channels: Vec<ChannelMention>,
    pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub timestamp: DateTime<Utc>,
    pub tts: bool,
    pub webhook_id: Option<WebhookId>,
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub message_reference: Option<MessageReference>,
    pub flags: Option<MessageFlags>,
    pub stickers: Vec<Sticker>,
    pub referenced_message: Option<Box<GuildMessage>>,
}

impl AsRef<MessageId> for GuildMessage {
    fn as_ref(&self) -> &MessageId {
        &self.id
    }
}

impl AsRef<ChannelId> for GuildMessage {
    fn as_ref(&self) -> &ChannelId {
        &self.channel_id
    }
}

impl From<Message> for GuildMessage {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            attachments: message.attachments,
            author: message.author,
            channel_id: message.channel_id,
            content: message.content,
            edited_timestamp: message.edited_timestamp,
            embeds: message.embeds,
            guild_id: message.guild_id.unwrap(),
            kind: message.kind,
            member: message.member.unwrap(),
            mention_everyone: message.mention_everyone,
            mention_roles: message.mention_roles,
            mention_channels: message.mention_channels,
            mentions: message.mentions,
            pinned: message.pinned,
            reactions: message.reactions,
            timestamp: message.timestamp,
            tts: message.tts,
            webhook_id: message.webhook_id,
            activity: message.activity,
            application: message.application,
            message_reference: message.message_reference,
            flags: message.flags,
            stickers: message.stickers,
            referenced_message: message
                .referenced_message
                .map(|referenced_message| Box::new(Self::from(*referenced_message))),
        }
    }
}

impl From<serenity::model::channel::Message> for Message {
    fn from(t: serenity::model::channel::Message) -> Self {
        Self {
            id: t.id,
            attachments: t.attachments,
            author: t.author,
            channel_id: t.channel_id,
            content: t.content,
            edited_timestamp: t.edited_timestamp,
            embeds: t.embeds,
            guild_id: t.guild_id,
            kind: t.kind,
            member: t.member,
            mention_everyone: t.mention_everyone,
            mention_roles: t.mention_roles,
            mention_channels: t.mention_channels,
            mentions: t.mentions,
            pinned: t.pinned,
            reactions: t.reactions,
            timestamp: t.timestamp,
            tts: t.tts,
            webhook_id: t.webhook_id,
            activity: t.activity,
            application: t.application,
            message_reference: t.message_reference,
            flags: t.flags,
            stickers: t.stickers,
            referenced_message: t.referenced_message.map(|t| Box::new(Message::from(*t))),
        }
    }
}
