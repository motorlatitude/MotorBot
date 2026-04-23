use dotenv::dotenv;

use serenity::all::{ActivityData, Interaction};
use std::{env, vec};
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

mod error;

pub use self::error::{Error, Result};

mod plugins;
mod storage;

use crate::plugin::plugin_context::{PluginEvent, PluginEventLevel};
use crate::plugin::PluginContext;
use crate::plugins::jokes::JokesPlugin;
use crate::plugins::patches::PatchesPlugin;

use crate::plugins::coin_toss::CoinTossPlugin;
use crate::plugins::debug::DebugPlugin;
use crate::plugins::dice::DicePlugin;
use crate::plugins::ping::PingPlugin;
use crate::plugins::points::PointsPlugin;
use crate::plugins::time::TimePlugin;

mod plugin;
use plugin::MotorbotPlugin;
struct Handler {
    plugins: Vec<Box<dyn MotorbotPlugin + Send + Sync>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        debug!("Message");

        let plugin_ctx = PluginContext::new(ctx.clone(), &self.plugins).await;
        for plugin in self.plugins.iter() {
            match plugin.on_message(&plugin_ctx, &msg).await {
                Ok(()) => {}
                Err(e) => error!(
                    "[{}] Error occurred while handling message: {}",
                    &plugin.info().name,
                    e
                ),
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
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Added");
        let plugin_ctx = PluginContext::new(ctx.clone(), &self.plugins).await;
        for plugin in self.plugins.iter() {
            match plugin.on_reaction_add(&plugin_ctx, &reaction).await {
                Ok(()) => {}
                Err(e) => error!(
                    "[{}] Error occurred while handling reaction add: {}",
                    &plugin.info().name,
                    e
                ),
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        info!("Reaction Removed");
        let plugin_ctx = PluginContext::new(ctx.clone(), &self.plugins).await;
        for plugin in self.plugins.iter() {
            match plugin.on_reaction_remove(&plugin_ctx, &reaction).await {
                Ok(()) => {}
                Err(e) => error!(
                    "[{}] Error occurred while handling reaction remove: {}",
                    &plugin.info().name,
                    e
                ),
            }
        }
    }

    // Ready Event
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(
            Some(ActivityData::custom("😶‍🌫️")),
            OnlineStatus::Online,
        );

        let plugin_ctx = PluginContext::new(ctx.clone(), &self.plugins).await;
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        plugin_ctx
            .send_event(
                PluginEvent::new(
                    PluginEventLevel::Info,
                    "Bot Started",
                ).description(&format!(
                    "<:lightning:1494777585874370600> MotorBot v{} reporting for duty!",
                    VERSION
                )),
            )
            .await
            .unwrap_or_else(|err| {
                error!("Failed to send startup event: {:?}", err);
            });
        for plugin in self.plugins.iter() {
            match plugin.on_ready(&plugin_ctx).await {
                Ok(()) => {}
                Err(e) => error!(
                    "[{}] Error occurred while handling ready event: {}",
                    &plugin.info().name,
                    e
                ),
            }
        }
    }

    // Handle Slash Command Trigger
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let plugin_ctx = PluginContext::new(ctx.clone(), &self.plugins).await;
        let plugin_interaction = interaction.clone();
        for plugin in self.plugins.iter() {
            match plugin
                .on_interaction_create(&plugin_ctx, &plugin_interaction)
                .await
            {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        "[{}] Error occurred while handling interaction create: {}",
                        &plugin.info().name,
                        e
                    );
                    plugin_ctx
                        .send_event(
                            PluginEvent::new(
                                PluginEventLevel::Error,
                                "Internal Error",
                            )
                            .description(&format!(
                                "*{}*\n{}",
                                plugin.info().name,
                                e
                            )),
                        )
                        .await
                        .unwrap_or_else(|err| {
                            error!("Failed to send error event: {:?}", err);
                        });
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

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let plugins: Vec<Box<dyn MotorbotPlugin + Send + Sync>> = vec![
        Box::new(PingPlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(TimePlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(DicePlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(CoinTossPlugin::new())
            as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(DebugPlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(PointsPlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(JokesPlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
        Box::new(PatchesPlugin::new()) as Box<dyn MotorbotPlugin + Send + Sync>,
    ];

    let handler = Handler { plugins };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    info!("Starting MotorBot v{}", VERSION);
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
