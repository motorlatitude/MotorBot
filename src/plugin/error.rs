use derive_more::{Display, From};

/// Custom error type for plugin operations.
#[derive(Debug, Display, From)]
#[non_exhaustive]
pub enum Error {
    /// An error indicating that a plugin failed to send a message or response
    /// due to an issue with the Serenity context or API.
    #[display("Failed to send response: {}", err)]
    FailedToRespond {
        /// The error that occurred during sending a message or response
        err: serenity::Error,
    },

    /// Plugin expected a UserId but got None
    #[display("Expected a UserId but got None")]
    MissingUserId,

    /// Plugin expected a ChannelId but got None
    #[display("Expected a ChannelId but got None")]
    MissingChannelId,

    /// Plugin expected an environment variable to be set but it was missing
    #[display("Missing environment variable: {}", var_name)]
    MissingEnvironmentVariable {
        /// The name of the missing environment variable
        var_name: String,
    },

    /// Plugin expected something to happen in a guild context but it was used
    /// somewhere else (e.g. in a DM)
    ExpectedGuild,

    /// An error indicating that an internal API response was invalid or could
    /// not be parsed as expected, such as when calling an external API for a
    /// plugin's functionality and receiving a response that doesn't match the
    /// expected format or contains invalid data.
    #[display("Received an invalid response from an internal API: {}", err)]
    InvalidInternalAPIResponse {
        /// The error that occurred during parsing or handling the API response
        err: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A channel was expected but the provided option was either not a channel
    /// or was missing entirely.
    #[display(
        "Expected a channel option but got an invalid value or none at all"
    )]
    InvalidChannel,

    /// A user was expected but the provided option was either not a user
    /// or was missing entirely.
    #[display(
        "Expected a user option but got an invalid value or none at all"
    )]
    InvalidUser,

    /// An error indicating that a subcommand option was either not a subcommand
    /// or was missing entirely when one was expected.
    #[display(
        "Expected a subcommand option but got an invalid value or none at all"
    )]
    InvalidSubCommand,

    /// An error indicating that the plugin is in an invalid internal state,
    /// such as missing necessary context or configuration that should have been
    /// set during initialization or operation.
    #[display("Plugin is in an invalid internal state")]
    InvalidInternalState,
}

impl std::error::Error for Error {}
