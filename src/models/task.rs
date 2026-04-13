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
    // Atomic Habits: Make it Obvious
    pub scheduled_time: Option<String>,     // implementation intention: "HH:MM"
    pub location: Option<String>,           // implementation intention: where/context
    pub habit_stack_after: Option<String>,  // habit stacking: task ID this follows
    // Atomic Habits: Make it Easy
    pub two_minute: bool,                   // two-minute rule: starter version of habit
    // Selective recurrence: which weekdays (0=Mon..6=Sun); None means every day
    pub scheduled_days: Option<Vec<u8>>,
}

impl Task {
    pub fn new(id: String, title: String, priority: Priority, category: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description: None,
            priority,
            category,
            completed: false,
            created_at: now,
            updated_at: now,
            due_date: None,
            is_daily: false,
            scheduled_time: None,
            location: None,
            habit_stack_after: None,
            two_minute: false,
            scheduled_days: None,
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

    pub fn with_scheduled_time(mut self, time: String) -> Self {
        self.scheduled_time = Some(time);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_habit_stack_after(mut self, task_id: String) -> Self {
        self.habit_stack_after = Some(task_id);
        self
    }

    pub fn with_two_minute(mut self, two_minute: bool) -> Self {
        self.two_minute = two_minute;
        self
    }

    pub fn with_scheduled_days(mut self, days: Vec<u8>) -> Self {
        self.scheduled_days = if days.is_empty() { None } else { Some(days) };
        self
    }

    /// Returns the weekday numbers (0=Mon..6=Sun) as short names.
    pub fn scheduled_days_display(&self) -> Option<String> {
        self.scheduled_days.as_ref().map(|days| {
            let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
            days.iter()
                .filter_map(|&d| names.get(d as usize))
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Priority ---

    #[test]
    fn test_priority_from_str_full_names() {
        assert_eq!(Priority::from_str("low"), Some(Priority::Low));
        assert_eq!(Priority::from_str("medium"), Some(Priority::Medium));
        assert_eq!(Priority::from_str("high"), Some(Priority::High));
        assert_eq!(Priority::from_str("critical"), Some(Priority::Critical));
    }

    #[test]
    fn test_priority_from_str_abbreviations() {
        assert_eq!(Priority::from_str("l"), Some(Priority::Low));
        assert_eq!(Priority::from_str("m"), Some(Priority::Medium));
        assert_eq!(Priority::from_str("h"), Some(Priority::High));
        assert_eq!(Priority::from_str("c"), Some(Priority::Critical));
    }

    #[test]
    fn test_priority_from_str_case_insensitive() {
        assert_eq!(Priority::from_str("LOW"), Some(Priority::Low));
        assert_eq!(Priority::from_str("Medium"), Some(Priority::Medium));
        assert_eq!(Priority::from_str("HIGH"), Some(Priority::High));
        assert_eq!(Priority::from_str("CRITICAL"), Some(Priority::Critical));
    }

    #[test]
    fn test_priority_from_str_invalid() {
        assert_eq!(Priority::from_str(""), None);
        assert_eq!(Priority::from_str("urgent"), None);
        assert_eq!(Priority::from_str("normal"), None);
        assert_eq!(Priority::from_str("123"), None);
    }

    #[test]
    fn test_priority_to_string() {
        assert_eq!(Priority::Low.to_string(), "Low");
        assert_eq!(Priority::Medium.to_string(), "Medium");
        assert_eq!(Priority::High.to_string(), "High");
        assert_eq!(Priority::Critical.to_string(), "Critical");
    }

    #[test]
    fn test_priority_value_ordering() {
        assert!(Priority::Low.value() < Priority::Medium.value());
        assert!(Priority::Medium.value() < Priority::High.value());
        assert!(Priority::High.value() < Priority::Critical.value());
    }

    #[test]
    fn test_priority_value_exact() {
        assert_eq!(Priority::Low.value(), 1);
        assert_eq!(Priority::Medium.value(), 2);
        assert_eq!(Priority::High.value(), 3);
        assert_eq!(Priority::Critical.value(), 4);
    }

    // --- Task defaults ---

    fn task(id: &str) -> Task {
        Task::new(id.to_string(), "Test".to_string(), Priority::Medium, "work".to_string())
    }

    #[test]
    fn test_task_new_defaults() {
        let t = task("1");
        assert_eq!(t.id, "1");
        assert_eq!(t.title, "Test");
        assert_eq!(t.priority, Priority::Medium);
        assert_eq!(t.category, "work");
        assert!(!t.completed);
        assert!(!t.is_daily);
        assert!(!t.two_minute);
        assert!(t.description.is_none());
        assert!(t.due_date.is_none());
        assert!(t.scheduled_time.is_none());
        assert!(t.location.is_none());
        assert!(t.habit_stack_after.is_none());
        assert!(t.scheduled_days.is_none());
    }

    // --- Builders ---

    #[test]
    fn test_with_description() {
        let t = task("1").with_description("some desc".to_string());
        assert_eq!(t.description, Some("some desc".to_string()));
    }

    #[test]
    fn test_with_due_date() {
        let due = Utc::now();
        let t = task("1").with_due_date(due);
        assert!(t.due_date.is_some());
    }

    #[test]
    fn test_with_daily() {
        assert!(task("1").with_daily(true).is_daily);
        assert!(!task("1").with_daily(false).is_daily);
    }

    #[test]
    fn test_with_scheduled_time() {
        let t = task("1").with_scheduled_time("07:30".to_string());
        assert_eq!(t.scheduled_time, Some("07:30".to_string()));
    }

    #[test]
    fn test_with_location() {
        let t = task("1").with_location("gym".to_string());
        assert_eq!(t.location, Some("gym".to_string()));
    }

    #[test]
    fn test_with_habit_stack_after() {
        let t = task("1").with_habit_stack_after("42".to_string());
        assert_eq!(t.habit_stack_after, Some("42".to_string()));
    }

    #[test]
    fn test_with_two_minute() {
        assert!(task("1").with_two_minute(true).two_minute);
        assert!(!task("1").with_two_minute(false).two_minute);
    }

    #[test]
    fn test_with_scheduled_days() {
        let t = task("1").with_scheduled_days(vec![0, 2, 4]);
        assert_eq!(t.scheduled_days, Some(vec![0, 2, 4]));
    }

    #[test]
    fn test_with_scheduled_days_empty_becomes_none() {
        let t = task("1").with_scheduled_days(vec![]);
        assert!(t.scheduled_days.is_none());
    }

    // --- scheduled_days_display ---

    #[test]
    fn test_scheduled_days_display_none() {
        assert!(task("1").scheduled_days_display().is_none());
    }

    #[test]
    fn test_scheduled_days_display_mon_wed_fri() {
        let t = task("1").with_scheduled_days(vec![0, 2, 4]);
        assert_eq!(t.scheduled_days_display(), Some("Mon, Wed, Fri".to_string()));
    }

    #[test]
    fn test_scheduled_days_display_weekend() {
        let t = task("1").with_scheduled_days(vec![5, 6]);
        assert_eq!(t.scheduled_days_display(), Some("Sat, Sun".to_string()));
    }

    #[test]
    fn test_scheduled_days_display_all() {
        let t = task("1").with_scheduled_days(vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(
            t.scheduled_days_display(),
            Some("Mon, Tue, Wed, Thu, Fri, Sat, Sun".to_string())
        );
    }

    // --- Mutating methods ---

    #[test]
    fn test_mark_complete() {
        let mut t = task("1");
        t.mark_complete();
        assert!(t.completed);
    }

    #[test]
    fn test_mark_complete_updates_timestamp() {
        let mut t = task("1");
        let before = t.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        t.mark_complete();
        assert!(t.updated_at >= before);
    }

    #[test]
    fn test_mark_incomplete() {
        let mut t = task("1");
        t.mark_complete();
        t.mark_incomplete();
        assert!(!t.completed);
    }

    #[test]
    fn test_update_priority() {
        let mut t = task("1");
        t.update_priority(Priority::Critical);
        assert_eq!(t.priority, Priority::Critical);
    }

    #[test]
    fn test_update_category() {
        let mut t = task("1");
        t.update_category("personal".to_string());
        assert_eq!(t.category, "personal");
    }

    #[test]
    fn test_update_daily_toggle() {
        let mut t = task("1");
        t.update_daily(true);
        assert!(t.is_daily);
        t.update_daily(false);
        assert!(!t.is_daily);
    }
}
