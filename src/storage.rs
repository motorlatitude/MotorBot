pub mod database;

// Re-exporting the Database struct for easier access
#[allow(unused)]
pub use database::{Database, GuildConfig, GuildConfigKey, GuildConfigValue};