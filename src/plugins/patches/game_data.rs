use tracing::error;

use crate::{plugins::patches::platforms::platform::Platform, storage::Database};

/// Represents the data associated with a game, including its unique ID,
/// platform, news items, name, thumbnail, and color. This struct is used to
/// store and manage game-related information, particularly for tracking news
/// items and displaying game details in the Patches Plugin.
#[derive(Debug)]
pub struct GameData {
    /// The unique ID for the game.
    pub id: String,
    /// The guild ID associated with the game, used to determine which Discord
    /// guild/server the game is being monitored for.
    pub guild: u64,
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
    pub news_items: Vec<String>,
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

impl GameData {
    /// Fetches game data from the database via its id
    ///
    /// # Arguments
    /// - `game_id` - The game id
    pub async fn from_id(game_id: &str) -> Self {
        let mut db = Database::open()
            .await
            .expect("Failed to connect to database");
        let data = db
            .game_news(&game_id)
            .await
            .expect("Failed to fetch game data");
        if let Err(why) = db.close().await {
            error!("Failed to close database connection {:?}", why);
        }
        data
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            id: String::from(""),
            guild: 0,
            platform: Platform::default(),
            news_items: Vec::new(),
            name: String::from(""),
            thumbnail: String::from(""),
            color: String::from(""),
        }
    }
}
