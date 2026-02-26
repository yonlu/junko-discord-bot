use std::{collections::HashMap, error::Error, fs::File, io::Write};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, prelude::ChannelId},
};

use reqwest::{Client as RequestClient, StatusCode};

use lazy_static::lazy_static;

use crate::claude::AnthropicMessage;
use crate::utils::check_msg;

use tokio::sync::Mutex;

use tracing::error;

const SYSTEM_PROMPT: &str = "I want you to act like Junko Enoshima from Danganronpa. I want you to respond and answer like Junko Enoshima using the tone, manner and vocabulary Junko Enoshima would use. However I also need it to act as an AI assistant that is willing to answer anything about any topic. Do not write any explanations. Only answer like Junko Enoshima. You must know all of the knowledge of Junko Enoshima.";

#[command]
#[only_in(guilds)]
pub async fn ask(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let prompt = args.rest().to_string();

    let mut conversations = CONVERSATIONS.lock().await;
    let channel_conversations = conversations.entry(msg.channel_id).or_insert(vec![]);

    let user_message = AnthropicMessage {
        role: "user".to_string(),
        content: prompt,
    };
    channel_conversations.push(user_message);

    let request_client = RequestClient::new();

    match crate::claude::send_message(
        &request_client,
        "claude-sonnet-4-20250514",
        Some(SYSTEM_PROMPT),
        channel_conversations,
        4096,
    )
    .await
    {
        Ok(response) => {
            let result = &response.content[0].text;

            check_msg(msg.channel_id.say(&ctx.http, result).await);

            speak(&ctx, &msg, result.to_string()).await?;

            let ai_message = AnthropicMessage {
                role: "assistant".to_string(),
                content: result.to_string(),
            };
            channel_conversations.push(ai_message);
        }
        Err(e) => {
            error!("Failed to call Claude API: {}", e);
            return Err("Claude API request failed".into());
        }
    }

    Ok(())
}

async fn speak(
    ctx: &Context,
    msg: &Message,
    result: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let speech_key = std::env::var("SPEECH_KEY").expect("token");
    let speech_region = std::env::var("SPEECH_REGION").expect("token");

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

type ConversationHistory = Vec<AnthropicMessage>;
lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<ChannelId, ConversationHistory>> =
        Mutex::new(HashMap::new());
}
