use std::env;
use std::fs::File;
use std::io::Write;

use reqwest::{Client as RequestClient, StatusCode};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::utils::check_msg;

#[command]
pub async fn tts(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = match args.rest().is_empty() {
        true => {
            check_msg(msg.channel_id.say(&ctx.http, "No text provided").await);
            return Ok(());
        }
        false => args.rest(),
    };

    // TODO make this into its own function
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
            let manager = songbird::get(ctx)
                .await
                .expect("Songbird Voice client placed in initialisation.")
                .clone();

            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;

                match songbird::ffmpeg("./output.mp3").await {
                    Ok(source) => {
                        println!("Playing output.mp3 in the voice channel");
                        let track_handle = handler.play_source(source.into());
                        if let Err(e) = track_handle.play() {
                            println!("Error during playback: {:?}", e);
                        }
                    }
                    Err(e) => {
                        println!("Error playing output.mp3: {:?}", e);
                        check_msg(
                            msg.channel_id
                                .say(&ctx.http, "Error playing audio in the voice channel")
                                .await,
                        );
                    }
                }
            } else {
                println!("No handler found for the guild");
                check_msg(
                    msg.channel_id
                        .say(&ctx.http, "The bot is not connected to a voice channel")
                        .await,
                );
            }
        }
        _ => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Error synthesizing TTS")
                    .await,
            );
        }
    }

    Ok(())
}
