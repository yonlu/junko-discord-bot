use std::sync::Arc;

use serenity::{
    async_trait,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    http::Http,
    model::{channel::Message, prelude::ChannelId},
};

use songbird::{
    input::YoutubeDl, Event, EventContext, EventHandler as VoiceEventHandler,
    TrackEvent,
};

use crate::utils::check_msg;

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a URL to a video or audio")
                    .await,
            );

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await,
        );

        return Ok(());
    }

    let guild_id = msg.guild(&ctx.cache).unwrap().id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        println!("Attempting to create source for URL: {}", url);
        let http_client = reqwest::Client::new();
        let source = YoutubeDl::new(http_client, url);

        let song = handler.enqueue_input(source.into()).await;
        let track_state = song.get_info().await;
        println!("Track state after enqueue: {:?}", track_state);
        let send_http = ctx.http.clone();
        let chan_id = msg.channel_id;

        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            SongEndNotifier {
                chan_id,
                http: send_http.clone(),
            },
        );

        let _ = song.add_event(
            Event::Track(TrackEvent::Play),
            SongPlayNotifier,
        );


        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Added song to queue: position {}", handler.queue().len()),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

struct SongPlayNotifier;

#[async_trait]
impl VoiceEventHandler for SongPlayNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        println!("[DEBUG] Track started playing!");
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
        check_msg(self.chan_id.say(&self.http, "Song finished playing!").await);

        None
    }
}
