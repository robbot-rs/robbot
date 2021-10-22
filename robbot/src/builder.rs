use serde::{Deserialize, Serialize};
use serenity::utils::Color;
use std::convert::{From, Into};

/// [`CreateMessage`] is used to construct a new
/// message.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateMessage {
    content: Option<String>,
    reference_message: Option<serenity::model::channel::MessageReference>,
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
        T: Into<serenity::model::channel::MessageReference>,
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
            builder.reference_message(reference_message);
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
            builder.color(color);
        }
    }
}
