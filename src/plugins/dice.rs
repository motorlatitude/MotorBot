use async_trait::async_trait;
use serenity::all::{
    Command, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, Interaction,
    ResolvedValue,
};
use tracing::info;

use crate::plugin::{option, PluginError};
use crate::Result;

use crate::plugin::{MotorbotPlugin, PluginContext, PluginInfo};

pub struct DicePlugin;

/// A simple plugin that responds with a random dice roll when a user sends
/// a "roll" slash command. This plugin serves as a basic example of how to
/// create a plugin for the MotorBot system.
#[async_trait]
impl MotorbotPlugin for DicePlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        DicePlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Dice".to_string(),
            description: "Dice roll command".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("DicePlugin is ready!");

        Command::create_global_command(
            &p_ctx.ctx.http,
            CreateCommand::new("roll")
                .description("Roll the dice, what will you get?")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "sides",
                        "Number of sides on the dice",
                    )
                    .required(false),
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
            // Not a command interaction, ignore
            return Ok(());
        };
        if command.data.name != "roll" {
            // Not the "roll" command, so we can skip processing
            return Ok(());
        }

        let options = &command.data.options();
        let sides = option!(options, "sides", ResolvedValue::Integer)
            .ok_or_else(|| 6i64)
            .unwrap_or(6i64); // Default to a 6-sided die if no option provided

        // clamp sides to a reasonable range (e.g., 2 to 100)
        let sides = sides.clamp(2, 100);

        let dice_roll = rand::random_range(1..=sides);
        command
            .create_response(
                &p_ctx.ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!(
                        ":game_die: You rolled a **{}**",
                        dice_roll
                    )),
                ),
            )
            .await
            .map_err(|err| PluginError::FailedToRespond { err })?;

        Ok(())
    }
}
