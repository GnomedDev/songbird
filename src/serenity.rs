//! Compatibility and convenience methods for working with [serenity].
//! Requires the `"serenity"` feature.
//!
//! [serenity]: https://crates.io/crates/serenity

use crate::{Config, Songbird};
use serenity::client::ClientBuilder;
use std::sync::Arc;

/// Installs a new songbird instance into the serenity client.
pub fn register(client_builder: ClientBuilder) -> ClientBuilder {
    let voice = Songbird::serenity();
    register_with(client_builder, voice)
}

/// Installs a given songbird instance into the serenity client.
pub fn register_with(client_builder: ClientBuilder, voice: Arc<Songbird>) -> ClientBuilder {
    client_builder.voice_manager_arc(voice)
}

/// Installs a given songbird instance into the serenity client.
pub fn register_from_config(client_builder: ClientBuilder, config: Config) -> ClientBuilder {
    let voice = Songbird::serenity_from_config(config);
    register_with(client_builder, voice)
}

/// Helper trait to add installation/creation methods to serenity's
/// `ClientBuilder`.
///
/// These install the client to receive gateway voice events, and
/// store an easily accessible reference to Songbird's managers.
pub trait SerenityInit {
    /// Registers a new Songbird voice system with serenity, storing it for easy
    /// access via [`get`].
    ///
    /// [`get`]: get
    #[must_use]
    fn register_songbird(self) -> Self;
    /// Registers a given Songbird voice system with serenity, as above.
    #[must_use]
    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self;
    /// Registers a Songbird voice system serenity, based on the given configuration.
    #[must_use]
    fn register_songbird_from_config(self, config: Config) -> Self;
}

impl SerenityInit for ClientBuilder {
    fn register_songbird(self) -> Self {
        register(self)
    }

    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self {
        register_with(self, voice)
    }

    fn register_songbird_from_config(self, config: Config) -> Self {
        register_from_config(self, config)
    }
}
