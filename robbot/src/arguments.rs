use crate::bot::Error;
use crate::model::id::{ChannelId, RoleId, UserId};

use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::str::FromStr;

pub trait ArgumentsExt: AsRef<[String]> {
    /// Returns the number of arguments.
    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<&String>;

    /// Pops and returns the first argument. Returns `None` if no
    /// arguments are avaliable.
    fn pop(&mut self) -> Option<String>;

    /// Returns `true` if no arguments are avaliable.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn pop_parse<T>(&mut self) -> Result<T, Error>
    where
        T: FromStr,
    {
        match self.pop() {
            Some(item) => item.parse().or(Err(Error::InvalidCommandUsage)),
            None => Err(Error::InvalidCommandUsage),
        }
    }

    fn join_rest<T>(&mut self) -> Result<T, Error>
    where
        T: FromStr,
    {
        let items = self.as_ref().join(" ");

        items.parse().or(Err(Error::InvalidCommandUsage))
    }
}

/// An alias for [`Arguments`].
pub type Args<'life0> = Arguments<'life0>;

/// A immutable view into [`OwnedArguments`].
#[derive(Copy, Clone, Debug)]
pub struct Arguments<'life0>(&'life0 [String]);

impl<'life0> Arguments<'life0> {
    /// Create a new empty argument list.
    pub fn new(args: &'life0 [String]) -> Self {
        Self(args)
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` is no arguments are stored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // /// Removes and returns the element at position `index`.
    // /// # Panics
    // /// Panics if `index` is out of bounds.
    // pub fn pop(&mut self) -> Option<&String> {
    //     match self.is_empty() {
    //         false => {
    //             let elem = &self.0[0];
    //             self.0 = &self.0[1..];
    //             Some(elem)
    //         }
    //         true => None,
    //     }
    // }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }
}

impl<'life0> ArgumentsExt for Arguments<'life0> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> Option<&String> {
        self.0.get(index)
    }

    fn pop(&mut self) -> Option<String> {
        match self.is_empty() {
            false => {
                let elem = &self.0[0];
                self.0 = &self.0[1..];
                Some(elem.to_owned())
            }
            true => None,
        }
    }
}

impl<'life0> AsRef<[String]> for Arguments<'life0> {
    fn as_ref(&self) -> &[String] {
        self.0
    }
}

impl<'life0> Deref for Arguments<'life0> {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'life0> Iterator for Arguments<'life0> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

// impl<'life0, T> PartialEq<T> for Arguments<'life0>
// where
//     T: AsRef<[String]>,
// {
//     fn eq(&self, other: &T) -> bool {
//         self.0.eq(other.as_ref())
//     }
// }

impl<'life0> Index<usize> for Arguments<'life0> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'life0, 'life1, T> PartialEq<T> for Arguments<'life0>
where
    T: AsRef<[&'life1 str]>,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'life0> Display for Arguments<'life0> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

/// A list of owned arguments.
#[derive(Clone, Debug, Default)]
pub struct OwnedArguments(Vec<String>);

impl OwnedArguments {
    /// Creates a new empty `OwnedArguments` list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new empty `OwnedArguments` list with the specified capacity.
    ///
    /// # Panics
    /// Panics if `capacity` exceeds `isize::MAX` bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns a new [`Arguments`] slice over all arguments.
    pub fn as_args(&self) -> Arguments {
        Arguments::new(&self.0)
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no arguments are stored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends a new argument at the end of the list.
    pub fn push(&mut self, item: String) {
        self.0.push(item);
    }

    /// Inserts a new argument at position `index`, shifting all
    /// arguments after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, item: String) {
        self.0.insert(index, item);
    }

    /// Removes the argument at positon `index`, shifting all
    /// arguments after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> String {
        self.0.remove(index)
    }
}

impl AsRef<[String]> for OwnedArguments {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

impl AsMut<[String]> for OwnedArguments {
    fn as_mut(&mut self) -> &mut [String] {
        self.0.as_mut()
    }
}

impl AsRef<Vec<String>> for OwnedArguments {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl AsMut<Vec<String>> for OwnedArguments {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

impl Deref for OwnedArguments {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for OwnedArguments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl Index<usize> for OwnedArguments {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for OwnedArguments {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'life0, T> PartialEq<T> for OwnedArguments
where
    T: AsRef<[&'life0 str]>,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<I> FromIterator<I> for OwnedArguments
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

impl Display for OwnedArguments {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_args().fmt(f)
    }
}

/// A wrapper around [`OwnedArguments`] that does not consume
/// popped arguments.
///
/// `CommandArguments` behaves exactly the same as [`OwnedArguments`] from the
/// outside, but it does not consume any arguments returned from [`pop`] and provides
/// an extra method [`as_full_args`] which returns an [`Arguments`] view that includes
/// all arguments that were already popped.
///
/// [`pop`]: Self::pop
/// [`as_full_args`]: Self::as_full_args
#[derive(Clone, Debug, Default)]
pub struct CommandArguments {
    owned: OwnedArguments,
    offset: usize,
}

impl CommandArguments {
    /// Creates a new `CommandArguments` list.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_args(&self) -> Arguments {
        Arguments::new(self.as_ref())
    }

    /// Returns a reference to the inner [`OwnedArguments`].
    pub fn as_owned(&self) -> &OwnedArguments {
        &self.owned
    }

    /// Returns a mutable reference to the inner [`OwnedArguments`].
    pub fn as_mut_owned(&mut self) -> &mut OwnedArguments {
        &mut self.owned
    }

    /// Converts `self` into an [`OwnedArguments`] list.
    ///
    /// **Note:** The returned [`OwnedArguments`] contain the inner
    /// arguments, not the slice.
    pub fn into_owned(self) -> OwnedArguments {
        self.owned
    }

    /// Returns an [`Arguments`] list containing all arguments before
    /// any routing was applied.
    pub fn as_full_args(&self) -> Arguments {
        Arguments::new(&self.owned)
    }
}

impl AsRef<[String]> for CommandArguments {
    fn as_ref(&self) -> &[String] {
        let slice: &[String] = self.owned.as_ref();
        &slice[self.offset..]
    }
}

impl ArgumentsExt for CommandArguments {
    fn len(&self) -> usize {
        self.owned.len() - self.offset
    }

    fn get(&self, index: usize) -> Option<&String> {
        self.owned.get(index + self.offset)
    }

    fn pop(&mut self) -> Option<String> {
        match self.owned.get(self.offset) {
            Some(arg) => {
                self.offset += 1;
                Some(arg.to_owned())
            }
            None => None,
        }
    }
}

impl<'life0, T> PartialEq<T> for CommandArguments
where
    T: AsRef<[&'life0 str]>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Iterator for CommandArguments {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl From<OwnedArguments> for CommandArguments {
    fn from(args: OwnedArguments) -> Self {
        Self {
            owned: args,
            offset: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InvalidMention;

/// A channel mention with the format `<#{id}>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChannelMention {
    pub id: ChannelId,
}

impl ChannelMention {
    /// Creates a new `ChannelMention`.
    pub fn new<T>(id: T) -> Self
    where
        T: Into<ChannelId>,
    {
        Self { id: id.into() }
    }
}

impl Display for ChannelMention {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<#{}>", self.id.0)
    }
}

impl FromStr for ChannelMention {
    type Err = InvalidMention;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<#") || !s.ends_with('>') {
            return Err(InvalidMention);
        }

        // SAFETY: The subslice is in bounds of the original slice.
        let id = unsafe { s.get_unchecked(2..s.len() - 1) };

        let id = id.parse().or(Err(InvalidMention))?;

        Ok(Self { id: ChannelId(id) })
    }
}

/// A role mention with the format `<@&{id}>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RoleMention {
    pub id: RoleId,
}

impl RoleMention {
    /// Creates a new `RoleMention`.
    pub fn new<T>(id: T) -> Self
    where
        T: Into<RoleId>,
    {
        Self { id: id.into() }
    }
}

impl Display for RoleMention {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<@&{}>", self.id.0)
    }
}

impl FromStr for RoleMention {
    type Err = InvalidMention;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<@&") || !s.ends_with('>') {
            return Err(InvalidMention);
        }

        // SAFETY: The subslice is in bounds of the original slice.
        let id = unsafe { s.get_unchecked(3..s.len() - 1) };

        let id = id.parse().or(Err(InvalidMention))?;

        Ok(Self { id: RoleId(id) })
    }
}

/// A user mention with the format `<@{id}>` or `<@!{id}>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UserMention {
    pub id: UserId,
}

impl UserMention {
    /// Creates a new `UserMention`.
    pub fn new<T>(id: T) -> Self
    where
        T: Into<UserId>,
    {
        Self { id: id.into() }
    }
}

impl FromStr for UserMention {
    type Err = InvalidMention;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<@") || !s.ends_with('>') {
            return Err(InvalidMention);
        }

        // SAFETY: The subslice is in bounds of the original slice.
        let id = unsafe { s.get_unchecked(2..s.len() - 1) };

        let id = match id.get(0..1) {
            // SAFETY: The subslice is in bounds of the original slice.
            Some("!") => unsafe { id.get_unchecked(1..id.len()) },
            _ => id,
        };

        let id = id.parse().or(Err(InvalidMention))?;

        Ok(Self { id: UserId(id) })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArgumentsExt, ChannelMention, CommandArguments, InvalidMention, OwnedArguments,
        RoleMention, UserMention,
    };

    #[test]
    fn test_owned_arguments() {
        let mut arguments: OwnedArguments = vec!["Hello", "123"].iter().collect();
        assert_eq!(arguments, vec!["Hello", "123"]);

        arguments.push(String::from("arg3"));
        assert_eq!(arguments, vec!["Hello", "123", "arg3"]);
    }

    #[test]
    fn test_command_arguments() {
        let args: OwnedArguments = vec!["Hello", "123", "arg3"].iter().collect();
        let mut args = CommandArguments::from(args);

        assert_eq!(args, vec!["Hello", "123", "arg3"]);
        assert_eq!(args.len(), 3);

        assert_eq!(args.pop().unwrap(), "Hello");
        assert_eq!(args, vec!["123", "arg3"]);
        assert_eq!(args.len(), 2);

        let mut args_ref = args.as_args();
        assert_eq!(args_ref, vec!["123", "arg3"]);
        assert_eq!(args_ref.len(), 2);

        assert_eq!(args_ref.pop().unwrap(), "123");
        assert_eq!(args_ref, vec!["arg3"]);
        assert_eq!(args_ref.len(), 1);
    }

    #[test]
    fn test_channel_mention() {
        let s = "<#12345>";
        assert_eq!(
            s.parse::<ChannelMention>().unwrap(),
            ChannelMention::new(12345)
        );

        let s = "<#1>";
        assert_eq!(s.parse::<ChannelMention>().unwrap(), ChannelMention::new(1));

        let s = "<1234>";
        assert_eq!(s.parse::<ChannelMention>().unwrap_err(), InvalidMention);

        let s = "<#1234";
        assert_eq!(s.parse::<ChannelMention>().unwrap_err(), InvalidMention);
    }

    #[test]
    fn test_role_mention() {
        let s = "<@&12345>";
        assert_eq!(s.parse::<RoleMention>().unwrap(), RoleMention::new(12345));

        let s = "<@&1>";
        assert_eq!(s.parse::<RoleMention>().unwrap(), RoleMention::new(1));

        let s = "<&1234>";
        assert_eq!(s.parse::<RoleMention>().unwrap_err(), InvalidMention);

        let s = "<@&1234";
        assert_eq!(s.parse::<RoleMention>().unwrap_err(), InvalidMention);
    }

    #[test]
    fn test_user_mention() {
        let s = "<@12345>";
        assert_eq!(s.parse::<UserMention>().unwrap(), UserMention::new(12345));

        let s = "<@!1>";
        assert_eq!(s.parse::<UserMention>().unwrap(), UserMention::new(1));

        let s = "<1234>";
        assert_eq!(s.parse::<UserMention>().unwrap_err(), InvalidMention);

        let s = "<@1234";
        assert_eq!(s.parse::<UserMention>().unwrap_err(), InvalidMention);

        let s = "<@!>";
        assert_eq!(s.parse::<UserMention>().unwrap_err(), InvalidMention);
    }
}
