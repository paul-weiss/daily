use anyhow::{Context, Result};
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
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

// Structured output from the plan command
#[derive(Debug, Deserialize)]
pub struct HabitPlan {
    pub explanation: String,
    pub actions: Vec<PlanAction>,
}

#[derive(Debug, Deserialize)]
pub struct PlanAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub title: String,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub description: Option<String>,
    pub scheduled_days: Option<Vec<String>>,  // ["mon", "wed", "fri"]
    pub scheduled_time: Option<String>,        // "HH:MM"
    pub location: Option<String>,
    pub two_minute: Option<bool>,
    pub habit_stack_after: Option<String>,     // task ID
}

impl PlanAction {
    /// Convert day abbreviations to weekday numbers (0=Mon..6=Sun).
    pub fn scheduled_days_as_nums(&self) -> Option<Vec<u8>> {
        self.scheduled_days.as_ref().map(|days| {
            days.iter()
                .filter_map(|d| day_str_to_num(d))
                .collect()
        })
    }
}

pub fn day_str_to_num(s: &str) -> Option<u8> {
    match s.to_lowercase().as_str() {
        "mon" | "monday" => Some(0),
        "tue" | "tuesday" => Some(1),
        "wed" | "wednesday" => Some(2),
        "thu" | "thursday" => Some(3),
        "fri" | "friday" => Some(4),
        "sat" | "saturday" => Some(5),
        "sun" | "sunday" => Some(6),
        _ => None,
    }
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
        let content = match context {
            Some(ctx) => format!("Context:\n{}\n\nRequest: {}", ctx, prompt),
            None => prompt.to_string(),
        };

        let request = ClaudeRequest {
            model: "claude-opus-4-6".to_string(),
            max_tokens: 1024,
            system: "You are a helpful task management assistant.".to_string(),
            messages: vec![Message { role: "user".to_string(), content }],
        };

        self.send_request(request).await
    }

    pub async fn plan_from_natural_language(
        &self,
        prompt: &str,
        tasks: &[crate::models::Task],
        today: NaiveDate,
    ) -> Result<HabitPlan> {
        let task_context = self.build_task_context(tasks);

        let system = format!(
            r#"You are a habit scheduling assistant for the Daily CLI tool.
Today's date is {today}.

Existing tasks:
{task_context}

The user will describe habits or tasks in natural language. Your job is to interpret their intent
and respond ONLY with valid JSON (no other text) in exactly this format:

{{
  "explanation": "Brief summary of what you are creating",
  "actions": [
    {{
      "type": "create_habit",
      "title": "Habit name",
      "category": "category name or null",
      "priority": "low | medium | high | critical",
      "description": "optional description or null",
      "scheduled_days": ["mon","tue","wed","thu","fri","sat","sun"] or null,
      "scheduled_time": "HH:MM" or null,
      "location": "place/context or null",
      "two_minute": false,
      "habit_stack_after": "task-id or null"
    }}
  ]
}}

Rules:
- Use "scheduled_days" only when the user names specific days. Use null to mean every day.
- Day values: "mon", "tue", "wed", "thu", "fri", "sat", "sun"
- If the user says "weekdays", use ["mon","tue","wed","thu","fri"].
- If the user says "weekends", use ["sat","sun"].
- Extract the time in 24-hour HH:MM format if mentioned.
- Infer a sensible category from context (e.g. "exercise" → "health", "read" → "learning").
- Respond ONLY with the JSON object. No markdown fences, no explanation outside the JSON."#,
            today = today,
            task_context = task_context,
        );

        let request = ClaudeRequest {
            model: "claude-opus-4-6".to_string(),
            max_tokens: 1024,
            system,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let raw = self.send_request(request).await?;

        // Extract JSON — Claude occasionally adds surrounding text despite the prompt
        let json_str = extract_json(&raw)
            .context("Claude did not return valid JSON. Raw response logged above.")?;

        serde_json::from_str::<HabitPlan>(json_str)
            .context(format!("Failed to parse plan JSON:\n{}", json_str))
    }

    async fn send_request(&self, request: ClaudeRequest) -> Result<String> {
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
        format_task_list(tasks)
    }
}

pub fn format_task_list(tasks: &[crate::models::Task]) -> String {
    if tasks.is_empty() {
        return "No existing tasks.".to_string();
    }
    let mut context = String::new();
    for task in tasks {
        context.push_str(&format!(
            "- [{}] {} (ID: {}, Priority: {}, Category: {}, Daily: {})\n",
            if task.completed { "x" } else { " " },
            task.title,
            task.id,
            task.priority.to_string(),
            task.category,
            task.is_daily,
        ));
    }
    context
}

/// Find the outermost JSON object in a string.
fn extract_json(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end >= start {
        Some(&s[start..=end])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Task};

    fn task(id: &str, title: &str) -> Task {
        Task::new(id.to_string(), title.to_string(), Priority::Medium, "default".to_string())
    }

    fn action_with_days(days: Option<Vec<&str>>) -> PlanAction {
        PlanAction {
            action_type: "create_habit".to_string(),
            title: "Test".to_string(),
            category: None,
            priority: None,
            description: None,
            scheduled_days: days.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
            scheduled_time: None,
            location: None,
            two_minute: None,
            habit_stack_after: None,
        }
    }

    // --- day_str_to_num ---

    #[test]
    fn test_day_str_to_num_abbreviations() {
        assert_eq!(day_str_to_num("mon"), Some(0));
        assert_eq!(day_str_to_num("tue"), Some(1));
        assert_eq!(day_str_to_num("wed"), Some(2));
        assert_eq!(day_str_to_num("thu"), Some(3));
        assert_eq!(day_str_to_num("fri"), Some(4));
        assert_eq!(day_str_to_num("sat"), Some(5));
        assert_eq!(day_str_to_num("sun"), Some(6));
    }

    #[test]
    fn test_day_str_to_num_full_names() {
        assert_eq!(day_str_to_num("monday"), Some(0));
        assert_eq!(day_str_to_num("tuesday"), Some(1));
        assert_eq!(day_str_to_num("wednesday"), Some(2));
        assert_eq!(day_str_to_num("thursday"), Some(3));
        assert_eq!(day_str_to_num("friday"), Some(4));
        assert_eq!(day_str_to_num("saturday"), Some(5));
        assert_eq!(day_str_to_num("sunday"), Some(6));
    }

    #[test]
    fn test_day_str_to_num_case_insensitive() {
        assert_eq!(day_str_to_num("MON"), Some(0));
        assert_eq!(day_str_to_num("Monday"), Some(0));
        assert_eq!(day_str_to_num("FRI"), Some(4));
        assert_eq!(day_str_to_num("SUNDAY"), Some(6));
    }

    #[test]
    fn test_day_str_to_num_invalid() {
        assert_eq!(day_str_to_num(""), None);
        assert_eq!(day_str_to_num("weekday"), None);
        assert_eq!(day_str_to_num("mo"), None);
        assert_eq!(day_str_to_num("1"), None);
        assert_eq!(day_str_to_num("tues"), None);
    }

    // --- extract_json ---

    #[test]
    fn test_extract_json_clean() {
        assert_eq!(extract_json(r#"{"key": "value"}"#), Some(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_extract_json_with_leading_text() {
        let s = r#"Here is the JSON: {"key": "val"}"#;
        assert_eq!(extract_json(s), Some(r#"{"key": "val"}"#));
    }

    #[test]
    fn test_extract_json_with_trailing_text() {
        let s = r#"{"key": "val"} done."#;
        assert_eq!(extract_json(s), Some(r#"{"key": "val"}"#));
    }

    #[test]
    fn test_extract_json_nested_object() {
        let s = r#"{"outer": {"inner": 1}}"#;
        assert_eq!(extract_json(s), Some(r#"{"outer": {"inner": 1}}"#));
    }

    #[test]
    fn test_extract_json_no_braces() {
        assert_eq!(extract_json("no json here"), None);
        assert_eq!(extract_json(""), None);
    }

    #[test]
    fn test_extract_json_multiline() {
        let s = "prefix\n{\n  \"a\": 1\n}\nsuffix";
        let result = extract_json(s).unwrap();
        assert!(result.contains("\"a\""));
    }

    // --- PlanAction::scheduled_days_as_nums ---

    #[test]
    fn test_scheduled_days_as_nums_none() {
        assert!(action_with_days(None).scheduled_days_as_nums().is_none());
    }

    #[test]
    fn test_scheduled_days_as_nums_mon_wed_fri() {
        let a = action_with_days(Some(vec!["mon", "wed", "fri"]));
        assert_eq!(a.scheduled_days_as_nums(), Some(vec![0, 2, 4]));
    }

    #[test]
    fn test_scheduled_days_as_nums_full_names() {
        let a = action_with_days(Some(vec!["monday", "friday"]));
        assert_eq!(a.scheduled_days_as_nums(), Some(vec![0, 4]));
    }

    #[test]
    fn test_scheduled_days_as_nums_filters_invalid() {
        let a = action_with_days(Some(vec!["mon", "bad", "fri"]));
        assert_eq!(a.scheduled_days_as_nums(), Some(vec![0, 4]));
    }

    #[test]
    fn test_scheduled_days_as_nums_weekend() {
        let a = action_with_days(Some(vec!["sat", "sun"]));
        assert_eq!(a.scheduled_days_as_nums(), Some(vec![5, 6]));
    }

    // --- format_task_list ---

    #[test]
    fn test_format_task_list_empty() {
        assert_eq!(format_task_list(&[]), "No existing tasks.");
    }

    #[test]
    fn test_format_task_list_single_task() {
        let t = task("1", "Morning Run");
        let result = format_task_list(&[t]);
        assert!(result.contains("Morning Run"));
        assert!(result.contains("Medium"));
        assert!(result.contains("default"));
        assert!(result.contains("false")); // is_daily
    }

    #[test]
    fn test_format_task_list_completed_task() {
        let mut t = task("1", "Done task");
        t.mark_complete();
        let result = format_task_list(&[t]);
        assert!(result.contains("[x]"));
    }

    #[test]
    fn test_format_task_list_incomplete_task() {
        let t = task("1", "Pending");
        let result = format_task_list(&[t]);
        assert!(result.contains("[ ]"));
    }

    #[test]
    fn test_format_task_list_multiple_tasks() {
        let tasks = vec![task("1", "Task A"), task("2", "Task B")];
        let result = format_task_list(&tasks);
        assert!(result.contains("Task A"));
        assert!(result.contains("Task B"));
    }
}
