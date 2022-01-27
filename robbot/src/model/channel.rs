use crate as robbot;
use crate::{Decode, Encode};

use super::id::{ChannelId, EmojiId, GuildId, MessageId};
use super::user::User;

use serde::{Deserialize, Serialize};

use serenity::model::channel::{
    Embed as SEmbed, EmbedAuthor as SEmbedAuthor, EmbedField as SEmbedField,
    EmbedFooter as SEmbedFooter, EmbedImage as SEmbedImage, EmbedProvider as SEmbedProvider,
    EmbedThumbnail as SEmbedThumbnail, EmbedVideo as SEmbedVideo,
    MessageReaction as SMessageReaction, MessageReference as SMessageReference,
    ReactionType as SReactionType,
};
use serenity::utils::Colour as SColor;

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Message {
    pub id: MessageId,
    // pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    // pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: Option<GuildId>,
    // pub kind: MessageType,
    // pub member: Option<PartialMember>,
    pub mention_everyone: bool,
    // pub mention_roles: Vec<RoleId>,
    // pub mention_channels: Vec<ChannelMention>,
    // pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    // pub timestamp: DateTime<Utc>,
    pub tts: bool,
    // pub webhook_id: Option<WebhookId>,
    // pub activity: Option<MessageActivity>,
    // pub application: Option<MessageApplication>,
    pub message_reference: Option<MessageReference>,
    // pub flags: Option<MessageFlags>,
    // pub stickers: Vec<Sticker>,
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

impl From<serenity::model::channel::Message> for Message {
    fn from(t: serenity::model::channel::Message) -> Self {
        Self {
            id: t.id.into(),
            // attachments: t.attachments,
            author: t.author.into(),
            channel_id: t.channel_id.into(),
            content: t.content,
            // edited_timestamp: t.edited_timestamp,
            embeds: t.embeds.into_iter().map(|v| v.into()).collect(),
            guild_id: t.guild_id.map(|v| v.into()),
            // kind: t.kind.into(),
            // member: t.member,
            mention_everyone: t.mention_everyone,
            // mention_roles: t.mention_roles,
            // mention_channels: t.mention_channels,
            // mentions: t.mentions.and_then(),
            pinned: t.pinned,
            reactions: t.reactions.into_iter().map(|v| v.into()).collect(),
            // timestamp: t.timestamp,
            tts: t.tts,
            // webhook_id: t.webhook_id,
            // activity: t.activity,
            // application: t.application,
            message_reference: t.message_reference.map(|v| v.into()),
            // flags: t.flags,
            // stickers: t.stickers,
            referenced_message: t.referenced_message.map(|t| Box::new(Message::from(*t))),
        }
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct Color(pub u32);

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

// #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
// pub enum MessageType {
//     Regular,
//     GroupRecipientAddition,
//     GroupRecipientRemoval,
//     GroupCallCreation,
//     GroupNameUpdate,
//     GroupIconUpdate,
//     PinsAdd,
//     MemberJoin,
//     NitroBoost,
//     NitroTier1,
//     NitroTier2,
//     NitroTier3,
//     ChannelFollowAdd,
//     GuildDiscoveryDisqualified,
//     GuildDiscoveryRequalified,
//     GuildDiscoveryGracePeriodInitialWarning,
//     GuildDiscoveryGracePeriodFinalWarning,
//     ThreadCreated,
//     InlineReply,
//     ApplicationCommand,
//     ThreadStarterMessage,
//     GuildInviteReminder,
//     Unknown,
// }

impl From<SEmbed> for Embed {
    fn from(src: SEmbed) -> Self {
        Self {
            author: src.author.map(|v| v.into()),
            color: src.colour.into(),
            description: src.description,
            fields: src.fields.into_iter().map(|v| v.into()).collect(),
            footer: src.footer.map(|v| v.into()),
            image: src.image.map(|v| v.into()),
            kind: src.kind,
            provider: src.provider.map(|v| v.into()),
            thumbnail: src.thumbnail.map(|v| v.into()),
            timestamp: src.timestamp,
            url: src.url,
            video: src.video.map(|v| v.into()),
        }
    }
}

impl From<SEmbedAuthor> for EmbedAuthor {
    fn from(src: SEmbedAuthor) -> Self {
        Self {
            icon_url: src.icon_url,
            name: src.name,
            proxy_icon_url: src.proxy_icon_url,
            url: src.url,
        }
    }
}

impl From<SEmbedField> for EmbedField {
    fn from(src: SEmbedField) -> Self {
        Self {
            inline: src.inline,
            name: src.name,
            value: src.value,
        }
    }
}

impl From<SEmbedFooter> for EmbedFooter {
    fn from(src: SEmbedFooter) -> Self {
        Self {
            icon_url: src.icon_url,
            proxy_icon_url: src.proxy_icon_url,
            text: src.text,
        }
    }
}

impl From<SEmbedImage> for EmbedImage {
    fn from(src: SEmbedImage) -> Self {
        Self {
            height: src.height,
            proxy_url: src.proxy_url,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<SEmbedProvider> for EmbedProvider {
    fn from(src: SEmbedProvider) -> Self {
        Self {
            name: src.name,
            url: src.url,
        }
    }
}

impl From<SEmbedThumbnail> for EmbedThumbnail {
    fn from(src: SEmbedThumbnail) -> Self {
        Self {
            height: src.height,
            proxy_url: src.proxy_url,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<SEmbedVideo> for EmbedVideo {
    fn from(src: SEmbedVideo) -> Self {
        Self {
            height: src.height,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<SColor> for Color {
    fn from(src: SColor) -> Self {
        Self(src.0)
    }
}

impl From<SMessageReference> for MessageReference {
    fn from(src: SMessageReference) -> MessageReference {
        Self {
            message_id: src.message_id.map(|v| v.into()),
            channel_id: src.channel_id.into(),
            guild_id: src.guild_id.map(|v| v.into()),
        }
    }
}

impl From<SMessageReaction> for MessageReaction {
    fn from(src: SMessageReaction) -> MessageReaction {
        Self {
            count: src.count,
            me: src.me,
            reaction_type: src.reaction_type.into(),
        }
    }
}

impl From<SReactionType> for ReactionType {
    fn from(src: SReactionType) -> ReactionType {
        match src {
            SReactionType::Custom { animated, id, name } => Self::Custom {
                animated,
                id: id.into(),
                name,
            },
            SReactionType::Unicode(s) => Self::Unicode(s),
            _ => unreachable!(),
        }
    }
}

/// A [`Message`] is sent inside a guild. Guarantees some
/// fields to have a non `None` type compared to [`Message`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMessage {
    pub id: MessageId,
    // pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    // pub edited_timestamp: Option<DateTime<Utc>>,
    pub embeds: Vec<Embed>,
    pub guild_id: GuildId,
    // pub kind: MessageType,
    // pub member: PartialMember,
    pub mention_everyone: bool,
    // pub mention_roles: Vec<RoleId>,
    // pub mention_channels: Vec<ChannelMention>,
    // pub mentions: Vec<User>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    // pub timestamp: DateTime<Utc>,
    pub tts: bool,
    // pub webhook_id: Option<WebhookId>,
    // pub activity: Option<MessageActivity>,
    // pub application: Option<MessageApplication>,
    pub message_reference: Option<MessageReference>,
    // pub flags: Option<MessageFlags>,
    // pub stickers: Vec<Sticker>,
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

impl From<Message> for GuildMessage {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            // attachments: message.attachments,
            author: message.author,
            channel_id: message.channel_id,
            content: message.content,
            // edited_timestamp: message.edited_timestamp,
            embeds: message.embeds,
            guild_id: message.guild_id.unwrap(),
            // kind: message.kind,
            // member: message.member.unwrap(),
            mention_everyone: message.mention_everyone,
            // mention_roles: message.mention_roles,
            // mention_channels: message.mention_channels,
            // mentions: message.mentions,
            pinned: message.pinned,
            reactions: message.reactions,
            // timestamp: message.timestamp,
            tts: message.tts,
            // webhook_id: message.webhook_id,
            // activity: message.activity,
            // application: message.application,
            message_reference: message.message_reference,
            // flags: message.flags,
            // stickers: message.stickers,
            referenced_message: message
                .referenced_message
                .map(|referenced_message| Box::new(Self::from(*referenced_message))),
        }
    }
}
