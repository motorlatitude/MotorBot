use std::{env, time::Duration};

use async_trait::async_trait;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serenity::all::{
    ChannelId, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    GuildId, Interaction, ResolvedValue,
};
use tracing::{error, info};

use crate::{
    plugin::{option, MotorbotPlugin, PluginContext, PluginError, PluginInfo},
    storage::{Database, GuildConfig, GuildConfigKey, GuildConfigValue},
    Error, Result,
};

#[derive(Deserialize, Debug)]
struct Response {
    body: Vec<Joke>,
    //success: bool
}

#[derive(Deserialize, Debug)]
struct Joke {
    _id: serde_json::Value,
    punchline: serde_json::Value,
    setup: serde_json::Value,
}

pub struct JokesPlugin;

impl JokesPlugin {
    async fn joke(
        http: std::sync::Arc<serenity::http::Http>,
        guilds: Vec<GuildId>,
    ) -> Result<()> {
        let mut db = Database::open().await?;
        let mut channel_ids = Vec::new();
        for guild in &guilds {
            let guild_id = guild.get();
            let jokes_channel_id = match db
                .get_guild_config(guild_id, GuildConfigKey::JokesChannel)
                .await
            {
                Ok(Some(config)) => match config.value {
                    GuildConfigValue::ChannelId(id) => Some(id),
                    _ => None,
                },
                Ok(None) => None,
                Err(why) => {
                    error!("Failed to get jokes channel from database for guild {}: {:?}", guild_id, why);
                    None
                }
            };
            if let Some(jokes_channel_id) = jokes_channel_id {
                channel_ids.push(ChannelId::new(jokes_channel_id));
            }
        }
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        let rapid_api_key: String =
            env::var("RAPID_API_KEY").map_err(|_| {
                Error::Plugin(PluginError::MissingEnvironmentVariable {
                    var_name: "RAPID_API_KEY".to_string(),
                })
            })?;
        headers.insert(
            "X-RapidAPI-Key",
            HeaderValue::from_str(&rapid_api_key)
                .unwrap_or(HeaderValue::from_static("")),
        );
        headers.insert(
            "X-RapidAPI-Host",
            HeaderValue::from_static("dad-jokes.p.rapidapi.com"),
        );

        let http_response = client
            .get("https://dad-jokes.p.rapidapi.com/random/joke")
            .headers(headers)
            .send()
            .await?;

        let joke = http_response.json::<Response>().await?;
        let Some(first_joke) = joke.body.first() else {
            error!("Dad jokes API returned an empty body array");
            return Err(Error::Plugin(
                PluginError::InvalidInternalAPIResponse {
                    err: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Empty joke array",
                    )),
                },
            ));
        };
        let setup = first_joke.setup.to_string();
        let punchline = first_joke.punchline.to_string();

        for channel_id in channel_ids {
            let pixel_animal_emojis = [
                "<:goat:1494789527921889371>",
                "<:bunny:1494789525740851393>",
                "<:cat:1494789524415451266>",
                "<:frog:1494789522779537501>",
                "<:dog:1494789521089237076>",
            ];
            let random_emoji = pixel_animal_emojis
                [rand::random_range(0..pixel_animal_emojis.len())];
            channel_id
                .send_message(
                    &http,
                    CreateMessage::new().content(format!(
                        "## {} Joke for {}\n{}\n\n||{}||",
                        random_emoji,
                        chrono::Local::now().format("%d %B %Y"),
                        setup,
                        punchline
                    )),
                )
                .await
                .map_err(|err| PluginError::FailedToRespond { err })?;
        }
        db.close().await?;
        Ok(())
    }
}

/// A plugin that shows a random joke at a set time every day
#[async_trait]
impl MotorbotPlugin for JokesPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        JokesPlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Jokes".to_string(),
            description: "Shows a random joke at a set time every day"
                .to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Jokes Plugin is ready!");

        // Register jokes command for all guilds
        let guilds = p_ctx.ctx.cache.guilds();
        for guild in &guilds {
            GuildId::create_command(
                *guild,
                &p_ctx.ctx.http,
                CreateCommand::new("jokes")
                    .description("Jokes command for MotorBot")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "channel",
                            "Set a channel for daily jokes",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::Channel,
                                "channel",
                                "The channel to post jokes in",
                            )
                            .required(true),
                        ),
                    ),
            )
            .await?;
        }

        let mut scheduler = AsyncScheduler::with_tz(chrono::Local);
        let http = p_ctx.ctx.http.clone();
        scheduler.every(1.day()).at("10:30 am").run(move || {
            let http = http.clone();
            let guilds = guilds.clone();
            async move {
                JokesPlugin::joke(http, guilds).await.unwrap_or_else(|err| {
                    error!("Failed to send joke: {:?}", err);
                });
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
        if command.data.name != "jokes" {
            // Not the "jokes" command, so we can skip processing
            return Ok(());
        }

        let options = command.data.options();
        let first_option =
            options.first().ok_or(PluginError::InvalidSubCommand)?;
        match first_option.name {
            "channel" => {
                let ResolvedValue::SubCommand(subcommand_options) =
                    first_option.value.clone()
                else {
                    return Err(Error::Plugin(PluginError::InvalidSubCommand));
                };
                let channel = option!(
                    subcommand_options,
                    "channel",
                    ResolvedValue::Channel
                )
                .ok_or(PluginError::MissingChannelId)?;
                let guild_id =
                    command.guild_id.ok_or(PluginError::ExpectedGuild)?;
                let mut db = Database::open().await?;
                let new_value = GuildConfig::from((
                    GuildConfigKey::JokesChannel,
                    GuildConfigValue::ChannelId(channel.id.get()),
                ));
                db.set_guild_config(guild_id.get(), new_value).await?;
                command.create_response(
                    &p_ctx.ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("<:dog:1494789521089237076> Jokes will be sent to <#{}> at the scheduled time", channel.id.get()))
                    )
                )
                .await
                .map_err(|err| PluginError::FailedToRespond { err })?;
                db.close().await?;
            }
            _ => {
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("<:warn:1495130104613965994> **You've chosen... poorly.**\nPlease select a valid subcommand: `channel`."),
                        ),
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
        }

        Ok(())
    }
}
