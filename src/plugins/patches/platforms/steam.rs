use regex::Regex;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::{debug, error};

pub struct Steam {}

pub struct SteamNews {
    pub title: String,
    pub content: String,
    pub url: String,
    pub image: String,
    pub gid: String,
}

// Steam URLs
const STEAM_BASE_URL: &str = "https://api.steampowered.com";
const STEAM_NEWS_ENDPOINT: &str = "/ISteamNews/GetNewsForApp/v2/";
const STEAM_CLAN_IMAGE: &str = "https://clan.akamai.steamstatic.com/images/";

impl Steam {
    pub fn new() -> Self {
        Self {}
    }

    /// Fetches patch notes from Steam for a Steam game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The Steam game id
    ///
    /// # Returns
    /// A `SteamNews` struct containing the patch notes
    pub async fn fetch(&self, game_id: &str) -> Option<SteamNews> {
        let notes = self.request::<Value>(game_id).await;
        match notes {
            Ok(n) => self.parse_response(n),
            Err(e) => {
                error!("Error fetching Steam patch notes: {:?}", e);
                None
            }
        }
    }

    /// Parses the response from Steam into a `SteamNews` struct
    ///
    /// # Arguments
    /// - `response` - The response from Steam
    ///
    /// # Returns
    /// A `SteamNews` struct containing the patch notes
    fn parse_response(&self, response: Value) -> Option<SteamNews> {
        let gid = response["appnews"]["newsitems"][0]["gid"]
            .to_string()
            .replace("\"", "");
        let content = response["appnews"]["newsitems"][0]["contents"]
            .to_string()
            .replace("\"", "")
            .replace("\\\"", "\"")
            .replace("[b]", "**")
            .replace("[/b]", "**")
            .replace("[i]", "*")
            .replace("[/i]", "*")
            .replace("[u]", "__")
            .replace("[/u]", "__")
            .replace("[quote]", "> ")
            .replace("[/quote]", "")
            .replace("\\n", "\n")
            .replace("[p]", "")
            .replace("[p align=\\start]", "")
            .replace("[/p]", "\n")
            .replace("[list]", "")
            .replace("[*]", "- ")
            .replace("[/list]", "")
            .replace("[h1]", "## ")
            .replace("[/h1]", "")
            .replace("[h2]", "### ")
            .replace("[/h2]", "")
            .replace("[h3]", "")
            .replace("[/h3]", "");
        let re = Regex::new(r"\[img](.*?)\[/img]").unwrap();
        let mut images = vec![];
        for (_, [path]) in re.captures_iter(&content).map(|c| c.extract()) {
            images.push(path.replace("{STEAM_CLAN_IMAGE}", STEAM_CLAN_IMAGE));
        }
        let re2 = Regex::new(r"\[img src=(.*?)]\[/img]").unwrap();
        for (_, [path]) in re2.captures_iter(&content).map(|c| c.extract()) {
            images.push(path.replace("{STEAM_CLAN_IMAGE}", STEAM_CLAN_IMAGE).replace("\\", ""));
        }
        let re3 = Regex::new(r"\[url=(.*?)](.*?)\[/url]").unwrap();
        let re_youtube = Regex::new(r"\[previewyoutube=(.*?)]\[/previewyoutube]").unwrap();
        let parsed_content_1 = re.replace_all(&content, "");
        let parsed_content_2 = re2.replace_all(&parsed_content_1, "");
        let parsed_content_3 = re3.replace_all(&parsed_content_2, "");
        let parsed_content = re_youtube.replace_all(&parsed_content_3, "");
        let parsed_trimmed_content = parsed_content.trim();
        let patch_notes_url = response["appnews"]["newsitems"][0]["url"]
            .to_string()
            .replace("\"", "");
        let patch_notes_title = response["appnews"]["newsitems"][0]["title"]
            .to_string()
            .replace("\"", "");
        let mut image = "";
        if images.len() > 0 {
            image = &images[0];
        }
        let truncated_content = match parsed_trimmed_content.char_indices().nth(400) {
            None => parsed_trimmed_content,
            Some((idx, _)) => &parsed_trimmed_content[..idx],
        };

        Some(SteamNews {
            title: patch_notes_title,
            content: format!(
                "{}{}",
                &truncated_content,
                (parsed_trimmed_content.len() > 400)
                    .then(|| "...")
                    .unwrap_or("")
            ),
            url: patch_notes_url,
            image: image.to_string(),
            gid,
        })
    }

    /// Fetches patch notes from Steam for a Steam game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The Steam game id
    async fn request<T>(&self, game_id: &str) -> Result<T, StatusCode>
    where
        T: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}{}?appid={}&count=1&maxlength=0&format=json&feeds=steam_community_announcements",
                STEAM_BASE_URL, STEAM_NEWS_ENDPOINT, game_id
            ))
            .header("Content-type", "application/json")
            .send()
            .await;

        match &response {
            Ok(r) => {
                debug!("Steam Web API Response: {:?}", r.status());
                if r.status() != StatusCode::OK {
                    Err(r.status())
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
                    Err(e.status().unwrap())
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        }
    }
}
