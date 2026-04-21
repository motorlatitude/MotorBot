use async_trait::async_trait;
use serenity::all::{
    Command, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, Interaction,
};
use tracing::info;

use crate::plugin::PluginError;
use crate::{Error, Result};

use crate::plugin::{MotorbotPlugin, PluginContext, PluginInfo};

pub struct CoinTossPlugin;

/// A simple plugin that responds with a random coin toss result when a user sends
/// a "heads-or-tails" slash command. This plugin serves as a basic example of how to
/// create a plugin for the MotorBot system.
#[async_trait]
impl MotorbotPlugin for CoinTossPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        CoinTossPlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Coin Toss".to_string(),
            description: "A simple coin toss command".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Coin Toss Plugin is ready!");

        let _ = Command::create_global_command(
            &p_ctx.ctx.http,
            CreateCommand::new("heads-or-tails")
                .description("Heads or tails? Let's find out!"),
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
            if command.data.name == "heads-or-tails" {
                let coin_toss = if rand::random() { "Heads" } else { "Tails" };
                command
                    .create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!(":coin: **{}**", coin_toss)),
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
