use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client as RequestClient;
use serde::{Deserialize, Serialize};
use std::env;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Serialize, Debug)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<AnthropicMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct AnthropicResponse {
    pub content: Vec<ContentBlock>,
}

#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    pub text: String,
}

pub async fn send_message(
    client: &RequestClient,
    model: &str,
    system: Option<&str>,
    messages: &[AnthropicMessage],
    max_tokens: u32,
) -> Result<AnthropicResponse, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable is not set")?;

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);
    headers.insert(
        "anthropic-version",
        HeaderValue::from_static(ANTHROPIC_VERSION),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let request_body = AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        system: system.map(|s| s.to_string()),
        messages: messages.to_vec(),
    };

    let response = client
        .post(ANTHROPIC_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let anthropic_response = response.json::<AnthropicResponse>().await?;
        Ok(anthropic_response)
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        tracing::error!("Anthropic API error ({}): {}", status, error_body);
        Err(format!("Anthropic API returned status {}: {}", status, error_body).into())
    }
}
