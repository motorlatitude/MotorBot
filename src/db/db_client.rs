use std::env;
use mongodb::{bson::{doc}, options::{ClientOptions, FindOneOptions, UpdateOptions}, Client, Collection, results::UpdateResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserScore {
    pub user_id: String,
    pub score: i32,
}

pub struct DBClient {
  client: Client,
}

impl DBClient {

  pub async fn connect() -> mongodb::error::Result<Self> {
    let mongo_url = env::var("MONGO_URL").expect("Expected MONGO_URL in environment");
    let client_options = ClientOptions::parse(mongo_url).await?;
    Ok(Self {
        client: Client::with_options(client_options)?,
    })
  }

  pub async fn fetch_user_score(&self, user_id: &u64) -> mongodb::error::Result<Option<UserScore>> {
    // Query the user_score in the collection with a filter and an option.
    let filter = doc! { "user_id": user_id.to_string() };
    let find_options = FindOneOptions::builder().sort(doc! { "user_id": 1 }).build();
    let cursor = self.user_score().find_one(filter, find_options).await?;

    Ok(cursor)
  }

  pub async fn set_user_score(&self, user_id: &u64, score: i32) -> mongodb::error::Result<UpdateResult> {
    let update_doc = doc! { "$set": {"user_id": user_id.to_string(), "score": score }};
    let filter = doc! { "user_id": user_id.to_string() };
    let update_options = UpdateOptions::builder().upsert(true).build();
    let cursor = self.user_score().update_one(filter, update_doc, update_options).await?;

    Ok(cursor)
  }

  fn user_score(&self) -> Collection<UserScore> {
    self.client.database("MotorBot").collection::<UserScore>("user_score")
  }

}