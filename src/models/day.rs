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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_remove_task() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();
        let mut day = Day::new(date);

        let task_id = "task-123".to_string();
        day.add_task(task_id.clone());
        assert_eq!(day.task_ids.len(), 1);

        day.remove_task(&task_id);
        assert_eq!(day.task_ids.len(), 0);
    }

    #[test]
    fn test_remove_task_not_present() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();
        let mut day = Day::new(date);

        day.add_task("task-123".to_string());
        day.remove_task("task-456");

        assert_eq!(day.task_ids.len(), 1);
        assert_eq!(day.task_ids[0], "task-123");
    }

    #[test]
    fn test_with_notes() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();
        let day = Day::new(date).with_notes("Important meeting at 2pm".to_string());

        assert!(day.notes.is_some());
        assert_eq!(day.notes.unwrap(), "Important meeting at 2pm");
    }

    #[test]
    fn test_with_notes_builder_pattern() {
        let date = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();
        let mut day = Day::new(date).with_notes("First note".to_string());

        day.add_task("task-123".to_string());
        assert_eq!(day.task_ids.len(), 1);
        assert_eq!(day.notes.unwrap(), "First note");
    }
}
