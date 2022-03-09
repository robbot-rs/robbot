use std::fmt::{self, Debug, Formatter};
use std::mem;

/// An `Option<T>` but with the same size as `T`. This means not every value of `T` can be used
/// to represent a `Some` value.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SmallOption<T>
where
    T: SmallOptionValue,
{
    value: T,
}

impl<T> SmallOption<T>
where
    T: SmallOptionValue,
{
    pub const NONE: Self = Self { value: T::NONE };

    /// Creates a new `SmallOption` for an equivalent [`Option`].
    ///
    /// # Panics
    /// Panics `value` is `Some` and `T` cannot be represented by an `SmallOption`.
    pub fn new(value: Option<T>) -> Self {
        match value {
            Some(value) => Self {
                value: T::some(value),
            },
            None => Self::default(),
        }
    }

    /// Creates a new `SmallOption` using `value` as the `Some` value without validating
    /// `value`.
    ///
    /// # Safety
    /// If `value` is not valid it might collide with the `None` value, causing this `SmallOption`
    /// to return broken values.
    #[inline]
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self { value }
    }

    /// Returns `true` if the [`SmallOption`] contains a value.
    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Returns `true` if the [`SmallOption`] contains no value.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.value == T::NONE
    }

    pub fn and<U>(self, optb: SmallOption<U>) -> SmallOption<U>
    where
        U: SmallOptionValue,
    {
        if self.is_some() {
            optb
        } else {
            SmallOption::default()
        }
    }

    pub fn and_then<U, F>(self, f: F) -> SmallOption<U>
    where
        U: SmallOptionValue,
        F: FnOnce(T) -> SmallOption<U>,
    {
        if self.is_some() {
            f(self.value)
        } else {
            SmallOption::default()
        }
    }

    pub fn or(self, optb: Self) -> Self {
        if self.is_some() {
            self
        } else {
            optb
        }
    }

    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        if self.is_some() {
            self
        } else {
            f()
        }
    }

    pub fn inspect<F>(&self, f: F)
    where
        F: FnOnce(&T),
    {
        if self.is_some() {
            f(&self.value);
        }
    }

    /// Takes the value out of the [`SmallOption`] and replaces it with a `None` value.
    /// Returns the value taken out.
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    /// Returns the contained `Some` value.
    ///
    /// # Panics
    /// Panics if the value is a `None` with the message provided by `msg`.
    pub fn expect(self, msg: &str) -> T {
        if self.is_some() {
            self.value
        } else {
            panic!("{}", msg);
        }
    }

    #[inline]
    pub fn unwrap(self) -> T {
        if self.is_some() {
            self.value
        } else {
            panic!("called `SmallOption::unwrap()` on a `None` value")
        }
    }

    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        if self.is_some() {
            self.value
        } else {
            default
        }
    }

    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if self.is_some() {
            self.value
        } else {
            f()
        }
    }

    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        if self.is_some() {
            self.value
        } else {
            T::default()
        }
    }

    #[inline]
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.value
    }
}

impl<T> Default for SmallOption<T>
where
    T: SmallOptionValue,
{
    fn default() -> Self {
        Self { value: T::NONE }
    }
}

impl<T> Debug for SmallOption<T>
where
    T: SmallOptionValue + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_some() {
            write!(f, "Some ({:?})", self.value)
        } else {
            write!(f, "None")
        }
    }
}

pub trait SmallOptionValue: Sized + PartialEq {
    /// The value for `None`.
    const NONE: Self;

    /// Creates a new `Some` value.
    ///
    /// # Panics
    /// Panics when `value` cannot be represented without colliding with the `None` value.
    fn some(value: Self) -> Self;
}

/// Use an `u8` as a SmallOptionValue where the MSD represent the `None` value.
/// The valid input range is 0..(2^7)-1.
impl SmallOptionValue for u8 {
    const NONE: Self = 0b10000000;

    fn some(value: Self) -> Self {
        if value > u8::pow(2, 7) - 1 {
            panic!("Tried to create SmallOption with a too big value");
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::{SmallOption, SmallOptionValue};

    #[test]
    fn test_small_option() {
        let opt: SmallOption<u8> = SmallOption::new(None);
        assert_eq!(opt.value, u8::NONE);
        assert!(opt.is_none());

        let opt: SmallOption<u8> = SmallOption::new(Some(0));
        assert_eq!(opt.value, 0);
        assert!(opt.is_some());

        let opt: SmallOption<u8> = SmallOption::new(Some(1));
        assert_eq!(opt.value, 1);
        assert!(opt.is_some());
    }
}
