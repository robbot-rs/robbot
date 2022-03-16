use super::channel;

use serenity::model::channel::{
    Attachment, Channel, ChannelCategory, ChannelType, Embed, EmbedAuthor, EmbedField, EmbedFooter,
    EmbedImage, EmbedProvider, EmbedThumbnail, EmbedVideo, GuildChannel, Message, MessageReaction,
    MessageReference, MessageType, PermissionOverwrite, PermissionOverwriteType, PrivateChannel,
    Reaction, ReactionType,
};

impl From<Message> for channel::Message {
    fn from(src: Message) -> Self {
        Self {
            id: src.id.into(),
            attachments: src.attachments.into_iter().map(|v| v.into()).collect(),
            author: src.author.into(),
            channel_id: src.channel_id.into(),
            content: src.content,
            edited_timestamp: src.edited_timestamp,
            embeds: src.embeds.into_iter().map(|v| v.into()).collect(),
            guild_id: src.guild_id.map(|v| v.into()),
            kind: src.kind.into(),
            member: src.member.map(|v| v.into()),
            mention_everyone: src.mention_everyone,
            mention_roles: src.mention_roles.into_iter().map(|v| v.into()).collect(),
            mention_channels: src
                .mention_channels
                .into_iter()
                .map(|v| v.id.into())
                .collect(),
            mentions: src.mentions.into_iter().map(|v| v.into()).collect(),
            pinned: src.pinned,
            reactions: src.reactions.into_iter().map(|v| v.into()).collect(),
            timestamp: src.timestamp,
            tts: src.tts,
            message_reference: src.message_reference.map(|v| v.into()),
            referenced_message: src.referenced_message.map(|v| Box::new(Self::from(*v))),
        }
    }
}

impl From<Attachment> for channel::Attachment {
    fn from(src: Attachment) -> Self {
        Self {
            id: src.id.into(),
            filename: src.filename,
            height: src.height,
            proxy_url: src.proxy_url,
            size: src.size,
            url: src.url,
            width: src.width,
            content_type: src.content_type,
            ephemeral: src.ephemeral,
        }
    }
}

impl From<MessageType> for channel::MessageKind {
    fn from(src: MessageType) -> Self {
        use MessageType::*;

        match src {
            Regular => Self::Regular,
            GroupRecipientAddition => Self::GroupRecipientAddition,
            GroupRecipientRemoval => Self::GroupRecipientRemoval,
            GroupCallCreation => Self::GroupCallCreation,
            GroupNameUpdate => Self::GroupNameUpdate,
            GroupIconUpdate => Self::GroupIconUpdate,
            PinsAdd => Self::PinsAdd,
            MemberJoin => Self::MemberJoin,
            NitroBoost => Self::NitroBoost,
            NitroTier1 => Self::NitroTier1,
            NitroTier2 => Self::NitroTier2,
            NitroTier3 => Self::NitroTier3,
            ChannelFollowAdd => Self::ChannelFollowAdd,
            GuildDiscoveryDisqualified => Self::GuildDiscoveryDisqualified,
            GuildDiscoveryRequalified => Self::GuildDiscoveryRequalified,
            GuildDiscoveryGracePeriodInitialWarning => {
                Self::GuildDiscoveryGracePeriodInitialWarning
            }
            GuildDiscoveryGracePeriodFinalWarning => Self::GuildDiscoveryGracePeriodFinalWarning,
            ThreadCreated => Self::ThreadCreated,
            InlineReply => Self::InlineReply,
            ApplicationCommand => Self::ApplicationCommand,
            ThreadStarterMessage => Self::ThreadStarterMessage,
            _ => Self::Unknown,
        }
    }
}

impl From<Embed> for channel::Embed {
    fn from(src: Embed) -> Self {
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
            title: src.title,
            url: src.url,
            video: src.video.map(|v| v.into()),
        }
    }
}

impl From<EmbedAuthor> for channel::EmbedAuthor {
    fn from(src: EmbedAuthor) -> Self {
        Self {
            icon_url: src.icon_url,
            name: src.name,
            proxy_icon_url: src.proxy_icon_url,
            url: src.url,
        }
    }
}

impl From<EmbedField> for channel::EmbedField {
    fn from(src: EmbedField) -> Self {
        Self {
            inline: src.inline,
            name: src.name,
            value: src.value,
        }
    }
}

impl From<EmbedFooter> for channel::EmbedFooter {
    fn from(src: EmbedFooter) -> Self {
        Self {
            icon_url: src.icon_url,
            proxy_icon_url: src.proxy_icon_url,
            text: src.text,
        }
    }
}

impl From<EmbedImage> for channel::EmbedImage {
    fn from(src: EmbedImage) -> Self {
        Self {
            height: src.height,
            proxy_url: src.proxy_url,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<EmbedProvider> for channel::EmbedProvider {
    fn from(src: EmbedProvider) -> Self {
        Self {
            name: src.name,
            url: src.url,
        }
    }
}

impl From<EmbedThumbnail> for channel::EmbedThumbnail {
    fn from(src: EmbedThumbnail) -> Self {
        Self {
            height: src.height,
            proxy_url: src.proxy_url,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<EmbedVideo> for channel::EmbedVideo {
    fn from(src: EmbedVideo) -> Self {
        Self {
            height: src.height,
            url: src.url,
            width: src.width,
        }
    }
}

impl From<MessageReference> for channel::MessageReference {
    fn from(src: MessageReference) -> Self {
        Self {
            message_id: src.message_id.map(|v| v.into()),
            channel_id: src.channel_id.into(),
            guild_id: src.guild_id.map(|v| v.into()),
        }
    }
}

impl From<MessageReaction> for channel::MessageReaction {
    fn from(src: MessageReaction) -> Self {
        Self {
            count: src.count,
            me: src.me,
            reaction_type: src.reaction_type.into(),
        }
    }
}

impl From<ReactionType> for channel::ReactionType {
    fn from(src: ReactionType) -> Self {
        match src {
            ReactionType::Custom { animated, id, name } => Self::Custom {
                animated,
                id: id.into(),
                name,
            },
            ReactionType::Unicode(s) => Self::Unicode(s),
            _ => unreachable!(),
        }
    }
}

impl From<Reaction> for channel::Reaction {
    fn from(src: Reaction) -> Self {
        Self {
            channel_id: src.channel_id.into(),
            emoji: src.emoji.into(),
            message_id: src.message_id.into(),
            user_id: src.user_id.map(|v| v.into()),
            guild_id: src.guild_id.map(|v| v.into()),
            member: src.member.map(|v| v.into()),
        }
    }
}

impl From<ChannelType> for channel::ChannelKind {
    fn from(src: ChannelType) -> Self {
        use ChannelType::*;

        match src {
            Text => Self::Text,
            Private => Self::Private,
            Voice => Self::Voice,
            Category => Self::Category,
            News => Self::News,
            Store => Self::Store,
            NewsThread => Self::NewsThread,
            PublicThread => Self::PublicThread,
            PrivateThread => Self::PrivateThread,
            Stage => Self::Stage,
            _ => Self::Unknown,
        }
    }
}

impl From<GuildChannel> for channel::GuildChannel {
    fn from(src: GuildChannel) -> Self {
        Self {
            id: src.id.into(),
            bitrate: src.bitrate,
            category_id: src.category_id.map(|v| v.into()),
            guild_id: src.guild_id.into(),
            kind: src.kind.into(),
            last_message_id: src.last_message_id.map(|v| v.into()),
            last_pin_timestamp: src.last_pin_timestamp,
            name: src.name,
            permission_overwrites: src
                .permission_overwrites
                .into_iter()
                .map(|v| v.into())
                .collect(),
            position: src.position,
            topic: src.topic,
            user_limit: src.user_limit,
            nsfw: src.nsfw,
        }
    }
}

impl From<PermissionOverwrite> for channel::PermissionOverwrite {
    fn from(src: PermissionOverwrite) -> Self {
        Self {
            allow: src.allow.into(),
            deny: src.deny.into(),
            kind: src.kind.into(),
        }
    }
}

impl From<PermissionOverwriteType> for channel::PermissionOverwriteKind {
    fn from(src: PermissionOverwriteType) -> Self {
        match src {
            PermissionOverwriteType::Member(user_id) => Self::Member(user_id.into()),
            PermissionOverwriteType::Role(role_id) => Self::Role(role_id.into()),
            v => panic!("Unknown PermissionOverwriteType: {:?}", v),
        }
    }
}

impl From<Channel> for channel::Channel {
    fn from(src: Channel) -> Self {
        match src {
            Channel::Guild(channel) => Self::Guild(channel.into()),
            Channel::Private(channel) => Self::Private(channel.into()),
            Channel::Category(channel) => Self::Category(channel.into()),
            v => panic!("Unknown Channel: {:?}", v),
        }
    }
}

impl From<PrivateChannel> for channel::PrivateChannel {
    fn from(src: PrivateChannel) -> Self {
        Self {
            id: src.id.into(),
            last_message_id: src.last_message_id.map(|v| v.into()),
            last_pin_timestamp: src.last_pin_timestamp,
            kind: src.kind.into(),
            recipient: src.recipient.into(),
        }
    }
}

impl From<ChannelCategory> for channel::CategoryChannel {
    fn from(src: ChannelCategory) -> Self {
        Self {
            id: src.id.into(),
            guild_id: src.guild_id.into(),
            category_id: src.category_id.map(|v| v.into()),
            position: src.position,
            kind: src.kind.into(),
            name: src.name,
            nsfw: src.nsfw,
            permission_overwrites: src
                .permission_overwrites
                .into_iter()
                .map(|v| v.into())
                .collect(),
        }
    }
}
