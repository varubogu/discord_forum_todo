//! Newtype wrappers around Discord snowflake IDs. The wrappers keep this crate
//! free of any `twilight-*` dependency; adapters perform the conversion.

use serde::{Deserialize, Serialize};

macro_rules! snowflake_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u64);

        impl $name {
            #[inline]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            #[inline]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                Self(v)
            }
        }

        impl From<$name> for u64 {
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}

snowflake_id!(GuildId);
snowflake_id!(ChannelId);
snowflake_id!(RoleId);
snowflake_id!(ThreadId);
snowflake_id!(MessageId);
snowflake_id!(CategoryId);
snowflake_id!(UserId);
