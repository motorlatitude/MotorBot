use async_trait::async_trait;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, EmojiId,
    GuildId, Interaction, Message, Reaction, ReactionType, ResolvedValue,
};
use tracing::info;

use crate::plugin::{option, PluginError};
use crate::{Error, Result};

use crate::{
    plugin::{MotorbotPlugin, PluginContext, PluginInfo},
    storage::{Database, GuildConfig, GuildConfigKey, GuildConfigValue},
};

pub struct PointsPlugin;

impl PointsPlugin {
    /// Adds points to a user.
    ///
    /// This is a helper function that can be called from
    /// various places in the plugin where points need to be added, such as when a
    /// user receives an upvote reaction on a message.
    async fn add_points(&self, user_id: &u64, points: i32) -> Result<()> {
        let mut db = Database::open().await?;
        let current_user_score = db.user_score(user_id).await?;
        db.set_user_score(user_id, current_user_score.score + points)
            .await?;
        Ok(())
    }

    /// Subtracts points from a user.
    ///
    /// This is a helper function that can be called
    /// from various places in the plugin where points need to be subtracted, such
    /// as when a user receives a downvote reaction on a message.
    async fn subtract_points(&self, user_id: &u64, points: i32) -> Result<()> {
        let mut db = Database::open().await?;
        let current_user_score = db.user_score(user_id).await?;
        db.set_user_score(user_id, current_user_score.score - points)
            .await?;
        Ok(())
    }
}

/// A plugin that creates a user points system. Users can check their own points
/// or someone else's points using the "points" slash command. Additionally,
/// certain channels can be configured to add vote reactions to messages,
/// allowing for an upvote/downvote system that contributes to user points.
#[async_trait]
impl MotorbotPlugin for PointsPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        PointsPlugin
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Points".to_string(),
            description: "A user points system with upvote/downvote reactions"
                .to_string(),
            version: "0.1.0".to_string(),
        }
    }

    async fn on_ready(&self, p_ctx: &PluginContext) -> Result<()> {
        info!("Points Plugin is ready!");

        for guild in p_ctx.ctx.cache.guilds() {
            let _ = GuildId::create_command(
                guild,
                &p_ctx.ctx.http,
                CreateCommand::new("points").description("Check your or someone else's points")
                    .add_option(CreateCommandOption::new(CommandOptionType::User, "user", "The user's score to look up").required(false))
                    .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "channel", "Set a channel that should allow users to vote on messages")
                        .add_sub_option(CreateCommandOption::new(CommandOptionType::Channel, "channel", "The channel to add vote reactions to").required(true))
                    ),
            ).await;
        }
        Ok(())
    }

    async fn on_message(
        &self,
        p_ctx: &PluginContext,
        message: &Message,
    ) -> Result<()> {
        let Some(msg_guild_id) = message.guild_id else {
            // Message is not in a guild, so we can skip processing
            return Ok(());
        };

        let mut db = Database::open().await?;
        let Some(stored_channel_ids) = db
            .get_guild_config(
                msg_guild_id.get(),
                GuildConfigKey::PointsChannels,
            )
            .await?
        else {
            // No channels configured for this guild, so we can skip processing
            return Ok(());
        };

        if let GuildConfigValue::ChannelIds(channel_ids) =
            stored_channel_ids.value()
        {
            let bot_user_id = p_ctx.ctx.cache.current_user().id.get();
            if (!message.attachments.is_empty()
                || message.content.contains("http"))
                && channel_ids.contains(&message.channel_id.get())
                && !message.author.id.eq(&bot_user_id)
            {
                message
                    .react(
                        &p_ctx.ctx.http,
                        ReactionType::Custom {
                            animated: false,
                            id: EmojiId::new(1494772872500088913),
                            name: Some("upvote".to_string()),
                        },
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
                message
                    .react(
                        &p_ctx.ctx.http,
                        ReactionType::Custom {
                            animated: false,
                            id: EmojiId::new(1494772826945617940),
                            name: Some("downvote".to_string()),
                        },
                    )
                    .await
                    .map_err(|err| PluginError::FailedToRespond { err })?;
            }
        }
        Ok(())
    }

    async fn on_reaction_add(
        &self,
        p_ctx: &PluginContext,
        reaction: &Reaction,
    ) -> Result<()> {
        let Some(guild_id) = reaction.guild_id else {
            // Reaction is not in a guild, so we can skip processing
            return Ok(());
        };

        let reaction_user_id =
            reaction.user_id.ok_or(PluginError::MissingUserId)?;
        let bot_user_id = p_ctx.ctx.cache.current_user().id.get();

        let mut db = Database::open().await?;
        let Some(stored_channel_ids) = db
            .get_guild_config(guild_id.get(), GuildConfigKey::PointsChannels)
            .await?
        else {
            // No channels configured for this guild, so we can skip processing
            return Ok(());
        };

        if let GuildConfigValue::ChannelIds(channel_ids) =
            stored_channel_ids.value()
        {
            if channel_ids.contains(&reaction.channel_id.get())
                && !reaction_user_id.eq(&bot_user_id)
            {
                let message = p_ctx
                    .ctx
                    .http
                    .get_message(reaction.channel_id, reaction.message_id)
                    .await?;
                let user_id = message.author.id;
                match reaction.emoji {
                    ReactionType::Custom { id, .. }
                        if id == 1494772872500088913 =>
                    {
                        // Add points to the user who sent the message
                        self.add_points(&user_id.get(), 1).await?;
                    }
                    ReactionType::Custom { id, .. }
                        if id == 1494772826945617940 =>
                    {
                        // Subtract points from the user who sent the message
                        self.subtract_points(&user_id.get(), 1).await?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn on_reaction_remove(
        &self,
        p_ctx: &PluginContext,
        reaction: &Reaction,
    ) -> Result<()> {
        let Some(guild_id) = reaction.guild_id else {
            // Reaction is not in a guild, so we can skip processing
            return Ok(());
        };

        let reaction_user_id =
            reaction.user_id.ok_or(PluginError::MissingUserId)?;
        let bot_user_id = p_ctx.ctx.cache.current_user().id.get();

        let mut db = Database::open().await?;
        let Some(stored_channel_ids) = db
            .get_guild_config(guild_id.get(), GuildConfigKey::PointsChannels)
            .await?
        else {
            // No channels configured for this guild, so we can skip processing
            return Ok(());
        };

        if let GuildConfigValue::ChannelIds(channel_ids) =
            stored_channel_ids.value()
        {
            if channel_ids.contains(&reaction.channel_id.get())
                && !reaction_user_id.eq(&bot_user_id)
            {
                let message = p_ctx
                    .ctx
                    .http
                    .get_message(reaction.channel_id, reaction.message_id)
                    .await?;
                let user_id = message.author.id;
                match reaction.emoji {
                    ReactionType::Custom { id, .. }
                        if id == 1494772872500088913 =>
                    {
                        // Subtract previously added points from the user who sent the message
                        self.subtract_points(&user_id.get(), 1).await?;
                    }
                    ReactionType::Custom { id, .. }
                        if id == 1494772826945617940 =>
                    {
                        // Add previously subtracted points to the user who sent the message
                        self.add_points(&user_id.get(), 1).await?;
                    }
                    _ => {}
                }
            }
        }
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
        if command.data.name != "points" {
            // Not the "points" command, so we can skip processing
            return Ok(());
        }
        let mut db = Database::open().await?;

        let options = command.data.options();
        let first_option = options.first();
        if let Some(resolved_option) = first_option {
            match resolved_option.name {
                "channel" => {
                    let ResolvedValue::SubCommand(subcommand) =
                        resolved_option.value.clone()
                    else {
                        return Err(Error::Plugin(
                            PluginError::InvalidSubCommand,
                        ));
                    };
                    let channel = subcommand
                        .iter()
                        .find_map(|opt| {
                            if opt.name == "channel" {
                                if let ResolvedValue::Channel(channel) =
                                    opt.value
                                {
                                    return Some(channel);
                                }
                            }
                            None
                        })
                        .ok_or(PluginError::InvalidChannel)?;
                    let guild_id =
                        command.guild_id.ok_or(PluginError::ExpectedGuild)?;

                    let mut channel_ids = match db
                        .get_guild_config(
                            guild_id.get(),
                            GuildConfigKey::PointsChannels,
                        )
                        .await?
                    {
                        Some(config) => match config.value {
                            GuildConfigValue::ChannelIds(ids) => ids,
                            _ => Vec::new(),
                        },
                        None => Vec::new(),
                    };

                    let response_msg = if !channel_ids
                        .contains(&channel.id.get())
                    {
                        channel_ids.push(channel.id.get());

                        let new_value = GuildConfig::from((
                            GuildConfigKey::PointsChannels,
                            GuildConfigValue::ChannelIds(channel_ids.clone()),
                        ));
                        db.set_guild_config(guild_id.get(), new_value).await?;

                        format!("Channel <#{}> has been added to the points system!", channel.id.get())
                    } else {
                        format!(
                            "Channel <#{}> is already part of the points system.",
                            channel.id.get()
                        )
                    };

                    command
                        .create_response(
                            &p_ctx.ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(response_msg),
                            ),
                        )
                        .await
                        .map_err(|err| PluginError::FailedToRespond { err })?;
                }
                "user" => {
                    let user: &serenity::all::User = option! {
                        options,
                        "user",
                        ResolvedValue::User
                    }
                    .ok_or(PluginError::InvalidUser)?;
                    let username = user.tag();
                    let user_score = db.user_score(&user.id.get()).await?;
                    command.create_response(
                        &p_ctx.ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!("<:pouch:1494773684295041256> {}'s score is {}", username, user_score.score))
                        )
                    )
                    .await
                    .map_err(|err| {
                        PluginError::FailedToRespond { err }
                    })?;
                }
                _ => {
                    command
                        .create_response(
                            &p_ctx.ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("<:warn:1495130104613965994> **You've chosen... poorly.**\nPlease select a valid subcommand: `user` or `channel`."),
                            ),
                        )
                        .await
                        .map_err(|err| PluginError::FailedToRespond { err })?;
                }
            }
        } else {
            // No sub command or user provided, default to showing the user's
            // own points
            let user_id = command.user.id;
            let username = command.user.tag();

            let user_score = db.user_score(&user_id.get()).await?;
            command
                .create_response(
                    &p_ctx.ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(
                            format!(
                                "<:pouch:1494773684295041256> {}'s score is {}",
                                username, user_score.score
                            ),
                        ),
                    ),
                )
                .await
                .map_err(|err| PluginError::FailedToRespond { err })?;
        }
        db.close().await?;
        Ok(())
    }
}
