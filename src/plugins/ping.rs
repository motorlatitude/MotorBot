use async_trait::async_trait;
use serenity::all::{
    Command, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, Interaction,
};
use tracing::info;

use crate::plugin::PluginError;
use crate::{Error, Result};

use crate::plugin::{MotorbotPlugin, PluginContext, PluginInfo};

pub struct PingPlugin;

/// A simple plugin that responds with "Pong!" when a user sends a "ping"
/// slash command. This plugin serves as a basic example of how to create a
/// plugin for the MotorBot system.
#[async_trait]
impl MotorbotPlugin for PingPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        PingPlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Ping".to_string(),
            description: "A simple ping command".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Ping Plugin is ready!");

        let _ = Command::create_global_command(
            &p_ctx.ctx.http,
            CreateCommand::new("ping").description("A simple ping command"),
        )
        .await;
        Ok(())
    }

    async fn on_interaction_create(
        &self,
        p_ctx: &PluginContext,
        interaction: &Interaction,
    ) -> Result<()> {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "ping" {
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Pong!"),
                        ),
                    )
                    .await
                    .map_err(|e| {
                        Error::Plugin(PluginError::FailedToRespond { err: e })
                    })?;
            }
        }
        Ok(())
    }
}
