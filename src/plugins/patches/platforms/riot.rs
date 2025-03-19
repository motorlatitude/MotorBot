use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use reqwest::StatusCode;
use scraper::{Html, Selector};
use tracing::{debug, error, info};

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
/// League of Legends base page data url
const LOL_BASE_NEWS_URL: &str = "https://www.leagueoflegends.com/en-gb/news/";
/// League of Legends base url
const LOL_BASE_URL: &str = "https://www.leagueoflegends.com/en-gb/";
/// Teamfight Tactics base page data url
const TFT_BASE_NEWS_URL: &str = "https://teamfighttactics.leagueoflegends.com/en-gb/news/";
/// Teamfight Tactics base url
const TFT_BASE_URL: &str = "https://teamfighttactics.leagueoflegends.com/en-gb/";
/// Valorant's base page data url
const VAL_BASE_NEWS_URL: &str = "https://playvalorant.com/en-gb/news/";
/// Valorant's base url
const VAL_BASE_URL: &str = "https://www.playvalorant.com";

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
        let raw_html = self.request(game_id, None).await;
        if raw_html.is_err() {
            return None;
        }
        let article_list = self.parse_news(game_id, raw_html.unwrap());
        if article_list.is_empty() {
            return None;
        }
        let latest_article = article_list[0].clone();
        info!("Latest article: {:?}", latest_article.title);
        Some(latest_article)
    }

    /// Parses the response from League of Legends, Valorant, and Teamfight Tactics patch notes
    /// and returns a `RiotNews` struct
    pub fn parse_news(&self, game_id: &str, raw_html: String) -> Vec<RiotNews> {
        let document = Html::parse_document(raw_html.as_str());
        let mut articles = vec![];
        let selector = Selector::parse(r#"section[id="news"] a[role="button"]"#).unwrap();
        let next_data_selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#).unwrap();
        let content_description_selector =
            Selector::parse(r#"div[data-testid="card-description"]"#).unwrap();
        for article in document.select(&selector) {
            let mut url: String = article.value().attr("href").unwrap_or_default().to_string();
            if url.starts_with("/") {
                let base_url = match RiotGameId::from_str(game_id).unwrap() {
                    RiotGameId::LoL => LOL_BASE_URL,
                    RiotGameId::TFT => TFT_BASE_URL,
                    RiotGameId::VAL => VAL_BASE_URL,
                    RiotGameId::Unknown => "",
                };
                url = format!("{}{}", base_url, url);
            }
            let title = article.value().attr("aria-label").unwrap_or_default();
            let content_description = match article
                .select(&content_description_selector)
                .next() {
                Some(c) => c.text().collect::<String>(),
                None => "".to_string(),
            };
            // image is not available in the html and would require rendering
            // the page to get the image url, however a script JSON element can be
            // parsed in order to retreive the image url from JSON
            //let next_data = document.select(&next_data_selector).next().unwrap();
            let next_data_text = match document.select(&next_data_selector).next() {
                Some(d) => d.text().collect::<String>(),
                None => "{}".to_string(),
            };
            let next_data_json: serde_json::Value = serde_json::from_str(&next_data_text).unwrap();
            let image = next_data_json["props"]["pageProps"]["page"]["blades"][2]["items"][0]
                ["media"]["url"]
                .as_str()
                .unwrap_or_default();
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            let gid = hasher.finish().to_string();
            let news = RiotNews {
                title: title.to_string(),
                content: content_description.to_string(),
                url: url.to_string(),
                image: image.to_string(),
                gid,
            };
            //println!("{:?}", news);
            articles.push(news);
        }
        articles
    }

    /// Fetches patch notes from Riot
    ///
    /// # Arguments
    /// - `game_id` - The Riot game id
    async fn request(
        &self,
        game_id: &str,
        article_path: Option<&str>,
    ) -> Result<String, StatusCode> {
        let client = reqwest::Client::new();
        let mut url = format!("{}{}", LOL_BASE_NEWS_URL, "");
        match RiotGameId::from_str(game_id).unwrap() {
            RiotGameId::LoL => {
                url = format!(
                    "{}{}",
                    LOL_BASE_NEWS_URL,
                    article_path
                        .is_some()
                        .then(|| article_path.unwrap())
                        .unwrap_or("")
                );
            }
            RiotGameId::TFT => {
                url = format!(
                    "{}{}",
                    TFT_BASE_NEWS_URL,
                    article_path
                        .is_some()
                        .then(|| article_path.unwrap())
                        .unwrap_or("")
                );
            }
            RiotGameId::VAL => {
                url = format!(
                    "{}{}",
                    VAL_BASE_NEWS_URL,
                    article_path
                        .is_some()
                        .then(|| article_path.unwrap())
                        .unwrap_or("")
                );
            }
            RiotGameId::Unknown => {}
        }
        info!("Riot Web API URL: {}", url);
        let response = client.get(url).send().await;

        match &response {
            Ok(r) => {
                debug!("Riot Web API Response: {:?}", r.status());
                if r.status() != StatusCode::OK {
                    Err(r.status())
                } else {
                    let content = response.unwrap().text().await;
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
                    Err(e.status().unwrap_or_default())
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        }
    }
}
