use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::{debug, error};
use voca_rs::*;

pub struct Riot {}

#[derive(Debug, Clone)]
pub struct RiotNews {
    pub title: String,
    pub content: String,
    pub url: String,
    pub image: String,
    pub gid: String,
}

// Riot URLs
// League of legends and TFT use the same base URL
const LOL_BASE_PAGE_DATA_URL: &str = "https://www.leagueoflegends.com/page-data/en-gb";
const LOL_BASE_URL: &str = "https://www.leagueoflegends.com/en-gb/";
const LOL_LATEST_NEWS_ENDPOINT: &str = "/latest-news/page-data.json";
// Valorant
const VAL_BASE_PAGE_DATA_URL: &str = "https://www.playvalorant.com/page-data/en-gb";
const VAL_BASE_URL: &str = "https://www.playvalorant.com/en-gb/";
const VAL_LATEST_NEWS_ENDPOINT: &str = "/news/page-data.json";

impl Riot {
    pub fn new() -> Self {
        Self {}
    }

    /// Fetches patch notes from Riot for a Riot game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The riot game id
    ///
    /// # Returns
    /// A `RiotNews` struct containing the patch notes
    pub async fn fetch(&self, game_id: &str) -> Option<RiotNews> {
        let article_list = self.request::<Value>(game_id, None).await;
        match article_list {
            Ok(n) => {
                let mut response = None;
                if game_id == "lol" || game_id == "tft" {
                    let articles = n["result"]["data"]["allArticles"]["edges"]
                        .as_array()
                        .unwrap();
                    for article in articles {
                        let is_tft = self.check_if_tft(&article);

                        if game_id == "lol" && is_tft {
                            // skip TFT news
                            continue;
                        } else {
                            let article_path = format!(
                                "{}page-data.json",
                                article["node"]["url"]["url"].to_string().replace("\"", "")
                            );
                            let article = self.request::<Value>(game_id, Some(&article_path)).await;
                            match article {
                                Ok(n) => response = self.parse_lol_response(n, game_id),
                                Err(e) => {
                                    error!("Error fetching Riot patch notes: {:?}", e);
                                    response = None
                                }
                            }
                            break;
                        }
                    }
                } else if game_id == "val" {
                    response = self.parse_val_response(n);
                }
                response
            }
            Err(e) => {
                error!("Error fetching Riot patch notes: {:?}", e);
                None
            }
        }
    }

    /// Checks if the article is a TFT article
    fn check_if_tft(&self, n: &Value) -> bool {
        let tags = n["node"]["article_tags"].as_array().unwrap();
        let mut is_tft = false;
        if tags.len() > 0 {
            for tag in tags {
                if tag["machine_name"] == "teamfight_tactics" {
                    is_tft = true;
                }
            }
        }
        is_tft
    }

    /// Parses the response from Riot into a `RiotNews` struct
    ///
    /// # Arguments
    /// - `response` - The response from Riot
    /// - `game_id` - The Riot game id
    ///
    /// # Returns
    /// A `SteamNews` struct containing the patch notes
    fn parse_val_response(&self, response: Value) -> Option<RiotNews> {
        let patch_notes_title = response["result"]["data"]["allContentstackArticles"]["nodes"][0]
            ["title"]
            .to_string()
            .replace("\"", "");
        let parsed_content = response["result"]["data"]["allContentstackArticles"]["nodes"][0]
            ["description"]
            .to_string();
        let trimmed_parsed_content = strip::strip_tags(
            &parsed_content
                .replace("\"", "")
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\t", "\t"),
        );
        let gid = response["result"]["data"]["allContentstackArticles"]["nodes"][0]["uid"]
            .to_string()
            .replace("\"", "");
        let patch_notes_url = response["result"]["data"]["allContentstackArticles"]["nodes"][0]
            ["external_link"]
            .to_string()
            .replace("\"", "");
        let rn = RiotNews {
            title: patch_notes_title,
            content: format!(
                "{}{}",
                &trimmed_parsed_content[0..std::cmp::min(trimmed_parsed_content.len(), 400)],
                (trimmed_parsed_content.len() > 400)
                    .then(|| "...")
                    .unwrap_or("")
            ),
            url: patch_notes_url,
            image: response["result"]["data"]["allContentstackArticles"]["nodes"][0]["banner"]
                ["url"]
                .to_string()
                .replace("\"", ""),
            gid,
        };
        //info!("Riot Val patch notes: {:?}", rn);
        Some(rn)
    }

    /// Parses the response from Riot into a `RiotNews` struct
    ///
    /// # Arguments
    /// - `response` - The response from Riot
    /// - `game_id` - The Riot game id
    ///
    /// # Returns
    /// A `SteamNews` struct containing the patch notes
    fn parse_lol_response(&self, response: Value, game_id: &str) -> Option<RiotNews> {
        let patch_notes_title = response["result"]["data"]["all"]["nodes"][0]["title"]
            .to_string()
            .replace("\"", "");
        let mut parsed_content = response["result"]["data"]["all"]["nodes"][0]["patch_notes_body"]
            [0]["patch_notes"]["html"]
            .to_string();
        if parsed_content.is_empty() || parsed_content == "null" {
            parsed_content = String::from("");
            let article_bodies = response["result"]["data"]["all"]["nodes"][0]["article_body"]
                .as_array()
                .unwrap();
            for article_body in article_bodies {
                //info!("Article body: {:?}", article_body);
                let c = article_body["rich_text_editor"]["rich_text_editor"].to_string();
                if c.is_empty() || c == "null" {
                    continue;
                }
                parsed_content = format!("{}\n{}", parsed_content, c.replace("\"", ""));
            }
        }
        let trimmed_parsed_content = strip::strip_tags(
            &parsed_content
                .replace("\"", "")
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\t", "\t"),
        );
        let gid = response["result"]["data"]["all"]["nodes"][0]["uid"]
            .to_string()
            .replace("\"", "");
        let patch_notes_url = format!(
            "{}{}",
            (game_id == "lol" || game_id == "tft")
                .then(|| LOL_BASE_URL)
                .unwrap_or(VAL_BASE_URL),
            response["result"]["data"]["all"]["nodes"][0]["url"]["url"]
                .to_string()
                .replace("\"", "")
        );
        let rn = RiotNews {
            title: patch_notes_title,
            content: format!(
                "{}{}",
                &trimmed_parsed_content[0..std::cmp::min(trimmed_parsed_content.len(), 400)],
                (trimmed_parsed_content.len() > 400)
                    .then(|| "...")
                    .unwrap_or("")
            ),
            url: patch_notes_url,
            image: response["result"]["data"]["all"]["nodes"][0]["banner"]["url"]
                .to_string()
                .replace("\"", ""),
            gid,
        };
        //info!("Riot patch notes: {:?}", rn);
        Some(rn)
    }

    /// Fetches patch notes from Riot
    ///
    /// # Arguments
    /// - `game_id` - The Riot game id
    async fn request<T>(&self, game_id: &str, article_path: Option<&str>) -> Result<T, StatusCode>
    where
        T: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let mut url = format!("{}{}", LOL_BASE_PAGE_DATA_URL, LOL_LATEST_NEWS_ENDPOINT);
        if game_id == "lol" {
            url = format!(
                "{}{}",
                LOL_BASE_PAGE_DATA_URL,
                article_path
                    .is_some()
                    .then(|| article_path.unwrap())
                    .unwrap_or(LOL_LATEST_NEWS_ENDPOINT)
            );
        } else if game_id == "tft" {
            url = format!(
                "{}{}",
                LOL_BASE_PAGE_DATA_URL,
                article_path
                    .is_some()
                    .then(|| article_path.unwrap())
                    .unwrap_or(LOL_LATEST_NEWS_ENDPOINT)
            );
        } else if game_id == "val" {
            url = format!(
                "{}{}",
                VAL_BASE_PAGE_DATA_URL,
                article_path
                    .is_some()
                    .then(|| article_path.unwrap())
                    .unwrap_or(VAL_LATEST_NEWS_ENDPOINT)
            );
        }
        //info!("Riot Web API URL: {}", url);
        let response = client
            .get(url)
            .header("Content-type", "application/json")
            .send()
            .await;

        match &response {
            Ok(r) => {
                debug!("Riot Web API Response: {:?}", r.status());
                if r.status() != StatusCode::OK {
                    return Err(r.status());
                } else {
                    let content = response.unwrap().json::<T>().await;
                    match content {
                        Ok(s) => Ok(s),
                        Err(e) => {
                            println!("{:?}", e);
                            Err(StatusCode::BAD_REQUEST)
                        }
                    }
                }
            }
            Err(e) => {
                println!("{} - {:?}", e.is_status(), e.status());
                if e.is_status() {
                    return Err(e.status().unwrap());
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        }
    }
}
