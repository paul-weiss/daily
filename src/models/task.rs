use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Some(Priority::Low),
            "medium" | "m" => Some(Priority::Medium),
            "high" | "h" => Some(Priority::High),
            "critical" | "c" => Some(Priority::Critical),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Priority::Low => "Low".to_string(),
            Priority::Medium => "Medium".to_string(),
            Priority::High => "High".to_string(),
            Priority::Critical => "Critical".to_string(),
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub category: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_daily: bool,
}

impl Task {
    pub fn new(title: String, priority: Priority, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description: None,
            priority,
            category,
            completed: false,
            created_at: now,
            updated_at: now,
            due_date: None,
            is_daily: false,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self
    }

    pub fn with_daily(mut self, is_daily: bool) -> Self {
        self.is_daily = is_daily;
        self
    }

    pub fn mark_complete(&mut self) {
        self.completed = true;
        self.updated_at = Utc::now();
    }

    pub fn mark_incomplete(&mut self) {
        self.completed = false;
        self.updated_at = Utc::now();
    }

    pub fn update_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }

    pub fn update_category(&mut self, category: String) {
        self.category = category;
        self.updated_at = Utc::now();
    }

    pub fn update_daily(&mut self, is_daily: bool) {
        self.is_daily = is_daily;
        self.updated_at = Utc::now();
    }
}
