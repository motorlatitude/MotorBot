use super::platforms::{platform::Platform, riot::Riot, steam::Steam};

pub struct PatchNotes {
    pub title: String,
    pub content: String,
    pub url: String,
    pub image: String,
    pub gid: String,
    pub success: bool,
}

impl PatchNotes {
    /// Fetches patch notes for a game via it's platform and game id
    ///
    /// # Arguments
    /// - `platform` - The platform
    /// - `game_id` - The game id
    pub async fn fetch_for_platform(platform: Platform, game_id: &str) -> Self {
        match platform {
            Platform::Steam => Self::from_steam(game_id).await,
            Platform::Riot => Self::from_riot(game_id).await,
            Platform::Unknown => Self::default(),
        }
    }

    /// Fetches patch notes from Steam for a Steam game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The Steam game id
    async fn from_steam(game_id: &str) -> Self {
        let steam_platform = Steam::new();
        let notes = steam_platform.fetch(game_id).await;
        match notes {
            Some(n) => Self {
                title: n.title,
                content: n.content,
                url: n.url,
                image: n.image,
                gid: n.gid,
                success: true,
            },
            None => Self::default(),
        }
    }

    async fn from_riot(game_id: &str) -> Self {
        let riot_platform: Riot = Riot::new();
        let notes = riot_platform.fetch(game_id).await;
        match notes {
            Some(n) => Self {
                title: n.title,
                content: n.content,
                url: n.url,
                image: n.image,
                gid: n.gid,
                success: true,
            },
            None => Self::default(),
        }
    }
}

impl Default for PatchNotes {
    fn default() -> Self {
        Self {
            title: String::from(""),
            content: String::from(""),
            url: String::from(""),
            image: String::from(""),
            gid: String::from(""),
            success: false,
        }
    }
}
