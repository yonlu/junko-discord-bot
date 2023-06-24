mod commands;
mod utils;

use crate::commands::join::*;
use crate::commands::leave::*;
use crate::commands::mvp::*;
use crate::commands::ping::*;
use crate::commands::play::*;
use crate::commands::skip::*;
use crate::commands::tts::*;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client as RequestClient, StatusCode};
use songbird::SerenityInit;

// Import the `Context` to handle commands.
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
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
    model::{channel::Message, gateway::Ready, prelude::ChannelId},
    prelude::GatewayIntents,
};
use tokio::sync::Mutex;

type ConversationHistory = Vec<ChatMessage>;
lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<ChannelId, ConversationHistory>> =
        Mutex::new(HashMap::new());
}

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
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    //env::set_var("RUST_BACKTRACE", "1");
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
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

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Debug, Clone)]
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

    let api_key = env::var("OPENAI_API_KEY").expect("token");

    let junko_partial_prompt = "I want you to act as a linux terminal. I will type commands and you will reply with what the terminal should show. I want you to only reply with the terminal output inside one unique code block, and nothing else. do not write explanations. do not type commands unless I instruct you to do so. when i need to tell you something in english, i will do so by putting text inside curly brackets {like this}. my first command is pwd".to_owned();

    let full_prompt = [junko_partial_prompt, prompt.to_string()].join("\n");

    println!("Full prompt: {:?}", full_prompt);

    // Fetch the conversation history for this channel
    let mut conversations = CONVERSATIONS.lock().await;
    let channel_conversations = conversations.entry(msg.channel_id).or_insert(vec![]);

    // Create a user message and add it to the conversation history
    let user_message = ChatMessage {
        role: "user".to_string(),
        content: full_prompt.to_string(),
    };
    channel_conversations.push(user_message);

    let request_body = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: channel_conversations.clone(),
    };

    // Create request headers
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", api_key).parse().unwrap(),
    );
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let response = RequestClient::new()
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?
        .json::<ChatCompletionResponse>()
        .await?;

    println!("OpenAI response: {:?}", response);

    let result = &response.choices[0].message.content;

    println!("Answer: {:?}", result);

    utils::check_msg(msg.channel_id.say(&ctx.http, result).await);

    // Update the conversation history with the AI's response
    let ai_message = ChatMessage {
        role: "assistant".to_string(),
        content: result.to_string(),
    };
    channel_conversations.push(ai_message);

    // TODO make this into its own function
    if false {
        let speech_key = env::var("SPEECH_KEY").expect("token");
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
        r#"<speak version="1.0" xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="pt-BR"><voice xml:lang="pt-BR" xml:gender="Female" name="pt-BR-FranciscaNeural"><mstts:express-as type="cheerful">{}</mstts:express-as></voice></speak>"#,
        result
    ))
    .send()
    .await?;

        match response.status() {
            StatusCode::OK => {
                let mut output_file = File::create("output.mp3")?;
                println!("Output file created: output.mp3");
                let bytes = response.bytes().await?;
                println!("TTS response length: {}", bytes.len());
                output_file.write_all(&bytes)?;
                drop(output_file);

                let guild_id = msg.guild_id.unwrap();
                let manager = songbird::get(ctx)
                    .await
                    .expect("Songbird Voice client placed in initialisation.")
                    .clone();

                if let Some(handler_lock) = manager.get(guild_id) {
                    let mut handler = handler_lock.lock().await;

                    let source = songbird::ffmpeg("./output.mp3").await.unwrap();
                    println!("Playing output.mp3 in the voice channel");
                    handler.play_source(source);
                } else {
                    println!("No handler found for the guild");
                }
            }
            _ => {
                println!("Error synthesizing TTS: {:?}", response);
                utils::check_msg(
                    msg.channel_id
                        .say(&ctx.http, "Error synthesizing TTS")
                        .await,
                );
            }
        }
    }

    Ok(())
}
