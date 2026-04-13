use crate::{
    MotorbotChannels, plugins::patches::game_data::{GameData, GuildGameData}, storage::{Database, GuildConfigKey, database::GuildConfigValue}
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
        self.update().await;
    }

    /// Looks for new patch notes for games
    pub async fn update(&self) {
        info!("Updating sources...");
        self.ctx.set_presence(
            Some(ActivityData::playing("🔍")),
            OnlineStatus::DoNotDisturb,
        );
        let mut db = Database::open()
            .await
            .expect("Failed to connect to database");
        let game_ids = db.game_ids().await;
        let games_to_monitor = match game_ids {
            Ok(ids) => ids,
            Err(e) => {
                error!("Failed to fetch game ids: {:?}", e);
                return;
            }
        };
        // Get latest patch notes for each game_id and check latest patch notes
        // against db if patch notes are different, post patch notes to channel
        for game_id in games_to_monitor {
            // Data from DB
            let game_data = GameData::from_id(&game_id).await;
            // Patch notes from Platform
            let patch_notes = PatchNotes::fetch_for_platform(game_data.platform, &game_id).await;
            // Compare gid
            let game_news_items = match &game_data.news_items {
                Some(items) => items,
                None => {
                    warn!("No news items found for game_id {}, skipping", game_id);
                    continue;
                }
            };
            for guild_data in &game_data.guild_data {
                if !game_news_items.contains(&patch_notes.gid) {
                    // Send patch notes
                    info!("[⬦] {} ({}) [{}]", &guild_data.name, game_id, guild_data.guild);
                    self.send_patch_notes(&mut db, &patch_notes, &game_id, guild_data).await;
                } else {
                    info!("[✔] {} ({}) [{}]", guild_data.name, game_id, guild_data.guild);
                }
            }
        }
        info!("Update complete");
        self.ctx
            .set_presence(Some(ActivityData::custom("😶‍🌫️")), OnlineStatus::Online);
    }

    /// Sends patch notes to a channel
    ///
    /// ## Arguments
    /// - `db` - A `DBClient` struct containing the database client
    /// - `platform_data` - A `PatchNotes` struct containing the patch notes
    /// - `game_id` - A string slice containing the game ID
    /// - `game_data` - A [GuildGameData] struct containing the game data
    async fn send_patch_notes(
        &self,
        db: &mut Database,
        platform_data: &PatchNotes,
        game_id: &str,
        game_data: &GuildGameData,
    ) {
        if platform_data.success == false {
            warn!("Patch notes failed to fetch for {}", game_id);
            return;
        }

        let channel_id = match db
            .get_guild_config(game_data.guild, GuildConfigKey::PatchNotesChannel)
            .await
        {
            Ok(Some(config)) => match config.value {
                GuildConfigValue::ChannelId(id) => ChannelId::new(id),
            },
            Ok(None) => {
                warn!(
                    "No patch notes channel configured for guild {}, skipping",
                    game_data.guild
                );
                return;
            }
            Err(e) => {
                error!(
                    "Failed to fetch patch notes channel for guild {}, error: {:?}",
                    game_data.guild, e
                );
                return;
            }
        };

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
                                CreateEmbedAuthor::new(&game_data.name)
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
                .add_news_item(&game_id, &platform_data.gid)
                .await
            {
                Ok(_) => (),
                Err(e) => error!("Failed to update game news id: {:?}", e),
            }
        }
    }
}
