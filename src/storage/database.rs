use rusqlite::{params, Connection};
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::{
    plugins::patches::{
        game_data::{GameData, GuildGameData},
        platforms::platform::Platform,
    },
    storage::StorageError,
};

pub use crate::{Error, Result};
/// The current schema version of the database. This is used to manage
/// database migrations in the future. If the schema changes, this version
/// should be incremented, and migration logic should be added.
const SCHEMA_VERSION: u8 = 1;

/// Represents a user's score in the database,
/// including the user ID and their score.
#[derive(Debug)]
#[allow(dead_code)]
pub struct UserScore {
    /// The unique discord user ID
    pub user_id: u64,
    /// The score value for the user
    pub score: i32,
}

/// Represents a key for guild configuration options in the database. This enum
/// defines the different configuration options that can be set for a guild.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuildConfigKey {
    /// The channel ID for where patch notes should be posted in the guild
    PatchNotesChannel,
    /// The channel IDs where vote reactions should be added to messages
    PointsChannels,
    /// The channel ID for where jokes should be posted in the guild
    JokesChannel,
    /// The channel ID for where bot events should be posted in the guild
    EventsChannel,
}

impl AsRef<str> for GuildConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            GuildConfigKey::PatchNotesChannel => "PATCH_NOTES_CHANNEL",
            GuildConfigKey::PointsChannels => "POINTS_CHANNELS",
            GuildConfigKey::JokesChannel => "JOKES_CHANNEL",
            GuildConfigKey::EventsChannel => "EVENTS_CHANNEL",
        }
    }
}

/// Represents a value for guild configuration options in the database
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GuildConfigValue {
    ChannelId(u64),
    ChannelIds(Vec<u64>),
}

impl From<GuildConfigValue> for String {
    fn from(val: GuildConfigValue) -> Self {
        match val {
            GuildConfigValue::ChannelId(id) => id.to_string(),
            GuildConfigValue::ChannelIds(ids) => ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(","),
        }
    }
}

pub struct GuildConfig {
    pub key: GuildConfigKey,
    pub value: GuildConfigValue,
}

impl GuildConfig {
    pub fn key(&self) -> &GuildConfigKey {
        &self.key
    }

    pub fn value(&self) -> &GuildConfigValue {
        &self.value
    }
}

impl From<(GuildConfigKey, String)> for GuildConfig {
    fn from((key, value): (GuildConfigKey, String)) -> Self {
        let value = match key {
            GuildConfigKey::PatchNotesChannel => {
                let channel_id = value.parse::<u64>().unwrap_or(0);
                GuildConfigValue::ChannelId(channel_id)
            }
            GuildConfigKey::PointsChannels => {
                let channel_ids = value
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u64>().ok())
                    .collect();
                GuildConfigValue::ChannelIds(channel_ids)
            }
            GuildConfigKey::JokesChannel => {
                let channel_id = value.parse::<u64>().unwrap_or(0);
                GuildConfigValue::ChannelId(channel_id)
            }
            GuildConfigKey::EventsChannel => {
                let channel_id = value.parse::<u64>().unwrap_or(0);
                GuildConfigValue::ChannelId(channel_id)
            }
        };
        Self { key, value }
    }
}

impl From<(GuildConfigKey, GuildConfigValue)> for GuildConfig {
    fn from((key, value): (GuildConfigKey, GuildConfigValue)) -> Self {
        Self { key, value }
    }
}

/// Database struct that manages the SQLite
/// connection and provides methods for interacting
/// with the database.
pub struct Database {
    /// The SQLite connection to the database file.
    /// This connection is used for all database operations.
    ///
    /// If the connection is `None`, it indicates that the database
    /// is not currently connected. Utilising the connection in
    /// this state causes an InvalidConnectionState error to be
    /// returned.
    connection: Option<Connection>,
}

impl Database {
    /// Creates a new Database instance and establishes a connection
    /// to the SQLite database file.
    ///
    /// This method creates a new Database instance. If the database
    /// file does not exist, it will create a new one and set up the
    /// necessary tables. If the file already exists, it will simply
    /// open the connection after verifying that the schema version
    /// matches the expected version or carry out necessary migrations
    /// if it does not.
    ///
    /// ### File Path Configuration
    /// The file path can be configured using the `DATABASE_PATH` environment
    /// variable, which should point to a directory where the database
    /// file can be created and accessed. If the environment variable
    /// is not set, it defaults to `/data/`. The `data` directory is
    /// created in the Dockerfile by default.
    ///
    /// ## Returns
    /// * `Ok(Database)` - If the database connection is successfully
    ///   established and the schema version is valid, it returns a
    ///   Database instance.
    /// * `Err(Error)` - If there is an error during connection, schema
    ///   version validation, or table creation, it returns an appropriate
    ///   Error] variant.
    pub async fn open() -> Result<Self> {
        let default_db_path = "/data/";
        // Retrieve the database path from the environment variable, or use the default path if not set
        let db_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| default_db_path.to_string());
        let path = Path::new(&db_path).join("motorbot.db");
        if !path.exists() {
            info!(
                "Database file not found, creating new database at {}",
                &path.to_string_lossy()
            );
            match Connection::open(&path) {
                Ok(conn) => {
                    let mut db = Self {
                        connection: Some(conn),
                    };
                    db.create_tables().await?;
                    info!(
                        "Database created successfully at {}",
                        &path.to_string_lossy()
                    );
                    Ok(db)
                }
                Err(e) => {
                    error!("Failed to create database: {:?}", e);
                    Err(Error::Storage(StorageError::UnableToConnect {
                        err: e,
                    }))
                }
            }
        } else {
            let mut db = Self {
                connection: Some(Connection::open(path).map_err(|err| {
                    Error::Storage(StorageError::UnableToConnect { err })
                })?),
            };
            let version = db.schema_version().await?;
            // Check if schema version matches current version.
            // Future migrations should be handled here.
            if version == SCHEMA_VERSION {
                Ok(db)
            } else {
                Err(Error::Storage(StorageError::InvalidSchemaVersion {
                    expected: SCHEMA_VERSION,
                    found: version,
                }))
            }
        }
    }

    /// Fetches the user score for a given user ID
    ///
    /// ## Arguments
    /// * `user_id` - The ID of the user for whom to fetch the score.
    ///
    /// ## Returns
    /// * `Ok(UserScore)` - If the user score is found, it returns a
    ///   [UserScore] struct containing the user ID and score. If the
    ///   user is not found, it returns a UserScore with a score of 0.
    /// * `Err(Error)` - If there is an error during the
    ///   database query, it returns a [Error] variant with details
    ///   about the failure.
    pub async fn user_score(&mut self, user_id: &u64) -> Result<UserScore> {
        match &mut self.connection {
            Some(connection) => {
                const Q_USER_SCORE: &str =
                    "SELECT user_id, score FROM user_scores WHERE user_id = ?1";
                let mut stmt =
                    connection.prepare(Q_USER_SCORE).map_err(|e| {
                        Error::Storage(StorageError::with_sql(e, Q_USER_SCORE))
                    })?;
                let mut rows =
                    stmt.query(params![user_id.to_string()]).map_err(|e| {
                        Error::Storage(StorageError::with_sql(e, Q_USER_SCORE))
                    })?;

                if let Some(row) = rows.next()? {
                    let user_id: String = row.get(0)?;
                    Ok(UserScore {
                        user_id: user_id.parse::<u64>().unwrap_or(0),
                        score: row.get(1)?,
                    })
                } else {
                    debug!("Score not found for user {}, returning 0", user_id);
                    Ok(UserScore {
                        user_id: *user_id,
                        score: 0,
                    })
                }
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Sets the user score for a given user ID
    ///
    /// ## Arguments
    /// * `user_id` - The ID of the user for whom to set the score.
    /// * `score` - The score value to set for the user.
    ///
    /// ## Returns
    /// * `Ok(UserScore)` - If the user score is successfully set, it returns
    ///   a [UserScore] struct containing the user ID and the new score.
    /// * `Err(Error)` - If there is an error during the database operation, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn set_user_score(
        &mut self,
        user_id: &u64,
        score: i32,
    ) -> Result<UserScore> {
        match &mut self.connection {
            Some(connection) => {
                const Q_SET_USER_SCORE: &str =
                    "INSERT INTO user_scores (user_id, score) VALUES (?1, ?2)
          ON CONFLICT(user_id) DO UPDATE SET score = excluded.score";
                connection
                    .execute(
                        Q_SET_USER_SCORE,
                        params![user_id.to_string(), score],
                    )
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_SET_USER_SCORE,
                        ))
                    })?;
                Ok(UserScore {
                    user_id: *user_id,
                    score,
                })
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Gets the game news for a given game ID
    ///
    /// ## Arguments
    /// * `game_id` - The ID of the game for which to fetch news.
    ///
    /// ## Returns
    /// * `Ok(Vec<String>)` - If the game news is found, it returns a vector of
    ///   news item hashes.
    /// * `Err(Error)` - If there is an error during the database query, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn game_news(&mut self, game_id: &str) -> Result<Vec<String>> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GAME_NEWS: &str =
                    "SELECT game_id, news_id FROM game_news WHERE game_id = ?1";
                let mut stmt =
                    connection.prepare(Q_GAME_NEWS).map_err(|e| {
                        Error::Storage(StorageError::with_sql(e, Q_GAME_NEWS))
                    })?;
                let mut rows = stmt.query(params![game_id]).map_err(|e| {
                    Error::Storage(StorageError::with_sql(e, Q_GAME_NEWS))
                })?;

                let mut news_items = Vec::new();
                while let Some(row) = rows.next()? {
                    let news_id: String = row.get(1)?;
                    news_items.push(news_id);
                }

                Ok(news_items)
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Adds a new game to the database with the provided details
    ///
    /// ## Arguments
    /// * `game_id` - The unique ID for the game. This should be a steam game ID
    ///   or a riot games short code (e.g. "val" for Valorant).
    /// * `guild` - The guild ID associated with the game, used to determine which
    ///   Discord server it belongs to.
    /// * `platform` - The platform for the game.
    /// * `name` - The name of the game.
    /// * `thumbnail` - The URL of the game's thumbnail.
    /// * `color` - The color associated with the game.
    ///
    /// ## Returns
    /// * `Ok(())` - If the game is successfully added to the database, it returns
    ///   an empty Ok result.
    /// * `Err(Error)` - If there is an error during the database operation, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn add_game(
        &mut self,
        game_id: &str,
        guild: u64,
        platform: Platform,
        name: &str,
        thumbnail: &str,
        color: &str,
    ) -> Result<()> {
        match &mut self.connection {
            Some(connection) => {
                const Q_ADD_GAME: &str = "INSERT INTO games (id, guild, platform, name, thumbnail, color) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
                connection
                    .execute(
                        Q_ADD_GAME,
                        params![
                            game_id,
                            guild.to_string(),
                            platform.to_string(),
                            name,
                            thumbnail,
                            color
                        ],
                    )
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(e, Q_ADD_GAME))
                    })?;
                Ok(())
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Removes a game from the database based on the provided game ID and guild ID
    ///
    /// ## Arguments
    /// * `game_id` - The unique ID of the game to be removed.
    /// * `guild` - The guild ID associated with the game, used to determine which
    ///   Discord server it belongs to.
    ///
    /// ## Returns
    /// * `Ok(())` - If the game is successfully removed from the database, it returns
    ///   an empty Ok result.
    /// * `Err(Error)` - If there is an error during the database operation, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn remove_game(
        &mut self,
        game_id: &str,
        guild: u64,
    ) -> Result<()> {
        match &mut self.connection {
            Some(connection) => {
                const Q_REMOVE_GAME: &str =
                    "DELETE FROM games WHERE id = ?1 AND guild = ?2";
                connection
                    .execute(Q_REMOVE_GAME, params![game_id, guild.to_string()])
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(e, Q_REMOVE_GAME))
                    })?;
                Ok(())
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Gets the game details for a given game ID
    ///
    /// ## Arguments
    /// * `game_id` - The ID of the game for which to fetch details.
    ///
    /// ## Returns
    /// * `Ok(GameData)` - If the game details are found, it returns a [GameData]
    ///   struct.
    /// * `Err(Error)` - If there is an error during the database query, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn game_details(&mut self, game_id: &str) -> Result<GameData> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GAME_DETAILS: &str = "SELECT guild, platform, name, thumbnail, color FROM games WHERE id = ?1";
                let mut stmt =
                    connection.prepare(Q_GAME_DETAILS).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GAME_DETAILS,
                        ))
                    })?;
                let mut rows = stmt.query(params![game_id]).map_err(|e| {
                    Error::Storage(StorageError::with_sql(e, Q_GAME_DETAILS))
                })?;

                let mut platform: Platform = Platform::Unknown;
                let news_items = None;
                let mut guild_data = Vec::new();
                while let Some(row) = rows.next()? {
                    let guild_str: String = row.get(0)?;
                    let platform_str: String = row.get(1)?;
                    platform = Platform::from(platform_str.as_str());
                    let name: String = row.get(2)?;
                    let thumbnail: String = row.get(3)?;
                    let color: String = row.get(4)?;
                    guild_data.push(GuildGameData {
                        guild: guild_str.parse::<u64>().unwrap_or(0),
                        name,
                        thumbnail,
                        color,
                    });
                }
                Ok(GameData {
                    id: game_id.to_string(),
                    platform,
                    news_items,
                    guild_data,
                })
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Fetches all game ids from the database
    ///
    /// ## Returns
    /// * `Ok(Vec<String>)` - If the query is successful, it returns a
    ///   vector of game IDs that are currently being monitored.
    /// * `Err(Error)` - If there is an error during the database query, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn game_ids(&mut self) -> Result<Vec<String>> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GAME_IDS: &str = "SELECT id FROM games GROUP BY id";
                let mut stmt = connection.prepare(Q_GAME_IDS).map_err(|e| {
                    Error::Storage(StorageError::with_sql(e, Q_GAME_IDS))
                })?;
                let mut rows = stmt.query([]).map_err(|e| {
                    Error::Storage(StorageError::with_sql(e, Q_GAME_IDS))
                })?;

                let mut game_ids = Vec::new();
                while let Some(row) = rows.next()? {
                    let game_id: String = row.get(0)?;
                    game_ids.push(game_id);
                }
                Ok(game_ids)
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Fetches all game ids for a specific guild from the database
    ///
    /// ## Arguments
    /// * `guild` - The guild ID for which to fetch the game IDs.
    ///
    /// ## Returns
    /// * `Ok(Vec<String>)` - If the query is successful, it returns a vector of
    ///   game IDs that are currently being monitored for the specified guild.
    /// * `Err(Error)` - If there is an error during the database query, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn game_ids_for_guild(
        &mut self,
        guild: u64,
    ) -> Result<Vec<String>> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GAME_IDS_FOR_GUILD: &str =
                    "SELECT id FROM games WHERE guild = ?1";
                let mut stmt =
                    connection.prepare(Q_GAME_IDS_FOR_GUILD).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GAME_IDS_FOR_GUILD,
                        ))
                    })?;
                let mut rows =
                    stmt.query(params![guild.to_string()]).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GAME_IDS_FOR_GUILD,
                        ))
                    })?;

                let mut game_ids = Vec::new();
                while let Some(row) = rows.next()? {
                    let game_id: String = row.get(0)?;
                    game_ids.push(game_id);
                }
                Ok(game_ids)
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Adds a news item for a given game ID and automatically removes old news
    /// items if there are more than 5
    ///
    /// ## Arguments
    /// * `game_id` - The ID of the game for which to add the news
    /// * `news_id` - The Hash/ID of the news item to add
    ///
    /// ## Returns
    /// * `Ok(())` - If the news item is successfully added, it returns
    ///   an empty Ok result.
    /// * `Err(Error)` - If there is an error during the database operation, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn add_news_item(
        &mut self,
        game_id: &str,
        news_id: &str,
    ) -> Result<()> {
        match &mut self.connection {
            Some(connection) => {
                const Q_ADD_NEWS_ITEM: &str =
                    "INSERT INTO game_news (game_id, news_id) VALUES (?1, ?2)";
                connection
                    .execute(Q_ADD_NEWS_ITEM, params![game_id, news_id])
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_ADD_NEWS_ITEM,
                        ))
                    })?;

                // Remove old news items if there are more than 5
                const Q_COUNT_NEWS_ITEMS: &str =
                    "SELECT COUNT(*) FROM game_news WHERE game_id = ?1";
                let mut stmt =
                    connection.prepare(Q_COUNT_NEWS_ITEMS).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_COUNT_NEWS_ITEMS,
                        ))
                    })?;
                let count: i32 = stmt
                    .query_row(params![game_id], |row| row.get(0))
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_COUNT_NEWS_ITEMS,
                        ))
                    })?;

                if count > 5 {
                    const Q_DELETE_OLD_NEWS: &str = "DELETE FROM game_news WHERE game_id = ?1 AND uid IN (
                SELECT uid FROM game_news WHERE game_id = ?1 ORDER BY uid ASC LIMIT ?2
            )";
                    connection
                        .execute(Q_DELETE_OLD_NEWS, params![game_id, count - 5])
                        .map_err(|e| {
                            Error::Storage(StorageError::with_sql(
                                e,
                                Q_DELETE_OLD_NEWS,
                            ))
                        })?;
                }
                Ok(())
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Sets a guild configuration option in the database
    ///
    /// ## Arguments
    /// * `guild_id` - The ID of the guild for which to set the configuration
    /// * `guild_config` - The guild configuration option to set
    ///
    /// ## Returns
    /// * `Ok(())` - If the guild configuration is successfully set, it returns
    ///   an empty Ok result.
    /// * `Err(Error)` - If there is an error during the database operation, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn set_guild_config(
        &mut self,
        guild_id: u64,
        guild_config: GuildConfig,
    ) -> Result<()> {
        match &mut self.connection {
            Some(connection) => {
                const Q_SET_GUILD_CONFIG: &str = "INSERT INTO guild_config (guild_id, config_key, config_value) VALUES (?1, ?2, ?3)
          ON CONFLICT(guild_id, config_key) DO UPDATE SET config_value = excluded.config_value";
                let config_key: &str = guild_config.key().as_ref();
                let config_value: String = guild_config.value().clone().into();
                connection
                    .execute(
                        Q_SET_GUILD_CONFIG,
                        params![guild_id.to_string(), config_key, config_value],
                    )
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_SET_GUILD_CONFIG,
                        ))
                    })?;
                Ok(())
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Gets a guild configuration option from the database
    ///
    /// ## Arguments
    /// * `guild_id` - The ID of the guild for which to fetch the configuration
    /// * `guild_config` - The guild configuration option to fetch
    ///
    /// ## Returns
    /// * `Ok(Option<String>)` - If the guild configuration is found, it returns
    ///   the configuration value as a string wrapped in Some. If the configuration is
    ///   not found, it returns None.
    /// * `Err(Error)` - If there is an error during the database query, it
    ///   returns a [Error] variant with details about the failure.
    pub async fn get_guild_config(
        &mut self,
        guild_id: u64,
        guild_config_key: GuildConfigKey,
    ) -> Result<Option<GuildConfig>> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GET_GUILD_CONFIG: &str = "SELECT config_value FROM guild_config WHERE guild_id = ?1 AND config_key = ?2";
                let mut stmt =
                    connection.prepare(Q_GET_GUILD_CONFIG).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GET_GUILD_CONFIG,
                        ))
                    })?;
                let mut rows = stmt
                    .query(params![
                        guild_id.to_string(),
                        guild_config_key.as_ref()
                    ])
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GET_GUILD_CONFIG,
                        ))
                    })?;

                if let Some(row) = rows.next()? {
                    let config_value: String = row.get(0)?;
                    Ok(Some(GuildConfig::from((
                        guild_config_key,
                        config_value,
                    ))))
                } else {
                    debug!("Guild config not found for guild {}, key {}, returning None", guild_id, guild_config_key.as_ref());
                    Ok(None)
                }
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Closes the database connection
    ///
    /// This method should be called when the database is no longer needed to
    /// ensure that all resources are properly released. It will attempt to close
    /// the connection and return any errors that occur during the process.
    ///
    /// ## Returns
    /// * `Ok(())` - If the connection is successfully closed it returns an
    ///   empty Ok result.
    /// * `Err(Error)` - If there is an error during the closing of the connection
    ///   or if the connection is already closed or not initialized, it returns
    ///   [Error::InvalidConnection] error.
    pub async fn close(&mut self) -> Result<()> {
        if let Some(conn) = self.connection.take() {
            conn.close().map_err(|(_conn, err)| {
                Error::Storage(StorageError::UnableToConnect { err })
            })?;
            self.connection = None;
            Ok(())
        } else {
            warn!("Unable to close database: connection not initialized or already closed");
            Err(Error::Storage(StorageError::InvalidConnection))
        }
    }

    /// Creates the necessary tables in the database
    ///
    /// This method is called when a new database is
    /// created to set up the required schema.
    ///
    /// ## Returns
    /// * `Ok(())` - If the tables are created successfully, it returns an
    ///   empty Ok result.
    /// * `Err(Error)` - If there is an error during table creation, it returns
    ///   an appropriate [Error] variant with details about the failure.
    async fn create_tables(&mut self) -> Result<()> {
        match &mut self.connection {
            Some(connection) => {
                // Set the schema version
                connection
                    .pragma_update(None, "user_version", SCHEMA_VERSION)
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            "Failed to set schema version",
                        ))
                    })?;

                // Create user_scores table
                const Q_CREATE_USER_SCORES_TABLE: &str =
                    "CREATE TABLE IF NOT EXISTS user_scores (
            user_id TEXT PRIMARY KEY,
            score INTEGER NOT NULL
        )";
                connection.execute(Q_CREATE_USER_SCORES_TABLE, []).map_err(
                    |e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_CREATE_USER_SCORES_TABLE,
                        ))
                    },
                )?;

                // Create platforms table
                const Q_CREATE_PLATFORMS_TABLE: &str =
                    "CREATE TABLE IF NOT EXISTS platforms (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )";
                connection.execute(Q_CREATE_PLATFORMS_TABLE, []).map_err(
                    |e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_CREATE_PLATFORMS_TABLE,
                        ))
                    },
                )?;
                const Q_INSERT_PLATFORMS: &str =
                    "INSERT OR IGNORE INTO platforms (id, name) VALUES
            ('steam', 'Steam'),
            ('riot', 'Riot Games')";
                connection.execute(Q_INSERT_PLATFORMS, []).map_err(|e| {
                    Error::Storage(StorageError::with_sql(
                        e,
                        Q_INSERT_PLATFORMS,
                    ))
                })?;

                // Create games table
                const Q_CREATE_GAMES_TABLE: &str =
                    "CREATE TABLE IF NOT EXISTS games (
            id TEXT NOT NULL,
            guild TEXT NOT NULL,
            platform TEXT NOT NULL,
            name TEXT NOT NULL,
            thumbnail TEXT NOT NULL,
            color TEXT NOT NULL,
            PRIMARY KEY (id, guild),
            FOREIGN KEY (platform) REFERENCES platforms(id)
        )";
                connection.execute(Q_CREATE_GAMES_TABLE, []).map_err(|e| {
                    Error::Storage(StorageError::with_sql(
                        e,
                        Q_CREATE_GAMES_TABLE,
                    ))
                })?;

                // Create game_news table
                const Q_CREATE_GAME_NEWS_TABLE: &str =
                    "CREATE TABLE IF NOT EXISTS game_news (
            uid INTEGER PRIMARY KEY,
            game_id TEXT NOT NULL,
            news_id TEXT  NOT NULL
        )";
                connection.execute(Q_CREATE_GAME_NEWS_TABLE, []).map_err(
                    |e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_CREATE_GAME_NEWS_TABLE,
                        ))
                    },
                )?;

                // Create guild_config table
                const Q_CREATE_GUILD_CONFIG_TABLE: &str =
                    "CREATE TABLE IF NOT EXISTS guild_config (
            guild_id TEXT NOT NULL,
            config_key TEXT NOT NULL,
            config_value TEXT NOT NULL,
            PRIMARY KEY (guild_id, config_key)
        )";
                connection
                    .execute(Q_CREATE_GUILD_CONFIG_TABLE, [])
                    .map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_CREATE_GUILD_CONFIG_TABLE,
                        ))
                    })?;

                Ok(())
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }

    /// Retrieves the current schema version of the
    /// database.
    ///
    /// This method executes a PRAGMA query to fetch the user_version
    /// from the database.
    ///
    /// ## Returns
    /// * `Ok(u8)` - If the query is successful, it returns the schema version as an unsigned 8-bit integer.
    /// * `Err(Error)` - If there is an error it returns an appropriate [Error] variant.
    async fn schema_version(&mut self) -> Result<u8> {
        match &mut self.connection {
            Some(connection) => {
                const Q_GET_SCHEMA_VERSION: &str = "PRAGMA user_version";
                let mut stmt =
                    connection.prepare(Q_GET_SCHEMA_VERSION).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GET_SCHEMA_VERSION,
                        ))
                    })?;
                let version: u8 =
                    stmt.query_row([], |row| row.get(0)).map_err(|e| {
                        Error::Storage(StorageError::with_sql(
                            e,
                            Q_GET_SCHEMA_VERSION,
                        ))
                    })?;
                debug!("Database schema version: {}", version);
                Ok(version)
            }
            None => Err(Error::Storage(StorageError::InvalidConnection)),
        }
    }
}
