use serde::{Deserialize, Serialize};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use std::collections::HashMap;

use crate::utils::check_msg;

#[command]
pub async fn mvp(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Ready to track MVPs!").await?;

    let response =
        reqwest::get("https://www.uropk.com.br/?module=mvp&action=index&search=&status=1&submit=1")
            .await?
            .text()
            .await?;

    let mut map = HashMap::new();
    map.insert("1", "Balam (unholy)".to_string());
    map.insert("2", "Shax (unholy)".to_string());
    map.insert("3", "Raum (unholy)".to_string());
    map.insert("4", "Paimon (unholy)".to_string());
    map.insert("5", "Apollyon (unholy)".to_string());

    let mut mvp_timer_vec: Vec<Timer> = vec![];

    {
        let response_clone = response.clone();
        let document = scraper::Html::parse_document(&response_clone);

        let script_selector = scraper::Selector::parse(".table-responsive + script").unwrap();
        let script_tag = document.select(&script_selector).next().unwrap();
        let script_content = script_tag.text().collect::<String>();

        let js_object_start = script_content.find("{").unwrap();
        let js_object_end = script_content.rfind("}").unwrap() + 1;
        let js_object_str = &script_content[js_object_start..js_object_end];

        let json_object_str = js_object_str.replace("'", "\"");

        let parsed_object: Result<MVPCountdown, _> = serde_json::from_str(&json_object_str);

        let mut _timestamp_str = "0";

        let mut dead_mvp_ids_vec: Vec<String> = vec![];

        match parsed_object {
            Ok(mvp_countdown) => {
                println!("All good! Server time: {}", mvp_countdown.servertime);
                for timer in mvp_countdown.elements.iter() {
                    dead_mvp_ids_vec.push(timer.id.clone());
                    _timestamp_str = &timer.date.to_string();

                    mvp_timer_vec.push(Timer {
                        id: timer.id.clone(),
                        date: timer.date.clone(),
                    });
                }
                drop(mvp_countdown);
            }
            Err(_) => {
                println!("Something broke");
            }
        };

        println!("Dead mvp ids: {:?}", dead_mvp_ids_vec);
    }

    let mut mvps_close_respawn = String::from("MVPs respawn timers: \n");

    mvp_timer_vec.iter().for_each(|timer| {
        if map.contains_key(timer.id.as_str()) {
            println!("Timer id: {} \t date: {}", timer.id, timer.date);
            match map.get(timer.id.as_str()) {
                Some(mvp_timer) => {
                    mvps_close_respawn.push_str(mvp_timer);
                    mvps_close_respawn.push_str("\t");
                    mvps_close_respawn.push_str(&timer.date.to_string());
                    mvps_close_respawn.push_str("\n");
                }
                None => println!("Nothing here!"),
            }
        }
    });

    check_msg(msg.channel_id.say(&ctx.http, mvps_close_respawn).await);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct MVPCountdown {
    enable: bool,
    servertime: String,
    offset: i64,
    elements: Vec<Timer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timer {
    id: String,
    date: String,
}
