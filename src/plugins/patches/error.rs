use derive_more::Display;

use crate::plugins::patches::platforms::platform::Platform;

#[derive(Display, Debug)]
pub enum PatchesError {
    /// An error indicating that a game ID provided to the Patches plugin was
    /// invalid, such as not being a valid Steam game ID or not being found in
    /// the database when expected.
    #[display("Invalid game ID provided")]
    InvalidGameId,

    /// An error indicating that a game platform provided to the Patches plugin
    /// was invalid, such as not being a recognized platform like "steam" or
    /// "riot".
    #[display("Invalid game platform provided")]
    InvalidGamePlatform,

    /// An error indicating that a game name provided to the Patches plugin was
    /// invalid, such as being empty or not matching any known game names in the
    /// database when expected.
    #[display("Invalid game name provided")]
    InvalidGameName,

    /// An error indicating that a game thumbnail URL provided to the Patches
    /// plugin was invalid, such as not being a valid URL or not pointing to an
    /// image when expected.
    #[display("Invalid game thumbnail URL provided")]
    InvalidGameThumbnail,

    /// An error indicating that a game color provided to the Patches plugin was
    /// invalid, such as not being a valid hex color code when expected.
    #[display("Invalid game color provided")]
    InvalidGameColor,

    /// An error indicating that fetching patch notes for a game failed, such
    /// as when calling an external API to retrieve patch notes and receiving an
    /// error response or an invalid response that cannot be parsed as patch
    /// notes.
    #[display(
        "Failed to fetch patch notes for game '{}'on platform '{}'",
        game_id,
        platform
    )]
    FetchPatchNotesFailed {
        /// The platform for which fetching patch notes failed
        platform: Platform,
        game_id: String,
    },
}

impl std::error::Error for PatchesError {}
