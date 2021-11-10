use crate::bot::Error;
use std::{iter::FromIterator, ops::Index, str::FromStr};

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

    /// Removes and returns the element at position `index`.
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> String {
        self.0.remove(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut String> {
        self.0.iter_mut()
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
    ///     assert_eq!(arg1, "hello");
    ///     assert_eq!(arg2, 123i64);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn pop_first<T>(&mut self) -> Result<T, Error>
    where
        T: FromStr,
    {
        match self.is_empty() {
            false => {
                let item = self.remove(0);

                item.parse().or(Err(Error::InvalidCommandUsage))
            }
            true => Err(Error::InvalidCommandUsage),
        }
    }

    /// Join the rest of arguments into a single argument. If the list
    /// is empty or parsing fails an `[Error::InvalidCommandUsage]` error
    /// is returned.
    ///
    /// This allows for easy extraction using the `?` operator.
    /// # Examples
    /// ```
    /// use robbot::Arguments;
    /// # fn main() -> Result<(), robbot::Error> {
    ///     let mut args = Arguments::from(vec!["Hello", "World"]);
    ///
    ///     let arg: String = args.join_rest()?;
    ///     assert_eq!(arg, "Hello World");
    ///
    /// #   Ok(())
    /// # }
    /// ```
    pub fn join_rest<T>(&mut self) -> Result<T, Error>
    where
        T: FromStr,
    {
        match self.is_empty() {
            false => {
                let mut items = Vec::new();
                for _ in 0..self.len() {
                    items.push(self.remove(0));
                }

                items.join(" ").parse().or(Err(Error::InvalidCommandUsage))
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

impl Index<usize> for Arguments {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IntoIterator for Arguments {
    type Item = String;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<I> FromIterator<I> for Arguments
where
    I: ToString,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = I>,
    {
        Self(iter.into_iter().map(|item| item.to_string()).collect())
    }
}

impl AsRef<[String]> for Arguments {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

pub struct IntoIter(Arguments);

impl Iterator for IntoIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.is_empty() {
            false => Some(self.0.remove(0)),
            true => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Arguments;

    #[test]
    fn test_arguments() {
        let arguments = Arguments::from(vec!["Hello", "123"]);
        assert_eq!(arguments, vec!["Hello", "123"]);
    }
}
