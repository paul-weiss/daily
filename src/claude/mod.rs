use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

pub struct ClaudeClient {
    api_key: String,
    client: Client,
}

impl ClaudeClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?;

        Ok(Self {
            api_key,
            client: Client::new(),
        })
    }

    pub async fn chat(&self, prompt: &str, context: Option<&str>) -> Result<String> {
        let mut messages = Vec::new();

        // Add context if provided
        if let Some(ctx) = context {
            messages.push(Message {
                role: "user".to_string(),
                content: format!("Context: {}\n\nUser question: {}", ctx, prompt),
            });
        } else {
            messages.push(Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            });
        }

        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let claude_response: ClaudeResponse = response.json().await?;

        Ok(claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }

    pub fn build_task_context(&self, tasks: &[crate::models::Task]) -> String {
        let mut context = String::from("Current tasks:\n");
        for task in tasks {
            context.push_str(&format!(
                "- [{}] {} (Priority: {}, Category: {})\n",
                if task.completed { "x" } else { " " },
                task.title,
                task.priority.to_string(),
                task.category
            ));
        }
        context
    }
}
