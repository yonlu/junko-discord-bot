use reqwest::Client as RequestClient;
use serenity::http::Http;
use serenity::model::channel::Message;

use crate::claude::{self, AnthropicMessage};

const MODERATION_SYSTEM_PROMPT: &str = "You are a content moderation classifier. Your job is to determine if a message is about politics. Respond with exactly one word: YES if the message is about politics (political parties, elections, government policy, politicians, political ideologies, legislation, political debate), or NO if it is not about politics. Do not explain your reasoning.";

pub async fn moderate_message(
    http: &Http,
    msg: &Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = RequestClient::new();

    let messages = vec![AnthropicMessage {
        role: "user".to_string(),
        content: format!("Is this message about politics? Message: \"{}\"", msg.content),
    }];

    let response = claude::send_message(
        &client,
        "claude-haiku-4-5-20251001",
        Some(MODERATION_SYSTEM_PROMPT),
        &messages,
        10,
    )
    .await?;

    let answer = response.content[0].text.trim().to_uppercase();

    if answer.starts_with("YES") {
        msg.delete(&http).await?;
        tracing::info!(
            "Deleted political message from {} (ID: {}): {}",
            msg.author.name,
            msg.id,
            msg.content
        );
    }

    Ok(())
}
