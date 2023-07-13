mod commands;
mod utils;

use crate::commands::ask::*;
use crate::commands::join::*;
use crate::commands::leave::*;
use crate::commands::mvp::*;
use crate::commands::ping::*;
use crate::commands::play::*;
use crate::commands::skip::*;
use crate::commands::tts::*;

use std::env;

use songbird::Config;
use songbird::SerenityInit;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
};

use songbird::driver::DecodeMode;
use tracing_subscriber;

#[group]
#[commands(ping, join, leave, play, skip, list, ask, tts, mvp)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let songbird_config = Config::default()
        .decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| println!("Client ended: {:?}", why));
    });

    tokio::signal::ctrl_c()
        .await
        .map_err(|why| println!("Failed to handle Ctrl-C signal: {:?}", why))
        .ok();
    println!("Received Ctrl-C, shutting down.");
}

#[command]
#[only_in(guilds)]
async fn list(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let queue_str = queue.current_queue();

        println!("{:?}", queue_str);

        utils::check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Song List in queue printed to stdout, check your console."),
                )
                .await,
        );
    } else {
        utils::check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}
