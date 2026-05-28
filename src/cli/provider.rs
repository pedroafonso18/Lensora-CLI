use crate::cli::agent::AgentSpec;
use crate::cli::config::ProviderConfig;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ReviewRequest {
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Clone)]
pub enum ProviderClient {
    Anthropic(AnthropicClient),
    OpenAi(OpenAiClient),
}

impl ProviderClient {
    pub async fn review(&self, agent: &AgentSpec, request: &ReviewRequest) -> anyhow::Result<String> {
        match self {
            ProviderClient::Anthropic(provider) => provider.review(agent, request).await,
            ProviderClient::OpenAi(provider) => provider.review(agent, request).await,
        }
    }

    pub fn from_config(config: &ProviderConfig) -> anyhow::Result<Self> {
        let api_key = if let Some(api_key) = &config.api_key {
            api_key.clone()
        } else {
            let env_var = default_api_key_env(&config.name);
            std::env::var(env_var).map_err(|_| anyhow::anyhow!("missing API key for the selected provider"))?
        };

        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(120))
            .build()?;

        match config.name.to_lowercase().as_str() {
            "anthropic" => Ok(Self::Anthropic(AnthropicClient {
                client,
                api_key,
            })),
            "openai" => Ok(Self::OpenAi(OpenAiClient {
                client,
                api_key,
            })),
            other => anyhow::bail!("unsupported provider: {other}"),
        }
    }
}

#[derive(Clone)]
pub struct AnthropicClient {
    client: Client,
    api_key: String,
}

impl AnthropicClient {
    pub async fn review(&self, _agent: &AgentSpec, request: &ReviewRequest) -> anyhow::Result<String> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": request.model,
                "max_tokens": 2048,
                "system": request.system_prompt,
                "messages": [{"role": "user", "content": request.user_prompt}],
            }))
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    anyhow::anyhow!("Anthropic request timed out after 120 seconds")
                } else {
                    anyhow::Error::new(error)
                }
            })?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("Anthropic API error {status}: {body}");
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        extract_anthropic_text(&value)
    }
}

#[derive(Clone)]
pub struct OpenAiClient {
    client: Client,
    api_key: String,
}

impl OpenAiClient {
    pub async fn review(&self, _agent: &AgentSpec, request: &ReviewRequest) -> anyhow::Result<String> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&json!({
                "model": request.model,
                "temperature": 0,
                "messages": [
                    {"role": "system", "content": request.system_prompt},
                    {"role": "user", "content": request.user_prompt},
                ],
            }))
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    anyhow::anyhow!("OpenAI request timed out after 120 seconds")
                } else {
                    anyhow::Error::new(error)
                }
            })?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("OpenAI API error {status}: {body}");
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        extract_openai_text(&value)
    }
}

fn default_api_key_env(provider_name: &str) -> &'static str {
    match provider_name.to_lowercase().as_str() {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        _ => "ANTHROPIC_API_KEY",
    }
}

fn extract_anthropic_text(value: &serde_json::Value) -> anyhow::Result<String> {
    let content = value
        .get("content")
        .and_then(|content| content.as_array())
        .and_then(|items| items.first())
        .and_then(|item| item.get("text"))
        .and_then(|text| text.as_str())
        .unwrap_or_default();

    Ok(content.to_string())
}

fn extract_openai_text(value: &serde_json::Value) -> anyhow::Result<String> {
    let content = value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|items| items.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_default();

    Ok(content.to_string())
}
