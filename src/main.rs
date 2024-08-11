use dotenv::dotenv;
use rand::Rng;
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
use version_check::Version;

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

mod db;
use crate::db::DBClient;

mod plugins;
use crate::plugins::patches::patches_plugin;

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
        // println!("{}: {}\nAttachments: {}", msg.author.name, msg.content, msg.attachments.len());
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
            let db = DBClient::connect()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .fetch_user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(&message_user_id.get(), 1).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score + 1;
                if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
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
                .id; // The user id of the message that was downvote
            info!(
                "User {:?} downvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let db = DBClient::connect()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .fetch_user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(&message_user_id.get(), 0).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
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
            let db = DBClient::connect()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .fetch_user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(&message_user_id.get(), 0).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
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
                .id; // The user id of the message that was downvote was removed from
            info!(
                "User {:?} downvoted message from user: {:?}",
                user, message_user_id
            );

            // Update Database
            let db = DBClient::connect()
                .await
                .expect("Failed to connect to database");
            let user_score = db
                .fetch_user_score(&message_user_id.get())
                .await
                .expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(&message_user_id.get(), 1).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score + 1;
                if let Err(why) = db.set_user_score(&message_user_id.get(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        }
    }

    // Handle Slash Command Trigger
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {:#?}", command.data.name);

            let content = match command.data.name.as_str() {
                "ping" => "Pong!".to_string(),
                "time" => MessageBuilder::new()
                    .push(":alarm_clock: The current time for MotorBot is ")
                    .push(format!("{:?}", chrono::Local::now()))
                    .build(),
                "roll" => {
                    let mut rng = rand::thread_rng();
                    let roll = rng.gen_range(1..100);
                    MessageBuilder::new()
                        .push("You rolled a ")
                        .push_bold_safe(format!("{}", roll))
                        .build()
                }
                "headsortails" => {
                    let mut rng = rand::thread_rng();
                    let roll = rng.gen_range(1..100);
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
                    println!("Options: {:?}", options);
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

                    let db = DBClient::connect()
                        .await
                        .expect("Failed to connect to database");
                    let user_score = db
                        .fetch_user_score(&user_id.get())
                        .await
                        .expect("Failed to fetch user score");
                    let mut score = 0;
                    if !user_score.is_none() {
                        let uscore = user_score.unwrap();
                        score = uscore.score;
                    }
                    format!("{}'s score is {}", username, score)
                }
                "ai" => {
                    format!("Generating response...")
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
                    println!("Cannot respond to slash command: {}", why);
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
                    println!("Cannot respond to slash command: {}", why);
                }

                let endpoint: String = command.data.options[0].value.as_str().unwrap().to_string();
                let prompt: String = command.data.options[1].value.as_str().unwrap().to_string();

                println!("Endpoint: {} | Prompt: {}", endpoint, prompt);

                if endpoint.eq("completions") {
                    println!("Prompt: {}", prompt);
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
                            println!("Message: {}", message);
                            if let Err(why) = command
                                .create_followup(
                                    ctx.http,
                                    CreateInteractionResponseFollowup::new().content(message),
                                )
                                .await
                            {
                                println!("Cannot respond to slash command: {}", why);
                            }
                        }
                        Err(why) => {
                            println!("Error: {:?}", why);
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
                            println!("Message: {}", message);
                            if let Err(why) = command
                                .create_followup(
                                    ctx.http,
                                    CreateInteractionResponseFollowup::new().content(message),
                                )
                                .await
                            {
                                println!("Cannot respond to slash command: {}", why);
                            }
                        }
                        Err(why) => {
                            println!("Error: {:?}", why);
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
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
            }
        }
    }

    // Ready Event
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(Some(ActivityData::watching("you 👀")), OnlineStatus::Online);

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

        if let Err(why) = channel_id
            .say(&ctx.http, "MotorBot reporting for duty!")
            .await
        {
            error!("Error sending message: {:?}", why);
        }

        let loaded_patch_plugin = patches_plugin::PatchesPlugin::new(ctx.clone()).await;

        // Register Slash Commands

        let guild_id = GuildId::new(MotorBotGuilds::MotorBot as u64);

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
        ];

        let _commands = GuildId::set_commands(guild_id, &ctx.http, commands_list).await;

        //println!("I now have the following guild slash commands: {:#?}", commands);

        let _ = Command::create_global_command(
            &ctx.http,
            CreateCommand::new("ping").description("A simple ping command"),
        )
        .await;

        let mut scheduler = AsyncScheduler::with_tz(chrono::Utc);
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
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
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
    match Version::read() {
        Some(v) => {
            info!("Starting MotorBot v{} using rustc v{}", VERSION, v);
        }
        None => {
            warn!(
                "Starting MotorBot v{} using an unknown rustc version",
                VERSION
            );
        }
    }
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
