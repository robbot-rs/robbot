use crate as robbot;
use crate::model::channel::MessageReference;
use crate::util::color::Color;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};
use std::convert::{From, Into};

use serenity::model::id::{ChannelId, RoleId};

/// [`CreateMessage`] is used to construct a new
/// message.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct CreateMessage {
    content: Option<String>,
    reference_message: Option<MessageReference>,
    embed: Option<CreateEmbed>,
}

impl CreateMessage {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut builder = Self::default();
        f(&mut builder);
        builder
    }

    /// Set the content of the message.
    pub fn content<T>(&mut self, content: T) -> &mut Self
    where
        T: ToString,
    {
        self.content = Some(content.to_string());
        self
    }

    pub fn reference_message<T>(&mut self, reference: T) -> &mut Self
    where
        T: Into<MessageReference>,
    {
        self.reference_message = Some(reference.into());
        self
    }

    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed),
    {
        self.embed = Some(CreateEmbed::new(f));
        self
    }

    pub fn fill_builder(self, builder: &mut serenity::builder::CreateMessage) {
        if let Some(content) = self.content {
            builder.content(content);
        }

        if let Some(reference_message) = self.reference_message {
            builder.reference_message((
                serenity::model::id::ChannelId(reference_message.channel_id.0),
                serenity::model::id::MessageId(reference_message.message_id.unwrap().0),
            ));
        }

        if let Some(embed) = self.embed {
            builder.embed(|e| {
                embed.fill_builder(e);
                e
            });
        }
    }
}

impl<T> From<T> for CreateMessage
where
    T: AsRef<str>,
{
    fn from(t: T) -> Self {
        let mut builder = Self::default();
        builder.content(t.as_ref());
        builder
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct CreateEmbed {
    color: Option<Color>,
    description: Option<String>,
    title: Option<String>,
}

impl CreateEmbed {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut builder = Self::default();
        f(&mut builder);
        builder
    }

    pub fn description<T>(&mut self, description: T) -> &mut Self
    where
        T: ToString,
    {
        self.description = Some(description.to_string());
        self
    }

    pub fn title<T>(&mut self, title: T) -> &mut Self
    where
        T: ToString,
    {
        self.title = Some(title.to_string());
        self
    }

    pub fn color<T>(&mut self, color: T) -> &mut Self
    where
        T: Into<Color>,
    {
        self.color = Some(color.into());
        self
    }

    pub fn fill_builder(self, builder: &mut serenity::builder::CreateEmbed) {
        if let Some(description) = self.description {
            builder.description(description);
        }

        if let Some(title) = self.title {
            builder.title(title);
        }

        if let Some(color) = self.color {
            builder.color(color.0);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EditMember {
    deafen: Option<bool>,
    mute: Option<bool>,
    nickname: Option<String>,
    roles: Option<Vec<RoleId>>,
    voice_channel: Option<ChannelId>,
    voice_disconnect: Option<bool>,
}

impl EditMember {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut builder = Self::default();
        f(&mut builder);
        builder
    }

    pub fn deafen(&mut self, deafen: bool) -> &mut Self {
        self.deafen = Some(deafen);
        self
    }

    pub fn mute(&mut self, mute: bool) -> &mut Self {
        self.mute = Some(mute);
        self
    }

    pub fn nickname<T>(&mut self, nickname: T) -> &mut Self
    where
        T: ToString,
    {
        self.nickname = Some(nickname.to_string());
        self
    }

    pub fn roles<I, T>(&mut self, roles: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<RoleId>,
    {
        self.roles = Some(roles.into_iter().map(|role| *role.as_ref()).collect());
        self
    }

    pub fn voice_channel(&mut self, channel_id: ChannelId) -> &mut Self {
        self.voice_disconnect = None;
        self.voice_channel = Some(channel_id);
        self
    }

    pub fn voice_disconnect(&mut self) -> &mut Self {
        self.voice_channel = None;
        self.voice_disconnect = Some(true);
        self
    }

    pub fn fill_builder(self, builder: &mut serenity::builder::EditMember) {
        if let Some(deafen) = self.deafen {
            builder.deafen(deafen);
        }

        if let Some(mute) = self.mute {
            builder.mute(mute);
        }

        if let Some(nickname) = self.nickname {
            builder.nickname(nickname);
        }

        if let Some(roles) = self.roles {
            builder.roles(roles);
        }

        if let Some(voice_channel) = self.voice_channel {
            builder.voice_channel(voice_channel);
        }

        if self.voice_disconnect.is_some() {
            builder.disconnect_member();
        }
    }
}
