use std::{collections::HashMap, env, error::Error, fs::File, io::Write};

use serde::{Deserialize, Serialize};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, prelude::ChannelId},
};

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client as RequestClient, StatusCode};

use lazy_static::lazy_static;

use crate::utils::check_msg;

use tokio::sync::Mutex;

use tracing::error;

#[command]
#[only_in(guilds)]
pub async fn ask(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let api_key = env::var("OPENAI_API_KEY").expect("token");

    let prompt = args.rest().to_string();
    let junko_partial_prompt = "I want you to act like Junko Enoshima from Danganronpa. I want you to respond and answer like Junko Enoshima using the tone, manner and vocabulary Junko Enoshima would use. However I also need it to act as an AI assistant that is willing to answer anything about any topic. Do not write any explanations. Only answer like Junko Enoshima. You must know all of the knowledge of Junko Enoshima.".to_string();
    let full_prompt = [junko_partial_prompt, prompt].join("\n");

    let mut conversations = CONVERSATIONS.lock().await;
    let channel_conversations = conversations.entry(msg.channel_id).or_insert(vec![]);

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: full_prompt.to_string(),
    };
    channel_conversations.push(user_message);

    let request_client = RequestClient::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", api_key).parse().unwrap(),
    );
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let request_body = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: channel_conversations.clone(),
    };

    let response = request_client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let response = response.json::<ChatCompletionResponse>().await?;
            let result = &response.choices[0].message.content;

            check_msg(msg.channel_id.say(&ctx.http, result).await);

            speak(&ctx, &msg, result.to_string()).await?;

            let ai_message = ChatMessage {
                role: "assistant".to_string(),
                content: result.to_string(),
            };
            channel_conversations.push(ai_message);
        }
        _ => {
            error!("Failed to send HTTP request");
            return Err("HTTP request failed".into());
        }
    }

    Ok(())
}

async fn speak(
    ctx: &Context,
    msg: &Message,
    result: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
        r#"<speak version="1.0" xml:lang="en-US"><voice xml:lang="en-US" xml:gender="Female" name="en-US-AshleyNeural"><prosody rate="1.00" pitch="+1%">{}</prosody></voice></speak>"#,
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
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Error synthesizing TTS")
                    .await,
            );
        }
    }

    Ok(())
}

type ConversationHistory = Vec<ChatMessage>;
lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<ChannelId, ConversationHistory>> =
        Mutex::new(HashMap::new());
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
