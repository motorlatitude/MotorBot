use super::platforms::{platform::Platform, riot::Riot, steam::Steam};
use crate::plugins::patches::error::PatchesError;
use crate::{Error, Result};

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
    pub async fn fetch_for_platform(
        platform: Platform,
        game_id: &str,
    ) -> Result<Self> {
        match platform {
            Platform::Steam => Self::from_steam(game_id).await,
            Platform::Riot => Self::from_riot(game_id).await,
        }
    }

    /// Fetches patch notes from Steam for a Steam game via it's game id
    ///
    /// # Arguments
    /// - `game_id` - The Steam game id
    async fn from_steam(game_id: &str) -> Result<Self> {
        let steam_platform = Steam::new();
        let notes = steam_platform.fetch(game_id).await.ok_or_else(|| {
            Error::Custom(Box::new(PatchesError::FetchPatchNotesFailed {
                platform: Platform::Steam,
                game_id: game_id.to_string(),
            }))
        })?;
        Ok(Self {
            title: notes.title,
            content: notes.content,
            url: notes.url,
            image: notes.image,
            gid: notes.gid,
            success: true,
        })
    }

    async fn from_riot(game_id: &str) -> Result<Self> {
        let riot_platform: Riot = Riot::new();
        let notes = riot_platform.fetch(game_id).await.ok_or_else(|| {
            Error::Custom(Box::new(PatchesError::FetchPatchNotesFailed {
                platform: Platform::Riot,
                game_id: game_id.to_string(),
            }))
        })?;
        Ok(Self {
            title: notes.title,
            content: notes.content,
            url: notes.url,
            image: notes.image,
            gid: notes.gid,
            success: true,
        })
    }
}
