mod utils;
mod commands;

use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex as SyncMutex};
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
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use scraper;
use crate::commands::tts::*;

type ConversationHistory = Vec<ChatMessage>;
lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<ChannelId, ConversationHistory>> = Mutex::new(HashMap::new());
}

fn initialize_map() -> HashMap<String, i32> {
    let mut map = HashMap::new();
    map.insert("Lord of the Dead".to_string(), 1);
    map.insert("Fallen Bishop Hibram".to_string(), 2);
    map.insert("Detardeurus".to_string(), 3);
    map.insert("Samurai Specter".to_string(), 4);
    map.insert("Maya".to_string(), 5);
    map.insert("Lady Tanee".to_string(), 6);
    map.insert("Tao Gunka".to_string(), 7);
    map.insert("RSX-0806".to_string(), 8);
    map.insert("Dracula".to_string(), 9);
    map.insert("Doppelganger".to_string(), 10);
    map.insert("Dark Lord".to_string(), 11);
    map.insert("Evil Snake Lord".to_string(), 16);
    map.insert("Pharaoh".to_string(), 17);
    map.insert("Vesper".to_string(), 18);
    map.insert("Kiel D-01".to_string(), 19);
    map.insert("Egnigem Cenia".to_string(), 20);
    map.insert("White Lady".to_string(), 21);
    map.insert("Osiris".to_string(), 22);
    map.insert("Amon Ra".to_string(), 23);
    map.insert("Gopinich".to_string(), 24);
    map.insert("Valkyrie Randgris".to_string(), 25);
    map.insert("Moonlight Flower".to_string(), 26);
    map.insert("Baphomet".to_string(), 27);
    map.insert("Golden Thief Bug".to_string(), 28);
    map.insert("Gloom Under Night".to_string(), 29);
    map.insert("Ifrit".to_string(), 30);
    map.insert("Drake".to_string(), 31);
    map.insert("Turtle General".to_string(), 32);
    map.insert("Stormy Knight".to_string(), 33);
    map.insert("Orc Lord".to_string(), 35);
    map.insert("Orc Hero".to_string(), 36);
    map.insert("Hatii".to_string(), 37);
    map.insert("Mistress".to_string(), 38);
    map.insert("Phreeoni".to_string(), 39);
    map.insert("Wounded Morocc".to_string(), 40);
    map.insert("Eddga".to_string(), 41);
    map.insert("Atroce".to_string(), 44);
    map.insert("Balam".to_string(), 47);
    map.insert("Shax".to_string(), 48);
    map.insert("Raum".to_string(), 49);
    map.insert("Paimon".to_string(), 50);
    map.insert("Apollyon".to_string(), 51);
    map
}

lazy_static! {
    static ref MVP_TO_ID: SyncMutex<HashMap<String, i32>> = SyncMutex::new(initialize_map());
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
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_VOICE_STATES;

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

#[derive(Serialize, Deserialize, Debug)]
struct Timer {
    id: String,
    date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MVPCountdown {
    enable: bool,
    servertime: String,
    offset: i64,
    elements: Vec<Timer>
}

#[derive(Serialize, Deserialize, Debug)]
struct MVP {
    name: String,
    map: String,
}

#[command]
async fn mvp(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Ready to track MVPs!").await?;

    let response = reqwest::get(
        "http://www.uropk.com.br/?module=mvp",
    )
    .await?
    .text()
    .await?;


    let mut map = HashMap::new();
    map.insert("1", "Lord of the Dead (gef_dun03)".to_string());
    map.insert("2", "Fallen Bishop Hibram (abbey02)".to_string());
    map.insert("3", "Detardeurus (abyss_03)".to_string());
    map.insert("4", "Samurai Specter (ama_dun03)".to_string());
    map.insert("5", "Maya (anthell02)".to_string());
    map.insert("6", "Lady Tanee (ayo_dun02)".to_string());
    map.insert("7", "Tao Gunka (beach_dun)".to_string());
    map.insert("8", "RSX-0806 (ein_dun02)".to_string());
    map.insert("9", "Dracula (gef_dun01)".to_string());

    map.insert("10", "Doppelganger (gef_dun02)".to_string());
    map.insert("11", "Dark Lord (gl_chyard)".to_string());
    map.insert("12", "Eddga (gld_dun01)".to_string());
    map.insert("13", "Doppelganger (gld_dun02)".to_string());
    map.insert("14", "Maya (gld_dun03)".to_string());
    map.insert("15", "Dark Lord (gld_dun04)".to_string());
    map.insert("16", "Evil Snake Lord (gon_dun03)".to_string());
    map.insert("17", "Pharaoh (in_sphinx5)".to_string());
    map.insert("18", "Vesper (jupe_core)".to_string());
    map.insert("19", "Kiel D-01 (kh_dun02)".to_string());

    map.insert("20", "Egnigem Cenia (lhz_dun02)".to_string());
    map.insert("21", "White Lady (lou_dun03)".to_string());
    map.insert("22", "Osiris (moc_pryd04)".to_string());
    map.insert("23", "Amon Ra (moc_pryd06)".to_string());
    map.insert("24", "Gopinich (mosk_dun03)".to_string());
    map.insert("25", "Valkyrie Randgris (odin_tem03)".to_string());
    map.insert("26", "Moonlight Flower (pay_dun04)".to_string());
    map.insert("27", "Baphomet (prt_maze03)".to_string());
    map.insert("28", "Golden Thief Bug (prt_sewb4)".to_string());
    map.insert("29", "Gloom Under Night (ra_san05)".to_string());

    map.insert("30", "Ifrit (thor_v03)".to_string());
    map.insert("31", "Drake (treasure02)".to_string());
    map.insert("32", "Turtle General (tur_dun04)".to_string());
    map.insert("33", "Stormy Knight (xmas_dun02)".to_string());
    map.insert("34", "Orc Hero (gef_fild02)".to_string());
    map.insert("35", "Orc Lord (gef_fild10)".to_string());
    map.insert("36", "Orc Hero (gef_fild14)".to_string());
    map.insert("37", "Hatii (xmas_fild01)".to_string());
    map.insert("38", "Mistress (mjolnir_04)".to_string());
    map.insert("39", "Phreeoni (moc_fild17)".to_string());

    map.insert("40", "Wounded Morocc (moc_fild22)".to_string());
    map.insert("41", "Eddga (pay_fild11)".to_string());
    map.insert("42", "Atroce (ra_fild02)".to_string());
    map.insert("43", "Atroce (ra_fild03)".to_string());
    map.insert("44", "Atroce (ra_fild04)".to_string());
    map.insert("45", "Atroce (ve_fild01)".to_string());
    map.insert("47", "Balam (unholy)".to_string());
    map.insert("48", "Shax (unholy)".to_string());
    map.insert("49", "Raum (unholy)".to_string());

    map.insert("50", "Paimon (unholy)".to_string());
    map.insert("51", "Apollyon (unholy)".to_string());

    let mut response_mvp_string: String = Default::default();
    let _mvp_timer_vec: Vec<&Timer> = vec![];

    {
        let response_clone = response.clone();
        let document = scraper::Html::parse_document(&response_clone);

        let script_selector = scraper::Selector::parse(".table-responsive + script").unwrap();
        let script_tag = document
            .select(&script_selector)
            .next()
            .unwrap();
        let script_content = script_tag.text().collect::<String>();

        let js_object_start = script_content.find("{").unwrap();
        let js_object_end = script_content.rfind("}").unwrap() + 1;
        let js_object_str = &script_content[js_object_start..js_object_end];

        let json_object_str = js_object_str.replace("'", "\"");

        let parsed_object: Result<MVPCountdown, _> = serde_json::from_str(&json_object_str);
        println!("Parsed object content: {:?}", parsed_object);

        let mvp_string = parsed_object.unwrap().elements;

        mvp_string.iter().for_each(|x| {
            response_mvp_string.push_str(map.get(&x.id as &str).unwrap_or(&"unknown".to_string()));
            response_mvp_string.push_str("\t");
            response_mvp_string.push_str(&x.date);
            response_mvp_string.push_str("\n");
        });

        // response_mvp_string.push_str("Alive MVPs:");

        for (id, name) in &map {
            
          println!("id: {} \t name {} ", id, name);
        }
    }

    utils::check_msg(msg.channel_id.say(&ctx.http, response_mvp_string).await);

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

    let junko_partial_prompt = "Without repeating the same question, respond in less than 70 characters as Junko Enoshima from Danganronpa series, and do not mention 'OpenAI', 'ChatGPT' or 'AI language model', do not use quotations, and answer in Brazilian Portuguese: ".to_owned();

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
    headers.insert(AUTHORIZATION, format!("Bearer {}", api_key).parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let response = RequestClient::new()
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?
        .json::<ChatCompletionResponse>().await?;

    println!("OpenAI response: {:?}", response);

    let result = &response.choices[0].message.content;

    println!("Answer: {:?}", result);

    utils::check_msg(
        msg.channel_id
            .say(&ctx.http, result)
            .await,
    );

    // Update the conversation history with the AI's response
    let ai_message = ChatMessage {
        role: "assistant".to_string(),
        content: result.to_string(),
    };
    channel_conversations.push(ai_message);

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
            let manager = songbird::get(ctx).await
                .expect("Songbird Voice client placed in initialisation.").clone();

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
