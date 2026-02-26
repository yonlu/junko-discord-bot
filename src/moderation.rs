use std::time::Duration;

use reqwest::Client as RequestClient;
use scraper::{Html, Selector};
use serenity::http::Http;
use serenity::model::channel::{Embed, Message};
use serenity::model::id::{ChannelId, MessageId};

use crate::claude::{self, AnthropicMessage};

const MODERATION_SYSTEM_PROMPT: &str = "You are a content moderation classifier. Your job is to determine if a message is about politics. Respond with exactly one word: YES if the message is about politics (political parties, elections, government policy, politicians, political ideologies, legislation, political debate), or NO if it is not about politics. Do not explain your reasoning.";

pub async fn moderate_message(
    http: &Http,
    msg: &Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = RequestClient::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Extract URLs and fetch their content
    let urls = extract_urls(&msg.content);
    let mut link_context = String::new();
    for url in &urls {
        if let Some(content) = fetch_url_content(&client, url).await {
            link_context.push_str(&content);
            link_context.push('\n');
        }
    }

    let mut prompt = format!("Is this message about politics? Message: \"{}\"", msg.content);
    if !link_context.is_empty() {
        prompt.push_str(&format!(
            "\n\nLinked page content:\n{}",
            link_context.trim()
        ));
    }

    let messages = vec![AnthropicMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = claude::send_message(
        &client,
        "claude-haiku-4-5-20251001",
        Some(MODERATION_SYSTEM_PROMPT),
        &messages,
        20,
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

/// Moderate a message based on its embed content (called when Discord adds embeds via message_update)
pub async fn moderate_embeds(
    http: &Http,
    channel_id: ChannelId,
    message_id: MessageId,
    embeds: &[Embed],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let embed_text = extract_embed_text(embeds);
    if embed_text.is_empty() {
        return Ok(());
    }

    let client = RequestClient::new();

    let prompt = format!(
        "Is this content about politics? Content from a linked page embed:\n{}",
        embed_text
    );

    let messages = vec![AnthropicMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = claude::send_message(
        &client,
        "claude-haiku-4-5-20251001",
        Some(MODERATION_SYSTEM_PROMPT),
        &messages,
        20,
    )
    .await?;

    let answer = response.content[0].text.trim().to_uppercase();

    if answer.starts_with("YES") {
        channel_id.delete_message(&http, message_id).await?;
        tracing::info!(
            "Deleted message with political embed (ID: {}): {}",
            message_id,
            embed_text
        );
    }

    Ok(())
}

fn extract_embed_text(embeds: &[Embed]) -> String {
    let mut parts = Vec::new();
    for embed in embeds {
        if let Some(title) = &embed.title {
            if !title.trim().is_empty() {
                parts.push(format!("Title: {}", title.trim()));
            }
        }
        if let Some(desc) = &embed.description {
            if !desc.trim().is_empty() {
                parts.push(format!("Description: {}", desc.trim()));
            }
        }
        if let Some(author) = &embed.author {
            if !author.name.trim().is_empty() {
                parts.push(format!("Author: {}", author.name.trim()));
            }
        }
    }
    parts.join("\n")
}

fn extract_urls(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter(|word| word.starts_with("http://") || word.starts_with("https://"))
        .collect()
}

async fn fetch_url_content(client: &RequestClient, url: &str) -> Option<String> {
    // Use Instagram GraphQL API for Instagram URLs
    if let Some(shortcode) = extract_instagram_shortcode(url) {
        return fetch_instagram_caption(client, &shortcode).await;
    }

    let response = client.get(url).send().await.ok()?;
    let html_text = response.text().await.ok()?;
    let document = Html::parse_document(&html_text);

    let mut parts = Vec::new();

    // <title>
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = document.select(&sel).next() {
            let title = el.text().collect::<String>();
            if !title.trim().is_empty() {
                parts.push(format!("Title: {}", title.trim()));
            }
        }
    }

    // og:title
    if let Ok(sel) = Selector::parse(r#"meta[property="og:title"]"#) {
        if let Some(el) = document.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                if !content.trim().is_empty() {
                    parts.push(format!("OG Title: {}", content.trim()));
                }
            }
        }
    }

    // og:description
    if let Ok(sel) = Selector::parse(r#"meta[property="og:description"]"#) {
        if let Some(el) = document.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                if !content.trim().is_empty() {
                    parts.push(format!("OG Description: {}", content.trim()));
                }
            }
        }
    }

    // meta description
    if let Ok(sel) = Selector::parse(r#"meta[name="description"]"#) {
        if let Some(el) = document.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                if !content.trim().is_empty() {
                    parts.push(format!("Description: {}", content.trim()));
                }
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Extract the shortcode from an Instagram URL (e.g., "DVNJIa2ja2F" from ".../p/DVNJIa2ja2F/...")
fn extract_instagram_shortcode(url: &str) -> Option<String> {
    if !url.contains("instagram.com") {
        return None;
    }
    // Match /p/, /reel/, /reels/ followed by the shortcode
    let patterns = ["/p/", "/reel/", "/reels/"];
    for pattern in patterns {
        if let Some(idx) = url.find(pattern) {
            let after = &url[idx + pattern.len()..];
            let shortcode: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !shortcode.is_empty() {
                return Some(shortcode);
            }
        }
    }
    None
}

/// Fetch an Instagram post's caption using Instagram's internal GraphQL API
async fn fetch_instagram_caption(client: &RequestClient, shortcode: &str) -> Option<String> {
    let variables = serde_json::json!({ "shortcode": shortcode }).to_string();

    let response = client
        .post("https://www.instagram.com/api/graphql")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("X-IG-App-ID", "936619743392459")
        .header("X-FB-LSD", "AVqbxe3J_YA")
        .header("X-ASBD-ID", "129477")
        .header("Sec-Fetch-Site", "same-origin")
        .form(&[
            ("variables", variables.as_str()),
            ("doc_id", "10015901848480474"),
            ("lsd", "AVqbxe3J_YA"),
        ])
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = response.json().await.ok()?;

    let media = json
        .get("data")?
        .get("xdt_shortcode_media")?;

    let mut parts = Vec::new();

    // Caption text
    if let Some(caption) = media
        .get("edge_media_to_caption")
        .and_then(|c| c.get("edges"))
        .and_then(|e| e.get(0))
        .and_then(|e| e.get("node"))
        .and_then(|n| n.get("text"))
        .and_then(|t| t.as_str())
    {
        if !caption.trim().is_empty() {
            parts.push(format!("Instagram Caption: {}", caption.trim()));
        }
    }

    // Owner username
    if let Some(owner) = media
        .get("owner")
        .and_then(|o| o.get("username"))
        .and_then(|u| u.as_str())
    {
        parts.push(format!("Instagram Author: {}", owner));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}
