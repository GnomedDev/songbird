//! Newtypes around Discord IDs for library cross-compatibility.

#[cfg(feature = "driver-core")]
use crate::model::id::{GuildId as DriverGuild, UserId as DriverUser};
#[cfg(feature = "serenity")]
use serenity::model::id::{
    ChannelId as SerenityChannel,
    GuildId as SerenityGuild,
    UserId as SerenityUser,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    num::NonZeroU64,
};
#[cfg(feature = "twilight")]
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id as TwilightId,
};

/// ID of a Discord voice/text channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChannelId(pub NonZeroU64);

/// ID of a Discord guild (colloquially, "server").
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GuildId(pub NonZeroU64);

/// ID of a Discord user.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UserId(pub NonZeroU64);

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Creates a new Id from a u64
                ///
                /// # Panics
                /// Panics if the id is zero.
                #[must_use]
                pub fn new(id_as_u64: u64) -> Self {
                    Self(NonZeroU64::new(id_as_u64).unwrap())
                }

                /// Retrieves the inner ID as u64
                #[must_use]
                pub fn get(self) -> u64 {
                    self.0.get()
                }
            }

            impl From<u64> for $name {
                fn from(id: u64) -> Self {
                    $name::new(id)
                }
            }

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    Display::fmt(&self.0, f)
                }
            }
        )*
    }
}

#[cfg(feature = "serenity")]
macro_rules! id_u64_serenity {
    ($($name:ident, $serenity:ident;)*) => {
        $(
            impl From<$serenity> for $name {
                fn from(id: $serenity) -> Self {
                    Self(id.0)
                }
            }
        )*
    }
}

#[cfg(feature = "twilight")]
macro_rules! id_u64_twilight {
    ($($name:ident, $twilight:ident;)*) => {
        $(
            impl From<TwilightId<$twilight>> for $name {
                fn from(id: TwilightId<$twilight>) -> Self {
                    Self(id.into_nonzero())
                }
            }
        )*
    }
}

id_u64! {ChannelId; GuildId; UserId;}

#[cfg(feature = "serenity")]
id_u64_serenity!(
    ChannelId, SerenityChannel;
    GuildId, SerenityGuild;
    UserId, SerenityUser;
);

#[cfg(feature = "twilight")]
id_u64_twilight!(
    ChannelId, ChannelMarker;
    GuildId, GuildMarker;
    UserId, UserMarker;
);

#[cfg(feature = "driver-core")]
impl From<GuildId> for DriverGuild {
    fn from(id: GuildId) -> Self {
        Self(id.get())
    }
}

#[cfg(feature = "driver-core")]
impl From<UserId> for DriverUser {
    fn from(id: UserId) -> Self {
        Self(id.get())
    }
}
