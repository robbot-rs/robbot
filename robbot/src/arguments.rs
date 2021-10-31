use crate::bot::Error;
use std::convert::From;
use std::str::FromStr;

/// An alias for `[Arguments]`.
pub type Args = Arguments;

#[derive(Clone, Debug, Default)]
pub struct Arguments(Vec<String>);

impl Arguments {
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

    /// Pop the first argument from the list. If it
    /// is empty or parsing the argument failed an
    /// `[Error::InvalidCommandUsage]` error is returned.
    ///
    /// This allows for easy extraction using the `?` operator.
    /// # Examples
    /// ```
    /// let mut args = Arguments::from(vec!["hello", "123"]);
    ///
    /// let arg1: String = args.take()?;
    /// let arg2: i64 = arg.take()?;
    /// ```
    pub fn take<T>(&mut self) -> Result<T, Error>
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

impl<T> From<T> for Arguments
where
    T: Iterator<Item = String>,
{
    fn from(t: T) -> Self {
        Self(t.collect())
    }
}
