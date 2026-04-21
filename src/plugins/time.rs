use async_trait::async_trait;
use serenity::all::{
    Command, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, Interaction,
};
use tracing::info;

use crate::plugin::PluginError;
use crate::{Error, Result};

use crate::plugin::{MotorbotPlugin, PluginContext, PluginInfo};

pub struct TimePlugin;

/// A simple plugin that responds with the current server time when a user sends
/// a "time" slash command. This plugin serves as a basic example of how to
/// create a plugin for the MotorBot system.
#[async_trait]
impl MotorbotPlugin for TimePlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        TimePlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Time".to_string(),
            description: "Tells the current server time".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Time Plugin is ready!");

        Command::create_global_command(
            &p_ctx.ctx.http,
            CreateCommand::new("time")
                .description("Returns server time for MotorBot"),
        )
        .await?;

        Ok(())
    }

    async fn on_interaction_create(
        &self,
        p_ctx: &PluginContext,
        interaction: &Interaction,
    ) -> Result<()> {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "time" {
                let current_time = chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(
                                format!(
                                    ":alarm_clock: The current time is: {}",
                                    current_time
                                ),
                            ),
                        ),
                    )
                    .await
                    .map_err(|err| {
                        Error::Plugin(PluginError::FailedToRespond { err })
                    })?;
            }
        }
        Ok(())
    }
}
