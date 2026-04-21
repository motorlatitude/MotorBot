use std::time::Duration;

use crate::{
    plugin::{option, MotorbotPlugin, PluginContext, PluginError, PluginInfo},
    plugins::patches::{
        game_data::{GameData, GuildGameData},
        patch_notes::PatchNotes,
        platforms::platform::Platform,
    },
    storage::{Database, GuildConfig, GuildConfigKey, GuildConfigValue},
    Error, Result,
};
use async_trait::async_trait;
use clokwerk::{AsyncScheduler, TimeUnits};
use serenity::all::{
    ActivityData, ChannelId, Colour, CommandOptionType, Context,
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    GuildId, Interaction, OnlineStatus, ResolvedValue, Timestamp,
};
use tracing::{error, info, warn};

pub mod game_data;
pub mod patch_notes;
pub mod platforms;

#[derive(Clone)]
pub struct PatchesPlugin {
    ctx: Option<Context>,
}

impl PatchesPlugin {
    /// Sets the context for the plugin, allowing it to access the Serenity
    /// context. This will be called on ready, and the context will be passed to
    /// the plugin for use in its operations.
    pub fn set_context(&mut self, ctx: Context) {
        self.ctx = Some(ctx);
    }

    /// Looks for new patch notes for games
    pub async fn update(&self) {
        match &self.ctx {
            Some(ctx) => {
                info!("Updating sources...");
                ctx.set_presence(
                    Some(ActivityData::custom("🧭 Exploring...")),
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
                    let patch_notes = PatchNotes::fetch_for_platform(
                        game_data.platform,
                        &game_id,
                    )
                    .await;
                    // Compare gid
                    let game_news_items = match &game_data.news_items {
                        Some(items) => items,
                        None => {
                            warn!(
                                "No news items found for game_id {}, skipping",
                                game_id
                            );
                            continue;
                        }
                    };
                    for guild_data in &game_data.guild_data {
                        if !game_news_items.contains(&patch_notes.gid) {
                            // Send patch notes
                            info!(
                                "[+] {} ({}) [{}]",
                                &guild_data.name, game_id, guild_data.guild
                            );
                            self.send_patch_notes(
                                &mut db,
                                &patch_notes,
                                &game_id,
                                guild_data,
                            )
                            .await;
                        } else {
                            info!(
                                "[•] {} ({}) [{}]",
                                guild_data.name, game_id, guild_data.guild
                            );
                        }
                    }
                }
                info!("Update complete");
                ctx.set_presence(
                    Some(ActivityData::custom("😶‍🌫️")),
                    OnlineStatus::Online,
                );
            }
            None => {
                warn!(
                    "No context set for PatchesPlugin, cannot update sources"
                );
            }
        }
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
        match &self.ctx {
            Some(ctx) => {
                if !platform_data.success {
                    warn!("Patch notes failed to fetch for {}", game_id);
                    return;
                }

                let channel_id: Option<ChannelId> = match db
                    .get_guild_config(
                        game_data.guild,
                        GuildConfigKey::PatchNotesChannel,
                    )
                    .await
                {
                    Ok(Some(config)) => match config.value {
                        GuildConfigValue::ChannelId(id) => {
                            Some(ChannelId::new(id))
                        }
                        _ => None,
                    },
                    Ok(None) => None,
                    Err(e) => {
                        error!(
                          "Failed to fetch patch notes channel for guild {}, error: {:?}",
                          game_data.guild, e
                        );
                        None
                    }
                };

                let mut action_row = vec![CreateActionRow::Buttons(vec![
                    CreateButton::new_link(&platform_data.url)
                        .label("Patch Notes"),
                ])];
                // Clear the action row if the url is empty
                if platform_data.url.is_empty() {
                    action_row.clear();
                }
                if let Some(channel_id) = channel_id {
                    if let Err(why) = channel_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::new()
                                .content("")
                                .embed(
                                    CreateEmbed::new()
                                        .title(&platform_data.title)
                                        .description(&platform_data.content)
                                        //.thumbnail(&game_data.thumbnail)
                                        .color(Colour::new(
                                            u32::from_str_radix(
                                                &game_data.color,
                                                16,
                                            )
                                            .unwrap_or(0),
                                        ))
                                        .image(&platform_data.image)
                                        .url(&platform_data.url)
                                        .author(
                                            CreateEmbedAuthor::new(
                                                &game_data.name,
                                            )
                                            .icon_url(&game_data.thumbnail),
                                        )
                                        .timestamp(Timestamp::now())
                                        .footer(CreateEmbedFooter::new(
                                            "MotorBot - Patch Plugin",
                                        )),
                                )
                                .components(action_row),
                        )
                        .await
                    {
                        error!("Error sending message: {:?}", why);
                    } else {
                        match db
                            .add_news_item(game_id, &platform_data.gid)
                            .await
                        {
                            Ok(_) => (),
                            Err(e) => {
                                error!("Failed to update game news id: {:?}", e)
                            }
                        }
                    }
                } else {
                    warn!(
                        "No channel configured for guild {}, cannot send patch notes",
                        game_data.guild
                    );
                }
            }
            None => {
                warn!(
                    "No context set for PatchesPlugin, cannot send patch notes"
                );
            }
        }
    }
}

/// A plugin that provides information about the latest patches for various
/// games.
#[async_trait]
impl MotorbotPlugin for PatchesPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        PatchesPlugin { ctx: None }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Patches".to_string(),
            description: "Provides information about the latest patches for various games".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Patches Plugin is ready!");

        for guild in p_ctx.ctx.cache.guilds() {
            GuildId::create_command(
                guild,
                &p_ctx.ctx.http,
                CreateCommand::new("patches")
                  .description("Get the latest patch notes for monitored games")
                  .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "list", "List all monitored games"))
                  .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "channel", "Set the channel for patch notes to be posted in")
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::Channel, "channel", "The channel to post patch notes in").required(true))
                  )
                  .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "update", "Manually trigger an update to check for new patch notes"))
                  .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "add", "Add a game to monitor")
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "The game id to monitor (this should be either a Steam or Riot game ID)").required(true))
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "platform", "The platform the game is on (e.g. steam, riot)").required(true))
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "The friendly name of the game").required(true))
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "thumbnail", "The URL for the game's logo image").required(true))
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "color", "The color associated with the news item (hex color code)").required(true))
                  )
                  .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "remove", "Remove a game from monitoring")
                      .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "The game id to stop monitoring").required(true))
                  )
            ).await?;
        }

        // Start the update loop
        let mut scheduler = AsyncScheduler::with_tz(chrono::Local);
        let mut plugin = self.clone();
        plugin.set_context(p_ctx.ctx.clone());
        scheduler.every(30.minutes()).run(move || {
            let plugin = plugin.clone();
            async move {
                plugin.update().await;
            }
        });

        tokio::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        Ok(())
    }

    async fn on_interaction_create(
        &self,
        p_ctx: &PluginContext,
        interaction: &Interaction,
    ) -> Result<()> {
        let Interaction::Command(command) = interaction else {
            // Not a command interaction, so we can skip processing
            return Ok(());
        };
        if command.data.name != "patches" {
            // Not the "patches" command, so we can skip processing
            return Ok(());
        }
        let mut db = Database::open().await?;

        let guild_id = command.guild_id.ok_or(PluginError::ExpectedGuild)?;

        let options = command.data.options();
        let first_option =
            options.first().ok_or(PluginError::InvalidSubCommand)?;

        let response = match first_option.name {
            "channel" => {
                if let ResolvedValue::SubCommand(subcommand_options) =
                    first_option.value.clone()
                {
                    info!("Subcommand options: {:?}", subcommand_options);
                    let channel = option!(
                        subcommand_options,
                        "channel",
                        ResolvedValue::Channel
                    )
                    .ok_or(PluginError::MissingChannelId)?;

                    let config_option = GuildConfig::from((
                        GuildConfigKey::PatchNotesChannel,
                        GuildConfigValue::ChannelId(channel.id.get()),
                    ));
                    db.set_guild_config(guild_id.get(), config_option).await?;
                    format!(
                        "Patch notes channel set to <#{}>",
                        channel.id.get()
                    )
                } else {
                    "No subcommand options provided".to_string()
                }
            }
            "list" => {
                let game_ids = db
                    .game_ids_for_guild(guild_id.get())
                    .await?;
                let mut response = String::new();
                let mut count = 1;
                for game_id in game_ids {
                    let game_data = GameData::from_id(&game_id).await;
                    let guild_data = game_data
                        .guild_data
                        .iter()
                        .find(|g| g.guild == guild_id.get());
                    let game_name = match guild_data {
                        Some(guild_data) => &guild_data.name,
                        None => "Unknown Name",
                    };
                    response.push_str(&format!(
                        "{}. {} ({})\n",
                        count, game_name, game_data.id
                    ));
                    count += 1;
                }
                if response.is_empty() {
                    "No games are currently being monitored".to_string()
                } else {
                    response
                }
            }
            "update" => {
                let mut plugin = self.clone();
                plugin.set_context(p_ctx.ctx.clone());
                let _ = plugin.update().await;
                "Update complete".to_string()
            }
            "add" => {
                if let ResolvedValue::SubCommand(subcommand_options) =
                    first_option.value.clone()
                {
                    info!("Subcommand options: {:?}", subcommand_options);
                    let id = option!(
                        subcommand_options,
                        "id",
                        ResolvedValue::String
                    )
                    .ok_or(PluginError::InvalidGameId)?;

                    let platform = option!(
                        subcommand_options,
                        "platform",
                        ResolvedValue::String
                    )
                    .ok_or(PluginError::InvalidGamePlatform)?;
                    if !Platform::is_valid_platform(platform) {
                        return Err(Error::Plugin(
                            PluginError::InvalidGamePlatform,
                        ));
                    } else {
                        let name = option!(
                            subcommand_options,
                            "name",
                            ResolvedValue::String
                        ).ok_or(PluginError::InvalidGameName)?;
                        let thumbnail = option!(
                            subcommand_options,
                            "thumbnail",
                            ResolvedValue::String
                        ).ok_or(PluginError::InvalidGameThumbnail)?;

                        if !thumbnail.starts_with("http") {
                            return Err(Error::Plugin(
                                PluginError::InvalidGameThumbnail,
                            ));
                        } else {
                            let raw_color = option!(
                                subcommand_options,
                                "color",
                                ResolvedValue::String
                            ).ok_or(PluginError::InvalidGameColor)?;

                            if !raw_color.starts_with("#")
                                || raw_color.len() != 7
                            {
                                return Err(Error::Plugin(
                                    PluginError::InvalidGameColor,
                                ));
                            } else {
                                let color = raw_color[1..].to_uppercase();
                                db.add_game(
                                        id,
                                        guild_id.get(),
                                        Platform::from(platform),
                                        name,
                                        thumbnail,
                                        &color,
                                    )
                                    .await?;
                                format!("{} added to monitoring list", name)
                                        .to_string()
                            }
                        }
                    }
                } else {
                    Err(Error::Plugin(
                        PluginError::InvalidSubCommand,
                    ))?
                }
            }
            "remove" => {
                if let ResolvedValue::SubCommand(subcommand_options) =
                    first_option.value.clone()
                {
                    info!("Subcommand options: {:?}", subcommand_options);
                    let id = option!(
                        subcommand_options,
                        "id",
                        ResolvedValue::String
                    )
                    .ok_or(PluginError::InvalidGameId)?;
                    let guild = command
                        .guild_id
                        .ok_or(PluginError::ExpectedGuild)?
                        .get();

                    db.remove_game(id, guild).await?;
                    format!("Game `{}` removed from monitoring list", id)
                } else {
                    Err(Error::Plugin(
                        PluginError::InvalidSubCommand,
                    ))?
                }
            }
            _ => "<:warn:1495130104613965994> **You've chosen... poorly.**\nPlease select a valid subcommand: `channel`, `add`, `update`, `list` or `remove`".to_string(),
        };

        // Send response
        command
            .create_response(
                &p_ctx.ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(response),
                ),
            )
            .await
            .map_err(|err| PluginError::FailedToRespond { err })?;

        Ok(())
    }
}
