use tracing::error;

use crate::{plugins::patches::platforms::platform::Platform, storage::Database};

/// Represents the guild specific data associated with a game, including game
/// name, thumbnail, and color.
#[derive(Debug)]
pub struct GuildGameData {
    /// The guild ID associated with the game, used to determine which Discord
    /// guild/server the game is being monitored for.
    pub guild: u64,
    /// The friendly name of the game.
    pub name: String,
    /// The URL for the game's logo image.
    pub thumbnail: String,
    /// The color associated with the news item.
    ///
    /// This is used for the embed color in Discord and should
    /// take the form of a hex color code (e.g., "FF0000" for red).
    pub color: String,
}

/// Represents the data associated with a game, including its unique ID,
/// platform, news items, name, thumbnail, and color. This struct is used to
/// store and manage game-related information, particularly for tracking news
/// items and displaying game details in the Patches Plugin.
#[derive(Debug)]
pub struct GameData {
    /// The unique ID for the game.
    pub id: String,
    /// The platform the game is available on.
    pub platform: Platform,
    /// A vector of news hashes/ids associated with the game,
    /// used to track which news items have already been
    /// processed.
    ///
    /// The items can be a hash or id that uniquely identifies a
    /// news item.
    ///
    /// ### Note
    /// This does not contain all news hashes/ids for a game,
    /// only the last 5 hashes.
    pub news_items: Option<Vec<String>>,
    /// A vector of guild specific data for the guild specified game data
    pub guild_data: Vec<GuildGameData>,
}

impl GameData {
    /// Fetches game data from the database via its id
    ///
    /// # Arguments
    /// - `game_id` - The game id
    /// - `guild` - The guild id
    pub async fn from_id(game_id: &str) -> Self {
        let mut db = Database::open()
            .await
            .expect("Failed to connect to database");
        let game_details = db.game_details(game_id).await;
        let news_items: Option<Vec<String>> = match db.game_news(game_id).await {
            Ok(items) => Some(items),
            Err(e) => {
                error!("Failed to fetch game news for game_id {}: {:?}", game_id, e);
                None
            }
        };
        if let Err(why) = db.close().await {
            error!("Failed to close database connection {:?}", why);
        }
        match game_details {
            Ok(details) => {
                GameData {
                    id: details.id,
                    platform: details.platform,
                    news_items,
                    guild_data: details.guild_data,
                }
            },
            Err(e) => {
                error!("Failed to fetch game details for game_id {}: {:?}", game_id, e);
                GameData::default()
            }
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            id: String::from(""),
            platform: Platform::default(),
            news_items: None,
            guild_data: Vec::new(),
        }
    }
}
