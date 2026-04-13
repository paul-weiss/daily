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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 13).unwrap()
    }

    #[test]
    fn test_new_defaults() {
        let day = Day::new(date());
        assert_eq!(day.date, date());
        assert!(day.task_ids.is_empty());
        assert!(day.notes.is_none());
    }

    #[test]
    fn test_add_task() {
        let mut day = Day::new(date());
        day.add_task("1".to_string());
        assert_eq!(day.task_ids, vec!["1"]);
    }

    #[test]
    fn test_add_task_no_duplicates() {
        let mut day = Day::new(date());
        day.add_task("1".to_string());
        day.add_task("1".to_string());
        assert_eq!(day.task_ids.len(), 1);
    }

    #[test]
    fn test_add_multiple_distinct_tasks() {
        let mut day = Day::new(date());
        day.add_task("1".to_string());
        day.add_task("2".to_string());
        day.add_task("3".to_string());
        assert_eq!(day.task_ids.len(), 3);
    }
}
