use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serenity::{
    client::Context,
    framework::standard::{
            macros::command,
            CommandResult,
        },
    model::channel::Message,
};

use crate::utils::check_msg;

#[command]
pub async fn mvp(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Ready to track MVPs!").await?;

    let response = reqwest::get("http://www.uropk.com.br/?module=mvp")
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
        let script_tag = document.select(&script_selector).next().unwrap();
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

    check_msg(msg.channel_id.say(&ctx.http, response_mvp_string).await);

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
