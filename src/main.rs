use std::convert::TryFrom;
use std::env;
use std::u64;
use dotenv::dotenv;
use serenity::model::prelude::Channel;
use serenity::model::prelude::ChannelId;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        info!("Message");
        // println!("{}: {}\nAttachments: {}", msg.author.name, msg.content, msg.attachments.len());
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                error!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!time" {
            let response = MessageBuilder::new()
                .push("The time is ")
                .push(format!("{:?}", chrono::Utc::now()))
                .build();
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
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
        } else if msg.content == "!score" {
            let db = DBClient::connect().await.expect("Failed to connect to database");
            let user_score = db.fetch_user_score(msg.author.id.as_u64()).await.expect("Failed to fetch user score");
            let mut score = 0;
            if !user_score.is_none() {
                let uscore = user_score.unwrap();
                score = uscore.score;
            }
            let response = MessageBuilder::new()
                .push("Your score is ")
                .push(format!("{:?}", score))
                .build();

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {:?}", why);
            }
        }

        let channel_id: u64 = 130734377066954752;
        if msg.attachments.len() > 0 || msg.content.contains("http") {
            if msg.channel_id.as_u64() == &channel_id && msg.author.id.as_u64() != &169554882674556930 {
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
        let channel_id: u64 = 130734377066954752;

        if reaction.channel_id.as_u64() == &channel_id &&
            reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap() &&
            reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(channel_id, *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was upvoted

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
        } else if reaction.channel_id.as_u64() == &channel_id &&
                    reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap() &&
                    reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(channel_id, *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was downvote
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
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Removed");
        let channel_id: u64 = 130734377066954752;

        if reaction.channel_id.as_u64() == &channel_id &&
            reaction.emoji == ReactionType::try_from("<:upvote:429449534389616641>").unwrap() &&
            reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(channel_id, *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was the upvote was removed from

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
                let new_score = uscore.score - 1;
                if let Err(why) = db.set_user_score(message_user_id.as_u64(), new_score).await {
                    error!("Failed to set user score {:?}", why);
                }
            }
        } else if reaction.channel_id.as_u64() == &channel_id &&
                    reaction.emoji == ReactionType::try_from("<:downvote:429449638454493187>").unwrap() &&
                    reaction.user_id.unwrap().as_u64() != &169554882674556930 {
            let user = reaction.user_id.unwrap(); // The user id of who upvoted
            let message_id = reaction.message_id; // The message id of the message that was upvoted
            let message_user_id = ctx.http.get_message(channel_id, *message_id.as_u64()).await.unwrap().author.id; // The user id of the message that was downvote was removed from
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

    // Ready Event
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(Some(Activity::competing("Smooth Brain Marathon")), OnlineStatus::Online).await;

        let channel_id = ChannelId(432351112616738837);

        let _ = channel_id.send_message(&ctx.http, |m| {
            m.content("test")
                .embed(|e| e.title("I'm back!").description(format!("{:?}", chrono::Utc::now())))
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
