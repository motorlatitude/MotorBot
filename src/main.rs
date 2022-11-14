use std::convert::TryFrom;
use std::env;
use std::u64;
use std::time::Duration;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::{Result, Value};
use rand::Rng;
use dotenv::dotenv;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::GuildId;
use serenity::model::application::command::{CommandOptionType};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::application::command::Command;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

use clokwerk::{AsyncScheduler, TimeUnits, Job};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::gateway::Activity;
use serenity::model::channel::Reaction;
use serenity::model::prelude::ReactionType;
use serenity::model::user::OnlineStatus;
use serenity::utils::MessageBuilder;
use serenity::prelude::*;

mod db;
use crate::db::DBClient;

struct Handler;

#[derive(Deserialize, Debug)]
struct Response {
    body: Vec<Joke>,
    success: bool
}

#[derive(Deserialize, Debug)]
struct Joke {
    _id: serde_json::Value,
    punchline: serde_json::Value,
    setup: serde_json::Value,
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        info!("Message");
        // println!("{}: {}\nAttachments: {}", msg.author.name, msg.content, msg.attachments.len());
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                error!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!whoami" {
            let response = MessageBuilder::new()
                .push("```")
                .push(format!("id            : {:?}\n", msg.author.id.as_u64()))
                .push(format!("name          : {:?}\n", msg.author.name))
                .push(format!("discriminator : {:?}\n", msg.author.discriminator))
                .push(format!("avatar        : {:?}\n", msg.author.avatar.as_ref().unwrap()))
                .push(format!("flags         : {:?}\n", msg.author.public_flags.unwrap()))
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

        let channel_ids: Vec<u64> = vec![130734377066954752, 955479936871825509, 438307738250903553];

        if msg.attachments.len() > 0 || msg.content.contains("http") {
            if channel_ids.contains(msg.channel_id.as_u64()) && msg.author.id.as_u64() != &169554882674556930 {
                if let Err(why) = msg.react(&ctx, ReactionType::try_from("<:upvote:429449534389616641>").unwrap()).await {
                    error!("Failed to react to message {:?}", why);
                }
                if let Err(why) = msg.react(&ctx, ReactionType::try_from("<:downvote:429449638454493187>").unwrap()).await {
                    error!("Failed to react to message {:?}", why);
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Added");
        let channel_ids: Vec<u64> = vec![130734377066954752, 955479936871825509, 438307738250903553];

        if channel_ids.contains(reaction.channel_id.as_u64()) &&
            reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap() &&
            reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(*reaction.channel_id.as_u64(), *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was upvoted

            info!("User {:?} upvoted message from user: {:?}", user, message_user_id);

            // Update Database
            let db = DBClient::connect().await.expect("Failed to connect to database");
            let user_score = db.fetch_user_score(message_user_id.as_u64()).await.expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), 1).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score + 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        } else if channel_ids.contains(reaction.channel_id.as_u64()) &&
                    reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap() &&
                    reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(*reaction.channel_id.as_u64(), *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was downvote
            info!("User {:?} downvoted message from user: {:?}", user, message_user_id);

            // Update Database
            let db = DBClient::connect().await.expect("Failed to connect to database");
            let user_score = db.fetch_user_score(message_user_id.as_u64()).await.expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), 0).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Removed");
        let channel_ids: Vec<u64> = vec![130734377066954752, 955479936871825509, 438307738250903553];

        if channel_ids.contains(reaction.channel_id.as_u64()) &&
            reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap() &&
            reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(*reaction.channel_id.as_u64(), *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was the upvote was removed from

            info!("User {:?} upvoted message from user: {:?}", user, message_user_id);

            // Update Database
            let db = DBClient::connect().await.expect("Failed to connect to database");
            let user_score = db.fetch_user_score(message_user_id.as_u64()).await.expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), 0).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        } else if channel_ids.contains(reaction.channel_id.as_u64()) &&
                    reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap() &&
                    reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(*reaction.channel_id.as_u64(), *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was downvote was removed from
            info!("User {:?} downvoted message from user: {:?}", user, message_user_id);

            // Update Database
            let db = DBClient::connect().await.expect("Failed to connect to database");
            let user_score = db.fetch_user_score(message_user_id.as_u64()).await.expect("Failed to fetch user score");
            if user_score.is_none() {
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), 1).await {
                    error!("Failed to set user score {:?}", why);
                }
            } else {
                let uscore = user_score.unwrap();
                let new_score = uscore.score + 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        }
    }

    // Handle Slash Command Trigger
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command.data.name);

            let content = match command.data.name.as_str() {
                "ping" => "Pong!".to_string(),
                "time" => {
                    MessageBuilder::new()
                        .push(":alarm_clock: The current time for MotorBot is ")
                        .push(format!("{:?}", chrono::Utc::now()))
                        .build()
                },
                "roll" => {
                    let mut rng = rand::thread_rng();
                    let roll = rng.gen_range(1..100);
                    MessageBuilder::new()
                        .push("You rolled a ")
                        .push_bold_safe(roll)
                        .build()
                },
                "headsortails" => {
                    let mut rng = rand::thread_rng();
                    let roll = rng.gen_range(1..100);
                    if roll >= 50 {
                        MessageBuilder::new()
                            .push(":coin: Heads")
                            .build()
                    } else {
                        MessageBuilder::new()
                            .push(":coin: Tails")
                            .build()
                    }
                },
                "score" => {
                    let mut user_id = command.user.id.as_u64();
                    let mut username = command.user.tag();

                    let options = command
                        .data
                        .options
                        .get(0);
                    println!("Options: {:?}", options);
                    if !options.is_none() {
                        let option = options
                            .expect("Expected User Id")
                            .resolved
                            .as_ref()
                            .expect("Expected User Id");
                        if let CommandDataOptionValue::User(user, _member) = option {
                            user_id = user.id.as_u64();
                            username = user.tag();
                        }
                    }

                    let db = DBClient::connect().await.expect("Failed to connect to database");
                    let user_score = db.fetch_user_score(user_id).await.expect("Failed to fetch user score");
                    let mut score = 0;
                    if !user_score.is_none() {
                        let uscore = user_score.unwrap();
                        score = uscore.score;
                    }
                    format!("{}'s score is {}", username, score)
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    // Ready Event
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(Some(Activity::competing("Smooth Brain Marathon")), OnlineStatus::Online).await;

        let channel_id = ChannelId(432351112616738837);

        if let Err(why) = channel_id.say(&ctx.http, "I'm back!").await {
            error!("Error sending message: {:?}", why);
        }

        // Register Slash Commands

        let guild_id = GuildId(130734377066954752);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("time").description("Returns server time for MotorBot")
                })
                .create_application_command(|command| {
                    command.name("roll").description("Roll the dice, what will you get?")
                })
                .create_application_command(|command| {
                    command.name("headsortails").description("Heads or tails?")
                })
                .create_application_command(|command| {
                    command.name("score").description("Get a user's score").create_option(|option| {
                        option
                            .name("user")
                            .description("The user's score to look up")
                            .kind(CommandOptionType::User)
                            .required(false)
                    })
                })
        }).await;

        println!("I now have the following guild slash commands: {:#?}", commands);

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            command.name("ping").description("A simple ping command")
        })
        .await;

        let mut scheduler = AsyncScheduler::with_tz(chrono::Utc);
        // Add some tasks to it
        scheduler.every(5.minutes()).run(move || {
        //scheduler.every(1.day()).at("10:30 am").run(move || {
            let ctx = ctx.clone();
            async move {
                let channel_id = ChannelId(1040719087585742980);
                let client = reqwest::Client::new();

                let mut headers = HeaderMap::new();
                let rapid_api_key: String = env::var("RAPID_API_KEY").expect("Expected a token in the environment");
                headers.insert("X-RapidAPI-Key", HeaderValue::from_str(&rapid_api_key).unwrap());
                headers.insert("X-RapidAPI-Host", HeaderValue::from_str("dad-jokes.p.rapidapi.com").unwrap());

                let http_response = client
                .get("https://dad-jokes.p.rapidapi.com/random/joke")
                .headers(headers)
                .send()
                .await.expect("Failed to request joke")
                .json::<Response>()
                .await.expect("Failed to parse joke");
                println!("{:#?}", http_response.body);

                let _ = channel_id.say(&ctx.http, format!("{}\n\n||{}||", http_response.body[0].setup.as_str().unwrap(), http_response.body[0].punchline.as_str().unwrap())).await;
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
    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    info!("Starting MotorBot");
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
    Ok(())
}
