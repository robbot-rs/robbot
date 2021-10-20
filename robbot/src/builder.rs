use std::convert::{From, Into};

/// [`CreateMessage`] is used to construct a new
/// message.
#[derive(Clone, Debug, Default)]
pub struct CreateMessage {
    content: Option<String>,
    reference_message: Option<serenity::model::channel::MessageReference>,
}

impl CreateMessage {
    /// Set the content of the message.
    pub fn content<T>(&mut self, content: T)
    where
        T: ToString,
    {
        self.content = Some(content.to_string());
    }

    pub fn reference_message<T>(&mut self, reference: T)
    where
        T: Into<serenity::model::channel::MessageReference>,
    {
        self.reference_message = Some(reference.into());
    }

    pub fn fill_builder(self, builder: &mut serenity::builder::CreateMessage) {
        if let Some(content) = self.content {
            builder.content(content);
        }

        if let Some(reference_message) = self.reference_message {
            builder.reference_message(reference_message);
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
