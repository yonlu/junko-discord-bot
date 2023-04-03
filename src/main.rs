mod utils;
use std::env;
use std::fs::File;
use std::sync::Arc;
use std::io::Write;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client as RequestClient, StatusCode};
use songbird::{
    input::restartable::Restartable,
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    SerenityInit,
    TrackEvent,
};

// Import the `Context` to handle commands.
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args,
            CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, prelude::ChannelId},
    prelude::GatewayIntents,
};
use serde::{Serialize, Deserialize};

#[group]
#[commands(ping, join, leave, play, skip, list, ask, tts)]
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
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    //env::set_var("RUST_BACKTRACE", "1");
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");
    
    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });

    tokio::signal::ctrl_c().await.map_err(|why| println!("Failed to handle Ctrl-C signal: {:?}", why)).ok();
    println!("Received Ctrl-C, shutting down.");
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            utils::check_msg(msg.reply(ctx, "Not in a voice channel").await);
            return Ok(())
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            utils::check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        utils::check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        utils::check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            utils::check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a URL to a video or audio")
                    .await,
            );

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        utils::check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await
        );

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;


    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                utils::check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            },
        };

        let song = handler.enqueue_source(source.into());
        let send_http = ctx.http.clone();
        let chan_id = msg.channel_id;

        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            SongEndNotifier {
                chan_id,
                http: send_http,
            },
        );

        utils::check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Added song to queue: position {}", handler.queue().len()),
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

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        utils::check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Song skipped: {} in queue.", queue.len()),
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

        let queue_str = queue
            .current_queue();

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

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>
}

#[derive(Serialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String, 
    usage: ChatCompletionUsage,
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[command]
#[only_in(guilds)]
async fn ask(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let prompt = args.rest();

    utils::check_msg(
        msg.channel_id
            .say(
                &ctx.http,
                format!("Command acknowledged! Prompt: {:?}", prompt),
            )
            .await,
    );

    let api_key = env::var("OPENAI_API_KEY").expect("token");

    let request_body = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }
        ],
    };

    // Create request headers
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", api_key).parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let response = RequestClient::new()
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?
        .json::<ChatCompletionResponse>().await?;

    let result = &response.choices[0].message.content;

    println!("Answer: {:?}", result);

    utils::check_msg(
        msg.channel_id
            .say(&ctx.http, result)
            .await,
    );

    // TODO make this into its own function
    let speech_key  = env::var("SPEECH_KEY").expect("token");
    let speech_region = env::var("SPEECH_REGION").expect("token");

    let url = format!(
        "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
        speech_region
    );

    let client = RequestClient::new();
    let response = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", speech_key)
        .header("Content-Type", "application/ssml+xml")
        .header(
            "X-Microsoft-OutputFormat",
            "audio-16khz-128kbitrate-mono-mp3",
        )
        .header("User-Agent", "reqwest")
        .body(format!(
            r#"<speak version="1.0" xml:lang="en-US"><voice xml:lang="en-US" xml:gender="Female" name="en-US-AshleyNeural"><prosody rate="1.00" pitch="+1%">{}</prosody></voice></speak>"#,
            result
        ))
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let mut output_file = File::create("output.mp3")?;
            let bytes = response.bytes().await?;
            output_file.write_all(&bytes)?;
            drop(output_file);

            let guild_id = msg.guild_id.unwrap();
            let manager = songbird::get(ctx).await
                .expect("Songbird Voice client placed in initialisation.").clone();

            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;

                let source = songbird::ffmpeg("./output.mp3").await.unwrap();
                handler.play_source(source);
            }
        }
        _ => {
            utils::check_msg(
                msg.channel_id
                    .say(&ctx.http, "Error synthesizing TTS")
                    .await,
            );
        }
    }

    Ok(())
}

#[command]
async fn tts(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = match args.rest().is_empty() {
        true => {
            utils::check_msg(msg.channel_id.say(&ctx.http, "No text provided").await);
            return Ok(());
        }
        false => args.rest(),
    };


    // TODO make this into its own function
    let speech_key  = env::var("SPEECH_KEY").expect("token");
    let speech_region = env::var("SPEECH_REGION").expect("token");

    let url = format!(
        "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
        speech_region
    );

    let client = RequestClient::new();
    let response = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", speech_key)
        .header("Content-Type", "application/ssml+xml")
        .header(
            "X-Microsoft-OutputFormat",
            "audio-16khz-128kbitrate-mono-mp3",
        )
        .header("User-Agent", "reqwest")
        .body(format!(
            r#"<speak version="1.0" xml:lang="en-US"><voice xml:lang="en-US" xml:gender="Female" name="en-US-AshleyNeural"><prosody rate="1.00" pitch="+1%">{}</prosody></voice></speak>"#,
            text
        ))
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let mut output_file = File::create("output.mp3")?;
            let bytes = response.bytes().await?;
            output_file.write_all(&bytes)?;
            drop(output_file);

            let guild_id = msg.guild_id.unwrap();
            let manager = songbird::get(ctx).await
                .expect("Songbird Voice client placed in initialisation.").clone();

            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;

                let source = songbird::ffmpeg("./output.mp3").await.unwrap();
                handler.play_source(source);
            }
        }
        _ => {
            utils::check_msg(
                msg.channel_id
                    .say(&ctx.http, "Error synthesizing TTS")
                    .await,
            );
        }
    }

    Ok(())
}


struct TrackEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            utils::check_msg(
                self.chan_id
                    .say(&self.http, &format!("Tracks ended: {}.", track_list.len()))
                    .await,
            );
        }

        None
    }
}

struct SongEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        utils::check_msg(
            self.chan_id
                .say(&self.http, "Song finished playing!")
                .await
        );

        None
    }
}
