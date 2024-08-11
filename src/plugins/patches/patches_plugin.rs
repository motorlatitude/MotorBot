use std::str::FromStr;

use crate::{
    db::DBClient,
    plugins::patches::{game_data::GameData, platforms::platform::Platform},
    MotorbotChannels,
};
use serenity::{
    all::{
        ActivityData, Colour, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor,
        CreateEmbedFooter, CreateMessage, Timestamp,
    },
    model::{prelude::ChannelId, user::OnlineStatus},
    prelude::*,
};
use tracing::{error, info, warn};

use super::patch_notes::PatchNotes;

#[derive(Clone)]
pub struct PatchesPlugin {
    ctx: Context,
}

impl PatchesPlugin {
    pub async fn new(ctx: Context) -> Self {
        info!("Starting Patches Plugin");
        let channel_id = ChannelId::new(MotorbotChannels::BotEvents as u64);
        if let Err(why) = channel_id.say(&ctx.http, "Launching Patches Plugin").await {
            error!("Error sending message: {:?}", why);
        }
        let new_plugin = Self { ctx };
        new_plugin.start().await;
        new_plugin
    }

    /// Start the plugin
    ///
    /// # Example
    /// ```
    /// let patches_plugin = PatchesPlugin::new(); // <- Create a new instance of the plugin will automatically start it
    /// ```
    pub async fn start(&self) {
        info!("Starting Patches Plugin");
        self.update().await;
    }

    /// Looks for new patch notes for games
    pub async fn update(&self) {
        info!("Updating sources...");
        self.ctx.set_presence(
            Some(ActivityData::playing("Patches 🔃")),
            OnlineStatus::DoNotDisturb,
        );
        // let channel_id = ChannelId(432351112616738837);
        // if let Err(why) = channel_id
        //     .say(&self.ctx.http, "Updating Patch Sources...")
        //     .await
        // {
        //     error!("Error sending message: {:?}", why);
        // }
        let db = DBClient::connect()
            .await
            .expect("Failed to connect to database");
        let game_ids = db.fetch_game_ids().await;
        let games_to_monitor = game_ids
            .iter()
            .map(|g| g.game_id.to_string())
            .collect::<Vec<_>>();
        // Get latest patch notes for each gameid and check latest patch notes against db
        // if patch notes are different, post patch notes to channel
        for game_id in games_to_monitor {
            // Data from DB
            let game_data = GameData::from_id(&game_id).await;
            // Patch notes from Platform
            let patch_notes = PatchNotes::fetch_for_platform(
                Platform::from_str(&game_data.platform).unwrap_or(Platform::Unknown),
                &game_id,
            )
            .await;

            // Compare gid
            if game_data.news_id != patch_notes.gid {
                // Send patch notes
                self.send_patch_notes(&db, patch_notes, game_data).await;
            } else {
                info!("No new patch notes for {}", game_id);
            }
        }

        self.ctx
            .set_presence(Some(ActivityData::watching("you 👀")), OnlineStatus::Online);
    }

    /// Sends patch notes to a channel
    ///
    /// # Arguments
    /// - `db` - A `DBClient` struct containing the database client
    /// - `platform_data` - A `PatchNotes` struct containing the patch notes
    /// - `game_data` - A `GameData` struct containing the game data
    async fn send_patch_notes(
        &self,
        db: &DBClient,
        platform_data: PatchNotes,
        game_data: GameData,
    ) {
        if platform_data.success == false {
            warn!("Patch notes failed to fetch for {}", game_data.game_id);
            return;
        }
        let channel_id = ChannelId::new(MotorbotChannels::PatchNotes as u64);
        let mut action_row = vec![CreateActionRow::Buttons(vec![CreateButton::new_link(
            &platform_data.url,
        )
        .label("Patch Notes")])];
        // Clear the action row if the url is empty
        if platform_data.url.is_empty() {
            action_row.clear();
        }
        if let Err(why) = channel_id
            .send_message(
                &self.ctx.http,
                CreateMessage::new()
                    .content("")
                    .embed(
                        CreateEmbed::new()
                            .title(&platform_data.title)
                            .description(&platform_data.content)
                            //.thumbnail(&game_data.thumbnail)
                            .color(Colour::new(
                                u32::from_str_radix(&game_data.color, 16).unwrap_or(0),
                            ))
                            .image(&platform_data.image)
                            .url(&platform_data.url)
                            .author(
                                CreateEmbedAuthor::new(&game_data.game_name)
                                    .icon_url(&game_data.thumbnail),
                            )
                            .timestamp(Timestamp::now())
                            .footer(CreateEmbedFooter::new("MotorBot - Patch Plugin")),
                    )
                    .components(action_row),
            )
            .await
        {
            error!("Error sending message: {:?}", why);
        } else {
            match db
                .set_game_news_id(&game_data.game_id, &platform_data.gid)
                .await
            {
                Ok(_) => (),
                Err(e) => error!("Failed to update game news id: {:?}", e),
            }
        }
    }
}
