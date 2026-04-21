use async_trait::async_trait;
use serenity::all::{Interaction, Message, Reaction};

pub mod error;
pub mod plugin_context;
pub mod plugin_info;

// Re-exporting for easier access
pub use error::Error as PluginError;
pub use plugin_context::PluginContext;

pub use crate::Result;

#[allow(unused)]
pub use plugin_info::PluginInfo;

/// Default trait definition for plugins in the MotorBot system
///
/// This trait defines the basic interface for all plugins in the MotorBot
/// system.
#[async_trait]
pub trait MotorbotPlugin {
    /// Creates a new instance of the plugin.
    ///
    /// This method is used to create a new instance of the plugin. It should be
    /// implemented by all plugins to allow for dynamic loading and initialization.
    ///
    /// Please note that at this point MotorBot may not yet have fully
    /// initialized, so plugins should avoid making any assumptions about the
    /// state of the system in this method. For example, plugins should not
    /// attempt to access the database or send messages to Discord in this method,
    /// as these systems may not yet be ready.
    fn new() -> Self
    where
        Self: Sized;

    /// Returns the plugin's information, including its name, description, and
    /// version.
    fn info(&self) -> PluginInfo;

    /// Called when motorbot is ready
    ///
    /// This method is called when motorbot is ready to start processing events.
    /// It can be used to perform any necessary initialization or setup for
    /// plugins.
    async fn on_ready(&self, _ctx: &PluginContext) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a message is created
    ///
    /// This method is called when a new message is created in any channel that
    /// the bot has access to. It can be used to perform any functionality that
    /// requires processing messages.
    async fn on_message(
        &self,
        _ctx: &PluginContext,
        _message: &Message,
    ) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a reaction is added to a message
    ///
    /// This method is called when a reaction is added to any message that the bot
    /// has access to. It can be used to perform any functionality that requires
    /// processing reactions.
    async fn on_reaction_add(
        &self,
        _ctx: &PluginContext,
        _reaction: &Reaction,
    ) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a reaction is removed from a message
    ///
    /// This method is called when a reaction is removed from any message that the
    /// bot has access to. It can be used to perform any functionality that
    /// requires processing reaction removals.
    async fn on_reaction_remove(
        &self,
        _ctx: &PluginContext,
        _reaction: &Reaction,
    ) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when an interaction is created
    async fn on_interaction_create(
        &self,
        _ctx: &PluginContext,
        _interaction: &Interaction,
    ) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Macro to simplify getting options from command interactions
///
/// # Example
/// To get an option named channel with a value of the Channel type, you can use
/// the macro like this:
/// ```rust
/// let options = command.data.options();
/// let channel_id = option!(
///   options,
///   "channel",
///   ResolvedValue::Channel
/// ).ok_or(PluginError::MissingChannelId)?.id.get();
/// ```
macro_rules! option {
    ($cmd:ident, $name:expr, $t:path) => {{
        let option = $cmd.iter().find(|opt| opt.name == $name);
        if let Some(option) = option {
            if let $t(value) = option.value {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }};
}

pub(crate) use option;
