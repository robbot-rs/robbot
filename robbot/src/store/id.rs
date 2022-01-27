//! Generated ID types

pub use snowflake::*;

mod snowflake {
    use crate::store::{Deserialize, Deserializer, Serialize, Serializer, Store, TypeSerializer};
    use chrono::Utc;

    use std::fmt::{self, Display, Formatter};

    const BITMASK_TIMESTAMP: u64 = 0xFFFFFFFFFF800000;
    const BITMASK_INSTANCE: u64 = 0x3FF000;
    const BITMASK_SEQUENCE: u64 = 0xFFF;

    /// The maximum value for the sequence section of the
    /// snowflake. (2^12 - 1 = 4095)
    const SEQUENCE_MAX: u64 = 4095;

    /// The maximum value for the instance section of the
    /// snowflake. (2^10 - 1 = 1023)
    const INSTANCE_MAX: u64 = 1023;

    /// A unique identifier.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct Snowflake(pub u64);

    impl Snowflake {
        const fn from_parts(timestamp: u64, instance: u64, sequence: u64) -> Self {
            let timestamp = (timestamp << 22) & BITMASK_TIMESTAMP;
            let instance = (instance << 12) & BITMASK_INSTANCE;

            Self(timestamp + instance + sequence)
        }

        /// Returns the timestamp component of the `Snowflake`.
        pub const fn timestamp(&self) -> u64 {
            (self.0 & BITMASK_TIMESTAMP) >> 22
        }

        /// Returns the instance component of the `Snowflake`.
        pub const fn instance(&self) -> u64 {
            (self.0 & BITMASK_INSTANCE) >> 12
        }

        /// Returns the sequence component of the `Snowflake`.
        pub const fn sequence(&self) -> u64 {
            self.0 & BITMASK_SEQUENCE
        }
    }

    impl Display for Snowflake {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    #[derive(Copy, Clone, Debug, Default)]
    pub struct SnowflakeGenerator {
        instance: u16,
        sequence: u16,
    }

    impl SnowflakeGenerator {
        /// Creates a new `SnowflakeGenerator` which yields
        /// snowflakes with the given `instance_id`.
        ///
        /// Note: If the `instance_id` value exceeds the maximum of
        /// 1023 (2^10 - 1), `None` is returned.
        pub const fn new(instance: u16) -> Option<Self> {
            if instance > INSTANCE_MAX as u16 {
                None
            } else {
                Some(Self::new_unchecked(instance))
            }
        }

        /// Creates a new `SnowflakeGenerator` which yields
        /// snowflakes with the given `instance_id`.
        ///
        /// Note: The caller must guarantee that `instance_id` does
        /// not exceed the maximum value of 1023 (2^10 -1).
        pub const fn new_unchecked(instance: u16) -> Self {
            Self {
                instance,
                sequence: 0,
            }
        }

        /// Yield a newly generated [`Snowflake`].
        pub fn yield_id(&mut self) -> Snowflake {
            let timestamp = Utc::now().timestamp_millis();

            let id =
                Snowflake::from_parts(timestamp as u64, self.instance as u64, self.sequence as u64);

            // Increment the sequence number.
            match self.sequence as u64 {
                SEQUENCE_MAX => self.sequence = 0,
                _ => self.sequence += 1,
            }

            id
        }
    }

    impl<T> Serialize<T> for Snowflake
    where
        T: Store,
        u64: Serialize<T>,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where
            S: Serializer<T>,
        {
            self.0.serialize(serializer)
        }

        fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
        where
            S: TypeSerializer<T>,
        {
            u64::serialize_type(serializer)
        }
    }

    impl<T> Deserialize<T> for Snowflake
    where
        T: Store,
        u64: Serialize<T>,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<T>,
        {
            let v = deserializer.deserialize_u64()?;

            Ok(Self(v))
        }
    }
}
