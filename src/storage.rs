pub mod database;
pub mod error;
// Re-exporting the Database struct for easier access
#[allow(unused)]
pub use database::{Database, GuildConfig, GuildConfigKey, GuildConfigValue};
pub use error::Error as StorageError;
