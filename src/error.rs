use derive_more::{Display, From};

use crate::{plugin, storage};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, From, Display)]
#[non_exhaustive]
pub enum Error {
    // -- Internal
    /// An error related to storage operations, such as database interactions.
    #[from]
    Storage(storage::StorageError),
    /// An error related to plugin operations.
    #[from]
    Plugin(plugin::PluginError),

    // -- External
    /// An error related to the Serenity library, such as issues with the Discord API.
    #[from]
    Serenity(serenity::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Storage(storage::StorageError::from(e))
    }
}

impl std::error::Error for Error {}
