use crate as robbot;
use crate::util::color::Color;
use crate::{Decode, Encode};

use super::guild::PartialMember;
use super::id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};
use super::permissions::Permissions;
use super::user::User;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Message {
    pub id: MessageId,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: Option<GuildId>,
    pub kind: MessageKind,
    pub member: Option<PartialMember>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mention_channels: Vec<ChannelId>,
    pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub timestamp: DateTime<Utc>,
    pub tts: bool,
    pub message_reference: Option<MessageReference>,
    pub referenced_message: Option<Box<Message>>,
}

impl AsRef<ChannelId> for Message {
    fn as_ref(&self) -> &ChannelId {
        &self.channel_id
    }
}

impl AsRef<MessageId> for Message {
    fn as_ref(&self) -> &MessageId {
        &self.id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct MessageReference {
    pub message_id: Option<MessageId>,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

impl From<(ChannelId, MessageId)> for MessageReference {
    fn from(src: (ChannelId, MessageId)) -> Self {
        Self {
            message_id: Some(src.1),
            channel_id: src.0,
            guild_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Embed {
    pub author: Option<EmbedAuthor>,
    pub color: Color,
    pub description: Option<String>,
    pub fields: Vec<EmbedField>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedImage>,
    pub kind: String,
    pub provider: Option<EmbedProvider>,
    pub thumbnail: Option<EmbedThumbnail>,
    pub title: Option<String>,
    pub timestamp: Option<String>,
    pub url: Option<String>,
    pub video: Option<EmbedVideo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedAuthor {
    pub icon_url: Option<String>,
    pub name: String,
    pub proxy_icon_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedField {
    pub inline: bool,
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedFooter {
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedImage {
    pub height: u64,
    pub proxy_url: String,
    pub url: String,
    pub width: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedProvider {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedThumbnail {
    pub height: u64,
    pub proxy_url: String,
    pub url: String,
    pub width: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct EmbedVideo {
    pub height: u64,
    pub url: String,
    pub width: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct MessageReaction {
    pub count: u64,
    pub me: bool,
    pub reaction_type: ReactionType,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum ReactionType {
    Custom {
        animated: bool,
        id: EmojiId,
        name: Option<String>,
    },
    Unicode(String),
}

/// A [`Message`] is sent inside a guild. Guarantees some
/// fields to have a non `None` type compared to [`Message`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMessage {
    pub id: MessageId,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: GuildId,
    pub kind: MessageKind,
    pub member: PartialMember,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mention_channels: Vec<ChannelId>,
    pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub timestamp: DateTime<Utc>,
    pub tts: bool,
    pub message_reference: Option<MessageReference>,
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

impl AsRef<GuildId> for GuildMessage {
    fn as_ref(&self) -> &GuildId {
        &self.guild_id
    }
}

pub struct NotAGuildMessage;

impl TryFrom<Message> for GuildMessage {
    type Error = NotAGuildMessage;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let guild_id = value.guild_id.ok_or(NotAGuildMessage)?;
        let member = value.member.ok_or(NotAGuildMessage)?;

        let referenced_message = match value.referenced_message {
            Some(msg) => Some(Box::new(Self::try_from(*msg)?)),
            None => None,
        };

        Ok(Self {
            id: value.id,
            attachments: value.attachments,
            author: value.author,
            channel_id: value.channel_id,
            content: value.content,
            edited_timestamp: value.edited_timestamp,
            embeds: value.embeds,
            guild_id,
            kind: value.kind,
            member,
            mention_everyone: value.mention_everyone,
            mention_roles: value.mention_roles,
            mention_channels: value.mention_channels,
            mentions: value.mentions,
            pinned: value.pinned,
            reactions: value.reactions,
            timestamp: value.timestamp,
            tts: value.tts,
            message_reference: value.message_reference,
            referenced_message,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Attachment {
    pub id: AttachmentId,
    pub filename: String,
    pub height: Option<u64>,
    pub proxy_url: String,
    pub size: u64,
    pub url: String,
    pub width: Option<u64>,
    pub content_type: Option<String>,
    pub ephemeral: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum MessageKind {
    Regular,
    GroupRecipientAddition,
    GroupRecipientRemoval,
    GroupCallCreation,
    GroupNameUpdate,
    GroupIconUpdate,
    PinsAdd,
    MemberJoin,
    NitroBoost,
    NitroTier1,
    NitroTier2,
    NitroTier3,
    ChannelFollowAdd,
    GuildDiscoveryDisqualified,
    GuildDiscoveryRequalified,
    GuildDiscoveryGracePeriodInitialWarning,
    GuildDiscoveryGracePeriodFinalWarning,
    ThreadCreated,
    InlineReply,
    ApplicationCommand,
    ThreadStarterMessage,
    Unknown,
}

/// An Emoji reaction to a Message.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Reaction {
    /// The [`ChannelId`] of the [`Message`] that was reacted to.
    pub channel_id: ChannelId,
    /// The Emoji reacted with.
    pub emoji: ReactionType,
    /// The [`MessageId`] of the [`Message`] that was reacted to.
    pub message_id: MessageId,
    pub user_id: Option<UserId>,
    /// The [`GuildId`] of the [`Message`], if it was sent in a Guild.
    pub guild_id: Option<GuildId>,
    pub member: Option<PartialMember>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum ChannelKind {
    Text,
    Private,
    Voice,
    Category,
    News,
    Store,
    NewsThread,
    PublicThread,
    PrivateThread,
    Stage,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum Channel {
    Guild(GuildChannel),
    Private(PrivateChannel),
    Category(CategoryChannel),
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct GuildChannel {
    pub id: ChannelId,
    pub bitrate: Option<u64>,
    pub category_id: Option<ChannelId>,
    pub guild_id: GuildId,
    pub kind: ChannelKind,
    pub last_message_id: Option<MessageId>,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
    pub topic: Option<String>,
    pub user_limit: Option<u64>,
    pub nsfw: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteKind,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum PermissionOverwriteKind {
    Member(UserId),
    Role(RoleId),
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct PrivateChannel {
    pub id: ChannelId,
    pub last_message_id: Option<MessageId>,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
    pub kind: ChannelKind,
    pub recipient: User,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct CategoryChannel {
    pub id: ChannelId,
    pub guild_id: GuildId,
    pub category_id: Option<ChannelId>,
    pub position: i64,
    pub kind: ChannelKind,
    pub name: String,
    pub nsfw: bool,
    pub permission_overwrites: Vec<PermissionOverwrite>,
}
