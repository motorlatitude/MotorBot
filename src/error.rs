use derive_more::{Display, From};

use crate::{plugin, storage};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, From, Display)]
#[non_exhaustive]
pub enum Error {
    /// An error indicating that a plugin failed to send an event due to an
    /// issue with the Serenity context or API.
    #[display("Failed to send event: {}", err)]
    FailedToSendEvent {
        /// The error that occurred during sending the event
        err: serenity::Error,
    },

    // -- Internal
    /// An error related to storage operations, such as database interactions.
    #[from]
    Storage(storage::StorageError),
    /// An error related to plugin operations.
    #[from]
    Plugin(plugin::PluginError),

    // -- External
    /// An error related to the Serenity library, such as issues with the
    /// Discord API.
    #[from]
    Serenity(serenity::Error),

    /// An error related to the reqwest library, such as issues with making
    /// HTTP requests.
    #[from]
    Reqwest(reqwest::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Storage(storage::StorageError::from(e))
    }
}

impl std::error::Error for Error {}
