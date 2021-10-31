use crate::bot::Error;
use std::{convert::From, iter::FromIterator, str::FromStr};

/// An alias for `[Arguments]`.
pub type Args = Arguments;

#[derive(Clone, Debug, Default)]
pub struct Arguments(Vec<String>);

impl Arguments {
    /// Create a new empty argument list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` is no arguments are stored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: usize) -> Option<&String> {
        self.0.get(i)
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut String> {
        self.0.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = String> {
        self.0.into_iter()
    }

    /// Pop the first argument from the list. If it
    /// is empty or parsing the argument failed an
    /// `[Error::InvalidCommandUsage]` error is returned.
    ///
    /// This allows for easy extraction using the `?` operator.
    /// # Examples
    /// ```
    /// use robbot::Arguments;
    /// # fn main() -> Result<(), robbot::Error> {
    ///     let mut args = Arguments::from(vec!["hello", "123"]);
    ///
    ///     let arg1: String = args.pop_first()?;
    ///     let arg2: i64 = args.pop_first()?;
    ///
    /// #   Ok(())
    /// # }
    /// ```
    pub fn pop_first<T>(&mut self) -> Result<T, Error>
    where
        T: FromStr,
    {
        match self.is_empty() {
            false => {
                let item = self.0.remove(0);

                item.parse().or(Err(Error::InvalidCommandUsage))
            }
            true => Err(Error::InvalidCommandUsage),
        }
    }
}

impl PartialEq<Vec<&str>> for Arguments {
    fn eq(&self, other: &Vec<&str>) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Vec<String>> for Arguments {
    fn eq(&self, other: &Vec<String>) -> bool {
        self.0.eq(other)
    }
}

impl<T, I> From<T> for Arguments
where
    T: IntoIterator<Item = I>,
    I: ToString,
{
    fn from(t: T) -> Self {
        Self(t.into_iter().map(|item| item.to_string()).collect())
    }
}