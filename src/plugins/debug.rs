use async_trait::async_trait;
use serenity::all::{
    Command, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, Interaction,
    MessageBuilder, ResolvedValue,
};
use tracing::info;

use crate::plugin::PluginError;
use crate::{Error, Result};

use crate::{
    plugin::{MotorbotPlugin, PluginContext, PluginInfo},
    storage::{Database, GuildConfig, GuildConfigKey, GuildConfigValue},
};

pub struct DebugPlugin;

/// A simple plugin that responds with a random dice roll when a user sends
/// a "roll" slash command. This plugin serves as a basic example of how to
/// create a plugin for the MotorBot system.
#[async_trait]
impl MotorbotPlugin for DebugPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        DebugPlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Debug".to_string(),
            description: "MotorBot debug command".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Debug Plugin is ready!");

        Command::create_global_command(
            &p_ctx.ctx.http,
            CreateCommand::new("debug")
                .description("Debug command for MotorBot")
                .add_option(CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "whoami",
                    "Get information about the user who invoked the command",
                ))
                .add_option(CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "version",
                    "Get the bot's version",
                ))
                .add_option(CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "plugins",
                    "Get a list of all loaded plugins",
                ))
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "events",
                        "Event log configuration",
                    )
                    .add_sub_option(
                        CreateCommandOption::new(
                            CommandOptionType::Channel,
                            "channel",
                            "Channel to send events to",
                        )
                        .required(true),
                    ),
                ),
        )
        .await?;
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
        if command.data.name != "debug" {
            // Not the "debug" command, so we can skip processing
            return Ok(());
        }

        let options = command.data.options();
        let first_option =
            options.first().ok_or(PluginError::InvalidSubCommand)?;
        match first_option.name {
            "whoami" => {
                let avatar = match command.user.avatar.as_ref() {
                    Some(avatar) => format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        command.user.id.get(),
                        avatar
                    ),
                    None => "No avatar".to_string(),
                };
                let public_flags =
                    command.user.public_flags.unwrap_or_default();
                let response = MessageBuilder::new()
                    .push("```")
                    .push(format!(
                        "id            : {:?}\n",
                        command.user.id.get()
                    ))
                    .push(format!("name          : {:?}\n", command.user.name))
                    .push(format!(
                        "global name   : {:?}\n",
                        command.user.global_name
                    ))
                    .push(format!(
                        "discriminator : {:?}\n",
                        command.user.discriminator
                    ))
                    .push(format!("avatar        : {:?}\n", avatar))
                    .push(format!("flags         : {:?}\n", public_flags))
                    .push(format!(
                        "accent colour : {:?}\n",
                        command.user.accent_colour
                    ))
                    .push(format!(
                        "banner        : {:?}\n",
                        command.user.banner
                    ))
                    .push(format!("bot           : {:?}\n", command.user.bot))
                    .push(format!(
                        "system        : {:?}\n",
                        command.user.system
                    ))
                    .push("```")
                    .build();
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(response),
                        ),
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
            "plugins" => {
                let plugins = p_ctx.plugins;
                let plugin_list = plugins
                    .iter()
                    .map(|p| {
                        format!(
                            "🧩 `v{}` **{}**: {}",
                            p.info().version,
                            p.info().name,
                            p.info().description
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(
                                format!("Loaded plugins:\n{}", plugin_list),
                            ),
                        ),
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
            "version" => {
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(
                                format!(
                                    "MotorBot version: v{}",
                                    env!("CARGO_PKG_VERSION")
                                ),
                            ),
                        ),
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
            "events" => {
                if let ResolvedValue::SubCommand(subcommand) =
                    &first_option.value
                {
                    let channel_option = subcommand
                        .iter()
                        .find(|opt| opt.name == "channel")
                        .ok_or(PluginError::MissingChannelId)?;

                    let ResolvedValue::Channel(channel_id) =
                        channel_option.value
                    else {
                        return Err(Error::Plugin(
                            PluginError::MissingChannelId,
                        ));
                    };
                    let mut db = Database::open().await?;
                    let config = GuildConfig::from((
                        GuildConfigKey::EventsChannel,
                        GuildConfigValue::ChannelId(channel_id.id.get()),
                    ));
                    let guild_id =
                        command.guild_id.ok_or(PluginError::ExpectedGuild)?;
                    db.set_guild_config(guild_id.get(), config).await?;

                    command
                        .create_response(
                            &p_ctx.ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "Events channel set to <#{}>",
                                        channel_id.id.get()
                                    )),
                            ),
                        )
                        .await
                        .map_err(|err| PluginError::FailedToRespond { err })?;
                }
            }
            _ => {
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("<:warn:1495130104613965994> **You've chosen... poorly.**\nPlease select a valid subcommand: `whoami`, `version`, `plugins`, or `events`."),
                        ),
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
        }

        Ok(())
    }
}
