use dotenv::dotenv;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json;
use serenity::all::{
    ActivityData, Command, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, CreateMessage, Interaction,
};
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::GuildId;
use std::convert::TryFrom;
use std::env;
use std::time::Duration;
use std::u64;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use oai_rs::{completions, images, models};

use clokwerk::{AsyncScheduler, Job, TimeUnits};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::model::prelude::ReactionType;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

mod storage;
use storage::{Database, GuildConfig, GuildConfigKey};

mod plugins;
use crate::plugins::patches::game_data::GameData;
use crate::plugins::patches::patches_plugin;
use crate::plugins::patches::platforms::platform::Platform;

struct Handler;

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

pub enum MotorbotChannels {
    General = 130734377066954752,
    BotEvents = 432351112616738837,
    PatchNotes = 438307738250903553,
    Jokes = 1040719087585742980,
}

pub enum MotorBotGuilds {
    MotorBot = 130734377066954752,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        debug!("Message");
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                error!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!whoami" {
            let response = MessageBuilder::new()
                .push("```")
                .push(format!("id            : {:?}\n", msg.author.id.get()))
                .push(format!("name          : {:?}\n", msg.author.name))
                .push(format!("discriminator : {:?}\n", msg.author.discriminator))
                .push(format!(
                    "avatar        : {:?}\n",
                    msg.author.avatar.as_ref().unwrap()
                ))
                .push(format!(
                    "flags         : {:?}\n",
                    msg.author.public_flags.unwrap()
                ))
                .push(format!("accent colour : {:?}\n", msg.author.accent_colour))
                .push(format!("banner        : {:?}\n", msg.author.banner))
                .push("```")
                .build();
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {:?}", why);
            }
        }

        if msg.content.contains("kys") {
            let response = MessageBuilder::new()
                .push("Calm down before you hurt yourself ")
                .user(msg.author.id)
                .build();
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {:?}", why);
            }
        }

        let channel_ids: Vec<u64> = vec![
            MotorbotChannels::General as u64,
            MotorbotChannels::PatchNotes as u64,
            MotorbotChannels::Jokes as u64,
        ];

        if msg.attachments.len() > 0 || msg.content.contains("http") {
            if channel_ids.contains(&msg.channel_id.get()) && !msg.author.id.eq(&169554882674556930)
            {
                if let Err(why) = msg
                    .react(
                        &ctx,
                        ReactionType::try_from("<:upvote:429449534389616641>").unwrap(),
                    )
                    .await
                {
                    error!("Failed to react to message {:?}", why);
                }
                if let Err(why) = msg
                    .react(
                        &ctx,
                        ReactionType::try_from("<:downvote:429449638454493187>").unwrap(),
                    )
                    .await
                {
                    error!("Failed to react to message {:?}", why);
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Added");
        let channel_ids: Vec<u64> = vec![
            MotorbotChannels::General as u64,
            MotorbotChannels::PatchNotes as u64,
            MotorbotChannels::Jokes as u64,
        ];

        if channel_ids.contains(&reaction.channel_id.get())
            && reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap()
            && !reaction.user_id.unwrap().eq(&169554882674556930)
        {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx
                .http
                .get_message(reaction.channel_id, message_id)
                .await
                .unwrap()
                .author
                .id; // The user id of the message that was upvoted

            info!(
                "User {:?} upvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let mut db = Database::open()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            let new_score = user_score.score + 1;
            if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                error!("Failed to set user score {:?}", why);
            }
            if let Err(why) = db.close().await {
                error!("Failed to close database connection {:?}", why);
            }
        } else if channel_ids.contains(&reaction.channel_id.get())
            && reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap()
            && !reaction.user_id.unwrap().eq(&169554882674556930)
        {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx
                .http
                .get_message(reaction.channel_id, message_id)
                .await
                .unwrap()
                .author
                .id; // The user id of the message that was downvoted
            info!(
                "User {:?} downvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let mut db = Database::open()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            let new_score = user_score.score - 1;
            if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                error!("Failed to set user score {:?}", why);
            }
            if let Err(why) = db.close().await {
                error!("Failed to close database connection {:?}", why);
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Removed");
        let channel_ids: Vec<u64> = vec![
            MotorbotChannels::General as u64,
            MotorbotChannels::PatchNotes as u64,
            MotorbotChannels::Jokes as u64,
        ];

        if channel_ids.contains(&reaction.channel_id.get())
            && reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap()
            && !reaction.user_id.unwrap().eq(&169554882674556930)
        {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx
                .http
                .get_message(reaction.channel_id, message_id)
                .await
                .unwrap()
                .author
                .id; // The user id of the message that was the upvote was removed from

            info!(
                "User {:?} upvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let mut db = Database::open()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            let new_score = user_score.score - 1;
            if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                error!("Failed to set user score {:?}", why);
            }
            if let Err(why) = db.close().await {
                error!("Failed to close database connection {:?}", why);
            }
        } else if channel_ids.contains(&reaction.channel_id.get())
            && reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap()
            && !reaction.user_id.unwrap().eq(&169554882674556930)
        {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx
                .http
                .get_message(reaction.channel_id, message_id)
                .await
                .unwrap()
                .author
                .id; // The user id of the message that was downvoted was removed from
            info!(
                "User {:?} downvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let mut db = Database::open()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            let new_score = user_score.score + 1;
            if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                error!("Failed to set user score {:?}", why);
            }
            if let Err(why) = db.close().await {
                error!("Failed to close database connection {:?}", why);
            }
        }
    }

    // Ready Event
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(Some(ActivityData::custom("😶‍🌫️")), OnlineStatus::Online);

        // let avatar = serenity::utils::read_image("./motorbot.png");
        // match avatar {
        //     Ok(av) => {
        //         info!("Avatar read successfully");
        //         let mut user = ctx.cache.current_user();
        //         let avatar_edit = user.edit(&ctx.http, |p| p.avatar(Some(&av))).await;
        //         match avatar_edit {
        //             Ok(_) => {
        //                 info!("Avatar updated successfully");
        //             }
        //             Err(why) => {
        //                 error!("Error updating avatar: {:?}", why);
        //             }
        //         }
        //     }
        //     Err(why) => {
        //         error!("Error reading avatar: {:?}", why);
        //     }
        // }

        let channel_id = ChannelId::new(MotorbotChannels::BotEvents as u64);
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        if let Err(why) = channel_id
            .say(&ctx.http, format!("MotorBot v{} reporting for duty!", VERSION))
            .await
        {
            error!("Error sending message: {:?}", why);
        }

        let loaded_patch_plugin = patches_plugin::PatchesPlugin::new(ctx.clone()).await;

        // Register Slash Commands
        let commands_list = vec![
            CreateCommand::new("time").description("Returns server time for MotorBot"),
            CreateCommand::new("roll").description("Roll the dice, what will you get?"),
            CreateCommand::new("headsortails").description("Heads or tails?"),
            CreateCommand::new("score")
                .description("Get a user's score")
                .add_option( CreateCommandOption::new(CommandOptionType::User, "user", "The user's score to look up").required(false)),
            CreateCommand::new("ai")
                .description("Allows you to interact with OpenAI")
                .add_option(CreateCommandOption::new(CommandOptionType::String, "endpoint", "The type of interaction to use for OpenAI either `completions` or `images`").required(true))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "prompt", "The prompt to use for OpenAI").required(true)),
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
        ];

        let mut guild_ids = Vec::new();
        ctx.http.get_guilds(None, None).await.unwrap().iter().for_each(|guild| {
            guild_ids.push(guild.id);
        });
        info!("Registering slash commands for guilds: {:?}", guild_ids);
        for guild_id in guild_ids {
            let _commands = GuildId::set_commands(guild_id, &ctx.http, commands_list.clone()).await;
        }

        let _ = Command::create_global_command(
            &ctx.http,
            CreateCommand::new("ping").description("A simple ping command"),
        )
        .await;

        let mut scheduler = AsyncScheduler::with_tz(chrono::Local);
        // Add some tasks to it
        let inner_ctx = loaded_patch_plugin.clone();
        scheduler.every(30.minutes()).run(move || {
            let inner_inner_ctx = inner_ctx.clone();
            async move {
                inner_inner_ctx.update().await;
            }
        });
        scheduler.every(1.day()).at("10:30 am").run(move || {
            let ctx = ctx.clone();
            async move {
                let channel_id = ChannelId::new(MotorbotChannels::Jokes as u64);
                let client = reqwest::Client::new();

                let mut headers = HeaderMap::new();
                let rapid_api_key: String =
                    env::var("RAPID_API_KEY").expect("Expected a token in the environment");
                headers.insert(
                    "X-RapidAPI-Key",
                    HeaderValue::from_str(&rapid_api_key).unwrap(),
                );
                headers.insert(
                    "X-RapidAPI-Host",
                    HeaderValue::from_str("dad-jokes.p.rapidapi.com").unwrap(),
                );

                let http_response = client
                    .get("https://dad-jokes.p.rapidapi.com/random/joke")
                    .headers(headers)
                    .send()
                    .await
                    .expect("Failed to request joke")
                    .json::<Response>()
                    .await
                    .expect("Failed to parse joke");

                let _ = channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new()
                            .content(format!(
                                "{}\n\n||{}||",
                                http_response.body[0].setup.as_str().unwrap(),
                                http_response.body[0].punchline.as_str().unwrap()
                            ))
                            .reactions([
                                ReactionType::try_from("<:upvote:429449534389616641>").unwrap(),
                                ReactionType::try_from("<:downvote:429449638454493187>").unwrap(),
                            ]),
                    )
                    .await;
            }
        });

        tokio::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    // Handle Slash Command Trigger
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received command interaction: {:#?}", command.data.name);

            let content = match command.data.name.as_str() {
                "ping" => "Pong!".to_string(),
                "time" => MessageBuilder::new()
                    .push(":alarm_clock: The current time for MotorBot is ")
                    .push(format!("{:?}", chrono::Local::now()))
                    .build(),
                "roll" => {
                    let roll = rand::random_range(1..100);
                    MessageBuilder::new()
                        .push("You rolled a ")
                        .push_bold_safe(format!("{}", roll))
                        .build()
                }
                "headsortails" => {
                    let roll = rand::random_range(1..100);
                    if roll >= 50 {
                        MessageBuilder::new().push(":coin: Heads").build()
                    } else {
                        MessageBuilder::new().push(":coin: Tails").build()
                    }
                }
                "score" => {
                    let mut user_id = command.user.id;
                    let mut username = command.user.tag();

                    let options = command.data.options.get(0);
                    info!("Options: {:?}", options);
                    if !options.is_none() {
                        let option = options.expect("Expected User Id").value.clone();
                        if let CommandDataOptionValue::User(user) = option {
                            user_id = user;
                        }
                        username = ctx
                            .http
                            .get_user(user_id)
                            .await
                            .expect("Failed to fetch user")
                            .tag();
                    }

                    let mut db = Database::open()
                        .await
                        .expect("Failed to connect to database");
                    let user_score = db
                        .user_score(&user_id.get())
                        .await
                        .expect("Failed to fetch user score");
                    if let Err(why) = db.close().await {
                        error!("Failed to close database connection {:?}", why);
                    }
                    format!("{}'s score is {}", username, user_score.score)
                }
                "ai" => {
                    "Generating response...".to_string()
                }
                "patches" => {
                    let subcommand = command.data.options.get(0);
                    if let Some(subcommand) = subcommand {
                        match subcommand.name.as_str() {
                            "channel" => {
                                if let CommandDataOptionValue::SubCommand(subcommand_options) = subcommand.value.clone() {
                                    info!("Subcommand options: {:?}", subcommand_options);
                                    let channel_id = subcommand_options
                                        .get(0)
                                        .expect("Expected channel")
                                        .value
                                        .as_channel_id()
                                        .unwrap().get();
                                    let guild_id = command.guild_id.unwrap().get();
                                    let mut db = Database::open()
                                        .await
                                        .expect("Failed to connect to database");
                                    let config_option = GuildConfig::from((GuildConfigKey::PatchNotesChannel, channel_id.to_string()));
                                    let result = db.set_guild_config(guild_id, config_option).await;
                                    match result {
                                        Ok(_) => format!("Patch notes channel set to <#{}>", channel_id),
                                        Err(e) => {
                                            error!("Failed to set patch notes channel: {:?}", e);
                                            "Failed to set patch notes channel".to_string()
                                        }
                                    }
                                } else {
                                    "No subcommand options provided".to_string()
                                }
                            },
                            "list" => {
                                let mut db = Database::open()
                                    .await
                                    .expect("Failed to connect to database");
                                let game_ids = db.game_ids_for_guild(command.guild_id.unwrap().get()).await;
                                let games_to_monitor = match game_ids {
                                    Ok(ids) => ids,
                                    Err(e) => {
                                        error!("Failed to fetch game ids: {:?}", e);
                                        return;
                                    }
                                };
                                let mut response = String::new();
                                let mut count = 1;
                                for game_id in games_to_monitor {
                                    let game_data = GameData::from_id(&game_id).await;
                                    response.push_str(&format!(
                                        "{}. {} ({})\n",
                                        count, game_data.name, game_data.id
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
                                let _ = patches_plugin::PatchesPlugin::new(ctx.clone()).await;
                                "Update complete".to_string()
                            }
                            "add" => {
                                if let CommandDataOptionValue::SubCommand(subcommand_options) = subcommand.value.clone() {
                                    info!("Subcommand options: {:?}", subcommand_options);
                                    let id = subcommand_options
                                        .get(0)
                                        .expect("Expected game id")
                                        .value
                                        .as_str()
                                        .unwrap();
                                    let platform = subcommand_options
                                        .get(1)
                                        .expect("Expected platform")
                                        .value
                                        .as_str()
                                        .unwrap();
                                    if !Platform::is_valid_platform(platform) {
                                        format!("Invalid platform provided. Valid platforms are: steam, riot")
                                    } else {
                                        let name = subcommand_options
                                            .get(2)
                                            .expect("Expected name")
                                            .value
                                            .as_str()
                                            .unwrap();
                                        let thumbnail = subcommand_options
                                            .get(3)
                                            .expect("Expected thumbnail")
                                            .value
                                            .as_str()
                                            .unwrap();
                                        if !thumbnail.starts_with("http") {
                                            format!("Invalid thumbnail URL provided. URL should start with http")
                                        } else {
                                            let raw_color = subcommand_options
                                                .get(4)
                                                .expect("Expected color")
                                                .value
                                                .as_str()
                                                .unwrap();
                                            if !raw_color.starts_with("#") || raw_color.len() != 7 {
                                                format!("Invalid color provided. Color should be a hex code starting with # and followed by 6 characters (e.g. #FF0000)")
                                            } else {
                                                let color = raw_color[1..].to_uppercase();
                                                let guild = command.guild_id.unwrap().get();
                                                let mut db = Database::open()
                                                    .await
                                                    .expect("Failed to connect to database");
                                                let result = db.add_game(id, guild, Platform::from(platform), name, thumbnail, &color).await;
                                                match result {
                                                    Ok(_) => format!("Game {} added to monitoring list", name),
                                                    Err(e) => {
                                                        error!("Failed to add game: {:?}", e);
                                                        "Failed to add game".to_string()
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    "No subcommand options provided".to_string()
                                }
                            },
                            "remove" => {
                                if let CommandDataOptionValue::SubCommand(subcommand_options) = subcommand.value.clone() {
                                    info!("Subcommand options: {:?}", subcommand_options);
                                    let id = subcommand_options
                                        .get(0)
                                        .expect("Expected game id")
                                        .value
                                        .as_str()
                                        .unwrap();
                                    let guild = command.guild_id.unwrap().get();
                                    let mut db = Database::open()
                                        .await
                                        .expect("Failed to connect to database");
                                    let result = db.remove_game(id, guild).await;
                                    match result {
                                        Ok(_) => format!("Game `{}` removed from monitoring list", id),
                                        Err(e) => {
                                            error!("Failed to remove game: {:?}", e);
                                            "Failed to remove game".to_string()
                                        }
                                    }
                                } else {
                                    "No subcommand options provided".to_string()
                                }
                            },
                            _ => "Unknown subcommand".to_string(),
                        }
                    } else {
                        "No subcommand provided".to_string()
                    }
                }
                _ => "not implemented :(".to_string(),
            };

            if command.data.name.as_str().ne("ai") {
                if let Err(why) = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(content),
                        ),
                    )
                    .await
                {
                    warn!("Cannot respond to slash command: {}", why);
                }
            } else {
                // Send deferred message whilst dealing with AI
                if let Err(why) = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
                    )
                    .await
                {
                    warn!("Cannot respond to slash command: {}", why);
                }

                let endpoint: String = command.data.options[0].value.as_str().unwrap().to_string();
                let prompt: String = command.data.options[1].value.as_str().unwrap().to_string();

                info!("Endpoint: {} | Prompt: {}", endpoint, prompt);

                if endpoint.eq("completions") {
                    info!("Prompt: {}", prompt);
                    let completions =
                        completions::build(models::CompletionModels::TEXT_DAVINCI_003)
                            .prompt(&prompt)
                            .max_tokens(50)
                            .user("MotorBot")
                            .complete()
                            .await;
                    match completions {
                        Ok(completions) => {
                            let message = completions.choices[0].text.as_str().to_string();
                            info!("Message: {}", message);
                            if let Err(why) = command
                                .create_followup(
                                    ctx.http,
                                    CreateInteractionResponseFollowup::new().content(message),
                                )
                                .await
                            {
                                warn!("Cannot respond to slash command: {}", why);
                            }
                        }
                        Err(why) => {
                            warn!("Error: {:?}", why);
                        }
                    }
                } else if endpoint.eq("images") {
                    let images = images::build()
                        .generate(prompt)
                        .size("256x256")
                        .user("MotorBot")
                        .done()
                        .await;
                    match images {
                        Ok(images) => {
                            let message = images.data[0].url.as_str().to_string();
                            info!("Message: {}", message);
                            if let Err(why) = command
                                .create_followup(
                                    ctx.http,
                                    CreateInteractionResponseFollowup::new().content(message),
                                )
                                .await
                            {
                                warn!("Cannot respond to slash command: {}", why);
                            }
                        }
                        Err(why) => {
                            warn!("Error: {:?}", why);
                        }
                    }
                } else {
                    if let Err(why) = command
                        .create_followup(
                            ctx.http,
                            CreateInteractionResponseFollowup::new().content("Unknown Endpoint"),
                        )
                        .await
                    {
                        warn!("Cannot respond to slash command: {}", why);
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let level = match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    info!("Starting MotorBot v{}", VERSION);
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
