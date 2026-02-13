use futures::TryStreamExt;
use mongodb::{
    Client, Collection, bson::doc, options::{ClientOptions, ResolverConfig}, results::UpdateResult
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserScore {
    pub user_id: String,
    pub score: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameNews {
    pub game_id: String,
    pub platform: String,
    pub news_id: String,
    pub game_name: String,
    pub thumbnail: String,
    pub color: String,
}

pub struct DBClient {
    client: Client,
}

impl DBClient {
    pub async fn connect() -> mongodb::error::Result<Self> {
        let mongo_url = env::var("MONGO_URL").expect("Expected MONGO_URL in environment");
        let client_options = ClientOptions::parse(&mongo_url).await?;

        match Client::with_options(client_options) {
            Ok(client) => Ok(Self { client }),
            Err(e) => {
                eprintln!("Failed to connect to MongoDB: {:?}", e);
                // try with cloudflare dns
                let client_options = ClientOptions::parse(&mongo_url)
                    .resolver_config(ResolverConfig::cloudflare())
                    .await?;
                match Client::with_options(client_options) {
                    Ok(client) => Ok(Self { client }),
                    Err(e) => {
                        eprintln!("Failed to connect to MongoDB with Cloudflare DNS: {:?}", e);
                        Err(e)
                    }
                }
            }
        }
    }

    pub async fn fetch_user_score(
        &self,
        user_id: &u64,
    ) -> mongodb::error::Result<Option<UserScore>> {
        // Query the user_score in the collection with a filter and an option.
        let filter = doc! { "user_id": user_id.to_string() };
        let cursor = self.user_score().find_one(filter).sort(doc! { "user_id": 1 }).await?;

        Ok(cursor)
    }

    pub async fn set_user_score(
        &self,
        user_id: &u64,
        score: i32,
    ) -> mongodb::error::Result<UpdateResult> {
        let update_doc = doc! { "$set": {"user_id": user_id.to_string(), "score": score }};
        let filter = doc! { "user_id": user_id.to_string() };
        let cursor = self
            .user_score()
            .update_one(filter, update_doc)
            .upsert(true)
            .await?;

        Ok(cursor)
    }

    fn user_score(&self) -> Collection<UserScore> {
        self.client
            .database("MotorBot")
            .collection::<UserScore>("user_score")
    }

    /// Fetches all game ids from the database
    ///
    /// The database stores an entry for each monitored game. This function
    /// returns a vector of all game ids that are currently being monitored.
    pub async fn fetch_game_ids(&self) -> Vec<GameNews> {
        let cursor = match self.game_news().find(doc! {}).sort(doc! { "game_id": 1 }).await {
            Ok(cursor) => cursor,
            Err(_) => return vec![],
        };
        cursor.try_collect().await.unwrap_or_else(|_| vec![])
    }

    /// Fetch details for a game id
    pub async fn fetch_game_news_id(
        &self,
        game_id: &str,
    ) -> mongodb::error::Result<Option<GameNews>> {
        // Query the user_score in the collection with a filter and an option.
        let filter = doc! { "game_id": game_id };
        let cursor = self.game_news().find_one(filter).sort(doc! { "game_id": 1 }).await?;

        Ok(cursor)
    }

    pub async fn set_game_news_id(
        &self,
        game_id: &str,
        news_id: &str,
    ) -> mongodb::error::Result<UpdateResult> {
        let update_doc = doc! { "$set": {"game_id": game_id, "news_id": news_id }};
        let filter = doc! { "game_id": game_id };
        let cursor = self
            .game_news()
            .update_one(filter, update_doc)
            .upsert(true)
            .await?;

        Ok(cursor)
    }

    fn game_news(&self) -> Collection<GameNews> {
        self.client
            .database("MotorBot")
            .collection::<GameNews>("game_news")
    }

    pub async fn shutdown(self) {
        self.client.shutdown().await;
    }
}
