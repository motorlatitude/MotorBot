use crate::db::DBClient;

pub struct GameData {
    pub game_id: String,
    pub platform: String,
    pub news_id: String,
    pub game_name: String,
    pub thumbnail: String,
    pub color: String,
}

impl GameData {
    /// Fetches game data from the database via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The game id
    pub async fn from_id(game_id: &str) -> Self {
        let db = DBClient::connect()
            .await
            .expect("Failed to connect to database");
        let raw = db
            .fetch_game_news_id(&game_id)
            .await
            .expect("Failed to fetch game gid");
        if !raw.is_none() {
            let game_data = raw.unwrap();
            Self {
                game_id: game_data.game_id,
                platform: game_data.platform,
                news_id: game_data.news_id,
                game_name: game_data.game_name,
                thumbnail: game_data.thumbnail,
                color: game_data.color,
            }
        } else {
            Self::default()
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            game_id: String::from(""),
            platform: String::from(""),
            news_id: String::from(""),
            game_name: String::from(""),
            thumbnail: String::from(""),
            color: String::from(""),
        }
    }
}
