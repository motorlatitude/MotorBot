use std::{fmt, str::FromStr};

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::{debug, error, info};
use voca_rs::*;

/// A struct containing the Riot game id
/// There are 3 main Riot games, others can be added later assuming they have
/// similar website structures.
///
/// # Riot Game IDs
/// - `LoL` - League of Legends
/// - `TFT` - Teamfight Tactics
/// - `VAL` - Valorant
///
/// # Example
/// ```
/// use patches::platforms::riot::RiotGameId;
///
/// let game_id = RiotGameId::from_str("lol").unwrap();
/// assert_eq!(game_id, RiotGameId::LoL);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum RiotGameId {
    LoL,
    TFT,
    VAL,
    Unknown,
}

impl fmt::Display for RiotGameId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RiotGameId::LoL => write!(f, "lol"),
            RiotGameId::TFT => write!(f, "tft"),
            RiotGameId::VAL => write!(f, "val"),
            RiotGameId::Unknown => write!(f, "unknown"),
        }
    }
}

impl FromStr for RiotGameId {
    type Err = ();

    fn from_str(input: &str) -> Result<RiotGameId, Self::Err> {
        match input {
            "lol" => Ok(RiotGameId::LoL),
            "tft" => Ok(RiotGameId::TFT),
            "val" => Ok(RiotGameId::VAL),
            _ => Ok(RiotGameId::Unknown),
        }
    }
}

/// A struct containing the Riot News endpoints to access patch notes for
/// Riot games.
pub struct Riot {}

/// A struct containing the patch notes for a Riot game
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
/// League of Legends base page data url (*Note: TFT uses the same base url*)
const LOL_BASE_PAGE_DATA_URL: &str = "https://www.leagueoflegends.com/page-data/en-gb";
/// League of Legends base url (*Note: TFT uses the same base url*)
const LOL_BASE_URL: &str = "https://www.leagueoflegends.com/en-gb/";
/// League of Legends latest news endpoint (*Note: TFT uses the same endpoint*)
const LOL_LATEST_NEWS_ENDPOINT: &str = "/latest-news/page-data.json";
/// Valorant's base page data url
const VAL_BASE_PAGE_DATA_URL: &str = "https://www.playvalorant.com/page-data/en-gb";
/// Valorant's base url
const VAL_BASE_URL: &str = "https://www.playvalorant.com/en-gb";
/// Valorant's latest news endpoint
const VAL_LATEST_NEWS_ENDPOINT: &str = "/news/page-data.json";

impl Riot {
    /// Creates a new Riot struct
    pub fn new() -> Self {
        Self {}
    }

    /// Fetches patch notes from Riot for a Riot game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The riot game id as a string
    ///
    /// # Returns
    /// A `RiotNews` struct containing the patch notes
    pub async fn fetch(&self, game_id: &str) -> Option<RiotNews> {
        let article_list = self.request::<Value>(game_id, None).await;
        match article_list {
            Ok(n) => {
                let mut response = None;
                match RiotGameId::from_str(game_id).unwrap() {
                    RiotGameId::LoL | RiotGameId::TFT => {
                        let articles = n["result"]["data"]["allArticles"]["edges"]
                            .as_array()
                            .unwrap();
                        for article in articles {
                            let is_tft = self.check_if_tft(&article);

                            if game_id == "lol" && is_tft {
                                // skip TFT news for LoL
                                continue;
                            } else if (game_id == "lol" && !is_tft) || (game_id == "tft" && is_tft)
                            {
                                if article["node"]["external_link"].as_str().unwrap_or("") == ""
                                    && article["node"]["youtube_link"].as_str().unwrap_or("") == ""
                                {
                                    // if the article has no external link, it's a normal article
                                    // articles with no external links need extra processing and need to be fetched
                                    let article_path = format!(
                                        "{}page-data.json",
                                        article["node"]["url"]["url"].as_str().unwrap_or("")
                                    );
                                    // info!("Article path: {}", article_path);
                                    let article =
                                        self.request::<Value>(game_id, Some(&article_path)).await;
                                    match article {
                                        Ok(n) => response = self.parse_lol_response(n, game_id),
                                        Err(e) => {
                                            error!("Error fetching Riot patch notes: {:?}", e);
                                            response = None
                                        }
                                    }
                                    break;
                                } else {
                                    // Articles with external links can be parsed immediately as they don't have an
                                    // article body
                                    // info!(
                                    //     "External link: {}",
                                    //     article["node"]["external_link"].as_str().unwrap_or("")
                                    // );
                                    let patch_notes_title =
                                        article["node"]["title"].as_str().unwrap_or("");
                                    let parsed_content =
                                        article["node"]["description"].as_str().unwrap_or("");
                                    let stripped_parsed_content = strip::strip_tags(parsed_content);
                                    let trimmed_parsed_content = stripped_parsed_content.trim();
                                    let gid = article["node"]["uid"].as_str().unwrap_or("");
                                    let mut patch_notes_url =
                                        article["node"]["external_link"].as_str().unwrap_or("");
                                    if patch_notes_url == "" {
                                        patch_notes_url =
                                            article["node"]["youtube_link"].as_str().unwrap_or("");
                                    }
                                    let rn = RiotNews {
                                        title: String::from(patch_notes_title),
                                        content: format!(
                                            "{}{}",
                                            &trimmed_parsed_content[0..std::cmp::min(
                                                trimmed_parsed_content.len(),
                                                400
                                            )],
                                            (trimmed_parsed_content.len() > 400)
                                                .then(|| "...")
                                                .unwrap_or("")
                                        ),
                                        url: String::from(patch_notes_url),
                                        image: String::from(
                                            article["node"]["banner"]["url"].as_str().unwrap_or(""),
                                        ),
                                        gid: String::from(gid),
                                    };
                                    response = Some(rn);
                                    break;
                                }
                            }
                        }
                    }
                    RiotGameId::VAL => {
                        response = self.parse_val_response(n);
                    }
                    RiotGameId::Unknown => {}
                }
                info!("Riot patch notes: {:?}", response);
                response
            }
            Err(e) => {
                error!("Error fetching Riot patch notes: {:?}", e);
                None
            }
        }
    }

    /// Checks if the article is a TFT article
    /// Riot's API doesn't allow filtering by game id, so we have to manually
    /// check if the article is a TFT article by using the article tags associated
    /// with an article. This relies on the fact that all TFT articles should
    /// have a tag with the machine name `teamfight_tactics`.
    ///
    /// # Arguments
    /// - `n` - The article to check if it's a TFT article in the form of a
    /// `serde_json::Value`.
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

    /// Parses the valorant page data response into a `RiotNews` struct
    ///
    /// # Arguments
    /// - `response` - The response from Riot
    ///
    /// # Returns
    /// A `RiotNews` struct containing the patch notes
    fn parse_val_response(&self, response: Value) -> Option<RiotNews> {
        let patch_notes_title = response["result"]["data"]["allContentstackArticles"]["nodes"][0]
            ["title"]
            .as_str()
            .unwrap_or("");
        let parsed_content = response["result"]["data"]["allContentstackArticles"]["nodes"][0]
            ["description"]
            .as_str()
            .unwrap_or("");
        let trimmed_parsed_content = strip::strip_tags(
            &parsed_content
                .replace("\"", "")
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\t", "\t"),
        );
        let gid = response["result"]["data"]["allContentstackArticles"]["nodes"][0]["uid"]
            .as_str()
            .unwrap_or("");
        let mut patch_notes_url = String::from(
            response["result"]["data"]["allContentstackArticles"]["nodes"][0]["external_link"]
                .as_str()
                .unwrap_or(""),
        );
        if patch_notes_url == "" {
            patch_notes_url = format!(
                "{}{}",
                VAL_BASE_URL,
                response["result"]["data"]["allContentstackArticles"]["nodes"][0]["url"]["url"]
                    .as_str()
                    .unwrap_or("")
            );
        }
        let rn = RiotNews {
            title: String::from(patch_notes_title),
            content: format!(
                "{}{}",
                &trimmed_parsed_content[0..std::cmp::min(trimmed_parsed_content.len(), 400)],
                (trimmed_parsed_content.len() > 400)
                    .then(|| "...")
                    .unwrap_or("")
            ),
            url: patch_notes_url,
            image: String::from(
                response["result"]["data"]["allContentstackArticles"]["nodes"][0]["banner"]["url"]
                    .as_str()
                    .unwrap_or(""),
            ),
            gid: String::from(gid),
        };
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
            .as_str()
            .unwrap_or("");
        let mut parsed_content = String::from(
            response["result"]["data"]["all"]["nodes"][0]["patch_notes_body"][0]["patch_notes"]
                ["html"]
                .as_str()
                .unwrap_or(""),
        );
        if parsed_content.is_empty() || parsed_content == "null" {
            parsed_content = String::from("");
            let article_bodies = response["result"]["data"]["all"]["nodes"][0]["article_body"]
                .as_array()
                .unwrap();
            for article_body in article_bodies {
                //info!("Article body: {:?}", article_body);
                let c = article_body["rich_text_editor"]["rich_text_editor"]
                    .as_str()
                    .unwrap_or("");
                if c.is_empty() || c == "null" {
                    continue;
                }
                parsed_content = format!("{}\n{}", parsed_content, c);
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
            .as_str()
            .unwrap_or("");
        let base = match RiotGameId::from_str(game_id).unwrap() {
            RiotGameId::LoL => LOL_BASE_URL,
            RiotGameId::TFT => LOL_BASE_URL,
            RiotGameId::VAL => VAL_BASE_URL,
            RiotGameId::Unknown => "",
        };
        let patch_notes_url = format!(
            "{}{}",
            base,
            response["result"]["data"]["all"]["nodes"][0]["url"]["url"]
                .as_str()
                .unwrap_or("")
        );
        let rn = RiotNews {
            title: String::from(patch_notes_title),
            content: format!(
                "{}{}",
                &trimmed_parsed_content[0..std::cmp::min(trimmed_parsed_content.len(), 399)],
                (trimmed_parsed_content.len() > 399)
                    .then(|| "...")
                    .unwrap_or("")
            ),
            url: patch_notes_url,
            image: String::from(
                response["result"]["data"]["all"]["nodes"][0]["banner"]["url"]
                    .as_str()
                    .unwrap_or(""),
            ),
            gid: String::from(gid),
        };
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
        match RiotGameId::from_str(game_id).unwrap() {
            RiotGameId::LoL | RiotGameId::TFT => {
                url = format!(
                    "{}{}",
                    LOL_BASE_PAGE_DATA_URL,
                    article_path
                        .is_some()
                        .then(|| article_path.unwrap())
                        .unwrap_or(LOL_LATEST_NEWS_ENDPOINT)
                );
            }
            RiotGameId::VAL => {
                url = format!(
                    "{}{}",
                    VAL_BASE_PAGE_DATA_URL,
                    article_path
                        .is_some()
                        .then(|| article_path.unwrap())
                        .unwrap_or(VAL_LATEST_NEWS_ENDPOINT)
                );
            }
            RiotGameId::Unknown => {}
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
                            error!("{:?}", e);
                            Err(StatusCode::BAD_REQUEST)
                        }
                    }
                }
            }
            Err(e) => {
                error!("{} - {:?}", e.is_status(), e.status());
                if e.is_status() {
                    return Err(e.status().unwrap());
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        }
    }
}
