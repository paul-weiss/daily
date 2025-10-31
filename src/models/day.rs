use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub date: NaiveDate,
    pub task_ids: Vec<String>,
    pub notes: Option<String>,
}

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            task_ids: Vec::new(),
            notes: None,
        }
    }

    pub fn add_task(&mut self, task_id: String) {
        if !self.task_ids.contains(&task_id) {
            self.task_ids.push(task_id);
        }
    }

    pub fn remove_task(&mut self, task_id: &str) {
        self.task_ids.retain(|id| id != task_id);
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }
}
