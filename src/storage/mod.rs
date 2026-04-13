use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;
use crate::models::{Task, Day, Category, Priority};

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("tasks"))?;
        fs::create_dir_all(data_dir.join("days"))?;
        fs::create_dir_all(data_dir.join("categories"))?;

        Ok(Self { data_dir })
    }

    pub fn default_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".daily"))
    }

    // Task operations
    pub fn save_task(&self, task: &Task) -> Result<()> {
        let path = self.data_dir.join("tasks").join(format!("{}.txt", task.id));
        let content = self.task_to_text(task);
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_task(&self, id: &str) -> Result<Task> {
        let path = self.data_dir.join("tasks").join(format!("{}.txt", id));
        let content = fs::read_to_string(path)?;
        self.text_to_task(&content)
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        let path = self.data_dir.join("tasks").join(format!("{}.txt", id));
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn list_all_tasks(&self) -> Result<Vec<Task>> {
        let tasks_dir = self.data_dir.join("tasks");
        let mut tasks = Vec::new();

        if let Ok(entries) = fs::read_dir(tasks_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(task) = self.text_to_task(&content) {
                        tasks.push(task);
                    }
                }
            }
        }

        Ok(tasks)
    }

    pub fn list_tasks_by_category(&self, category: &str) -> Result<Vec<Task>> {
        let all_tasks = self.list_all_tasks()?;
        Ok(all_tasks.into_iter().filter(|t| t.category == category).collect())
    }

    // Day operations
    pub fn save_day(&self, day: &Day) -> Result<()> {
        let path = self.data_dir.join("days").join(format!("{}.txt", day.date));
        let content = self.day_to_text(day);
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_day(&self, date: NaiveDate) -> Result<Day> {
        let path = self.data_dir.join("days").join(format!("{}.txt", date));
        if path.exists() {
            let content = fs::read_to_string(path)?;
            self.text_to_day(&content)
        } else {
            Ok(Day::new(date))
        }
    }

    // Category operations
    pub fn save_category(&self, category: &Category) -> Result<()> {
        let path = self.data_dir.join("categories").join(format!("{}.txt", category.name));
        let mut content = format!("name: {}\n", category.name);
        if let Some(desc) = &category.description {
            content.push_str(&format!("description: {}\n", desc));
        }
        if let Some(identity) = &category.identity {
            content.push_str(&format!("identity: {}\n", identity));
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn list_categories(&self) -> Result<Vec<Category>> {
        let cat_dir = self.data_dir.join("categories");
        let mut categories = Vec::new();

        if let Ok(entries) = fs::read_dir(cat_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(category) = self.text_to_category(&content) {
                        categories.push(category);
                    }
                }
            }
        }

        Ok(categories)
    }

    // Daily task log operations
    pub fn log_daily_completion(&self, task_id: &str, task_title: &str, date: NaiveDate) -> Result<()> {
        let log_path = self.data_dir.join("daily.log");
        let log_entry = format!("{} | {} | {}\n", date, task_id, task_title);

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        file.write_all(log_entry.as_bytes())?;
        Ok(())
    }

    pub fn is_daily_completed_on_date(&self, task_id: &str, date: NaiveDate) -> Result<bool> {
        let log_path = self.data_dir.join("daily.log");

        if !log_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(log_path)?;
        let date_str = date.to_string();

        for line in content.lines() {
            if let Some((log_date, rest)) = line.split_once(" | ") {
                if log_date == date_str {
                    if let Some((log_id, _)) = rest.split_once(" | ") {
                        if log_id == task_id {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn log_task_completion(&self, task_id: &str, task_title: &str) -> Result<()> {
        use chrono::Local;
        use std::fs::OpenOptions;
        use std::io::Write;

        let log_path = self.data_dir.join("history.log");
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("{} | {} | {}\n", timestamp, task_id, task_title);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        file.write_all(log_entry.as_bytes())?;
        Ok(())
    }

    pub fn get_next_task_id(&self) -> Result<String> {
        use std::fs::OpenOptions;
        use std::io::{Read, Write};

        let counter_path = self.data_dir.join("id_counter.txt");

        let next_id = if counter_path.exists() {
            let mut content = String::new();
            let mut file = OpenOptions::new().read(true).open(&counter_path)?;
            file.read_to_string(&mut content)?;
            content.trim().parse::<u64>().unwrap_or(1)
        } else {
            1
        };

        // Write the incremented counter back
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&counter_path)?;
        file.write_all(format!("{}", next_id + 1).as_bytes())?;

        Ok(next_id.to_string())
    }

    // Conversion helpers
    fn task_to_text(&self, task: &Task) -> String {
        let mut lines = vec![
            format!("id: {}", task.id),
            format!("title: {}", task.title),
            format!("priority: {}", task.priority.to_string()),
            format!("category: {}", task.category),
            format!("completed: {}", task.completed),
            format!("created_at: {}", task.created_at.to_rfc3339()),
            format!("updated_at: {}", task.updated_at.to_rfc3339()),
            format!("is_daily: {}", task.is_daily),
            format!("two_minute: {}", task.two_minute),
        ];

        if let Some(desc) = &task.description {
            lines.push(format!("description: {}", desc));
        }

        if let Some(due) = &task.due_date {
            lines.push(format!("due_date: {}", due.to_rfc3339()));
        }

        if let Some(time) = &task.scheduled_time {
            lines.push(format!("scheduled_time: {}", time));
        }

        if let Some(loc) = &task.location {
            lines.push(format!("location: {}", loc));
        }

        if let Some(after) = &task.habit_stack_after {
            lines.push(format!("habit_stack_after: {}", after));
        }

        if let Some(days) = &task.scheduled_days {
            let s: Vec<String> = days.iter().map(|d| d.to_string()).collect();
            lines.push(format!("scheduled_days: {}", s.join(",")));
        }

        lines.join("\n")
    }

    fn text_to_task(&self, text: &str) -> Result<Task> {
        let mut id = String::new();
        let mut title = String::new();
        let mut description = None;
        let mut priority = Priority::Medium;
        let mut category = String::from("default");
        let mut completed = false;
        let mut created_at = None;
        let mut updated_at = None;
        let mut due_date = None;
        let mut is_daily = false;
        let mut scheduled_time = None;
        let mut location = None;
        let mut habit_stack_after = None;
        let mut two_minute = false;
        let mut scheduled_days: Option<Vec<u8>> = None;

        for line in text.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                match key {
                    "id" => id = value.to_string(),
                    "title" => title = value.to_string(),
                    "description" => description = Some(value.to_string()),
                    "priority" => priority = Priority::from_str(value).unwrap_or(Priority::Medium),
                    "category" => category = value.to_string(),
                    "completed" => completed = value.parse().unwrap_or(false),
                    "created_at" => created_at = value.parse().ok(),
                    "updated_at" => updated_at = value.parse().ok(),
                    "due_date" => due_date = value.parse().ok(),
                    "is_daily" => is_daily = value.parse().unwrap_or(false),
                    "scheduled_time" => scheduled_time = Some(value.to_string()),
                    "location" => location = Some(value.to_string()),
                    "habit_stack_after" => habit_stack_after = Some(value.to_string()),
                    "two_minute" => two_minute = value.parse().unwrap_or(false),
                    "scheduled_days" => {
                        let nums: Vec<u8> = value.split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        if !nums.is_empty() {
                            scheduled_days = Some(nums);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(Task {
            id,
            title,
            description,
            priority,
            category,
            completed,
            created_at: created_at.context("Missing created_at")?,
            updated_at: updated_at.context("Missing updated_at")?,
            due_date,
            is_daily,
            scheduled_time,
            location,
            habit_stack_after,
            two_minute,
            scheduled_days,
        })
    }

    fn day_to_text(&self, day: &Day) -> String {
        let mut lines = vec![format!("date: {}", day.date)];

        if !day.task_ids.is_empty() {
            lines.push(format!("tasks: {}", day.task_ids.join(",")));
        }

        if let Some(notes) = &day.notes {
            lines.push(format!("notes: {}", notes));
        }

        lines.join("\n")
    }

    fn text_to_day(&self, text: &str) -> Result<Day> {
        let mut date = None;
        let mut task_ids = Vec::new();
        let mut notes = None;

        for line in text.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                match key {
                    "date" => date = Some(value.parse()?),
                    "tasks" => task_ids = value.split(',').map(|s| s.to_string()).collect(),
                    "notes" => notes = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(Day {
            date: date.context("Missing date")?,
            task_ids,
            notes,
        })
    }

    fn text_to_category(&self, text: &str) -> Result<Category> {
        let mut name = String::new();
        let mut description = None;
        let mut identity = None;

        for line in text.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                match key {
                    "name" => name = value.to_string(),
                    "description" => description = Some(value.to_string()),
                    "identity" => identity = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(Category { name, description, identity })
    }

    // Atomic Habits: streak tracking — "Make it Satisfying"
    // Returns the number of consecutive days (going backwards from `as_of`) the task was completed.
    pub fn get_streak_for_task(&self, task_id: &str, as_of: NaiveDate) -> Result<u32> {
        let mut streak = 0u32;
        let mut check = as_of;
        loop {
            if self.is_daily_completed_on_date(task_id, check)? {
                streak += 1;
                match check.pred_opt() {
                    Some(prev) => check = prev,
                    None => break,
                }
            } else {
                break;
            }
        }
        Ok(streak)
    }

    // Returns a vec of booleans for the last `days` days (oldest first, newest last).
    pub fn get_habit_grid(&self, task_id: &str, as_of: NaiveDate, days: u32) -> Result<Vec<bool>> {
        let mut grid = Vec::with_capacity(days as usize);
        for i in (0..days).rev() {
            let date = as_of - chrono::Duration::days(i as i64);
            grid.push(self.is_daily_completed_on_date(task_id, date)?);
        }
        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, Priority, Task};
    use chrono::Utc;
    use tempfile::TempDir;

    // --- helpers ---

    fn test_storage() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf()).unwrap();
        (dir, storage)
    }

    fn task(id: &str, title: &str) -> Task {
        Task::new(id.to_string(), title.to_string(), Priority::Medium, "default".to_string())
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // --- storage setup ---

    #[test]
    fn test_new_creates_subdirectories() {
        let dir = TempDir::new().unwrap();
        Storage::new(dir.path().to_path_buf()).unwrap();
        assert!(dir.path().join("tasks").is_dir());
        assert!(dir.path().join("days").is_dir());
        assert!(dir.path().join("categories").is_dir());
    }

    // --- task CRUD ---

    #[test]
    fn test_save_and_load_task_basic() {
        let (_dir, s) = test_storage();
        let t = task("1", "Buy groceries");
        s.save_task(&t).unwrap();
        let loaded = s.load_task("1").unwrap();
        assert_eq!(loaded.id, "1");
        assert_eq!(loaded.title, "Buy groceries");
        assert_eq!(loaded.priority, Priority::Medium);
        assert_eq!(loaded.category, "default");
        assert!(!loaded.completed);
        assert!(!loaded.is_daily);
        assert!(!loaded.two_minute);
    }

    #[test]
    fn test_save_and_load_task_all_optional_fields() {
        let (_dir, s) = test_storage();
        let due = Utc::now();
        let t = Task::new("2".to_string(), "Run".to_string(), Priority::High, "health".to_string())
            .with_description("Morning run".to_string())
            .with_due_date(due)
            .with_daily(true)
            .with_scheduled_time("06:30".to_string())
            .with_location("front door".to_string())
            .with_habit_stack_after("1".to_string())
            .with_two_minute(true)
            .with_scheduled_days(vec![0, 2, 4]);
        s.save_task(&t).unwrap();
        let loaded = s.load_task("2").unwrap();
        assert_eq!(loaded.description, Some("Morning run".to_string()));
        assert!(loaded.due_date.is_some());
        assert!(loaded.is_daily);
        assert_eq!(loaded.scheduled_time, Some("06:30".to_string()));
        assert_eq!(loaded.location, Some("front door".to_string()));
        assert_eq!(loaded.habit_stack_after, Some("1".to_string()));
        assert!(loaded.two_minute);
        assert_eq!(loaded.scheduled_days, Some(vec![0, 2, 4]));
    }

    #[test]
    fn test_load_task_not_found() {
        let (_dir, s) = test_storage();
        assert!(s.load_task("ghost").is_err());
    }

    #[test]
    fn test_delete_task() {
        let (_dir, s) = test_storage();
        s.save_task(&task("1", "Gone")).unwrap();
        s.delete_task("1").unwrap();
        assert!(s.load_task("1").is_err());
    }

    #[test]
    fn test_delete_task_not_found() {
        let (_dir, s) = test_storage();
        assert!(s.delete_task("ghost").is_err());
    }

    #[test]
    fn test_list_all_tasks_empty() {
        let (_dir, s) = test_storage();
        assert!(s.list_all_tasks().unwrap().is_empty());
    }

    #[test]
    fn test_list_all_tasks_returns_all() {
        let (_dir, s) = test_storage();
        s.save_task(&task("1", "A")).unwrap();
        s.save_task(&task("2", "B")).unwrap();
        s.save_task(&task("3", "C")).unwrap();
        assert_eq!(s.list_all_tasks().unwrap().len(), 3);
    }

    #[test]
    fn test_list_tasks_by_category() {
        let (_dir, s) = test_storage();
        let work = Task::new("1".to_string(), "Work".to_string(), Priority::High, "work".to_string());
        let personal = Task::new("2".to_string(), "Personal".to_string(), Priority::Low, "personal".to_string());
        let work2 = Task::new("3".to_string(), "Work2".to_string(), Priority::Medium, "work".to_string());
        s.save_task(&work).unwrap();
        s.save_task(&personal).unwrap();
        s.save_task(&work2).unwrap();

        let work_tasks = s.list_tasks_by_category("work").unwrap();
        assert_eq!(work_tasks.len(), 2);
        assert!(work_tasks.iter().all(|t| t.category == "work"));

        let personal_tasks = s.list_tasks_by_category("personal").unwrap();
        assert_eq!(personal_tasks.len(), 1);

        assert!(s.list_tasks_by_category("nonexistent").unwrap().is_empty());
    }

    #[test]
    fn test_task_priority_roundtrip() {
        let (_dir, s) = test_storage();
        let t = Task::new("1".to_string(), "T".to_string(), Priority::Critical, "w".to_string());
        s.save_task(&t).unwrap();
        assert_eq!(s.load_task("1").unwrap().priority, Priority::Critical);
    }

    #[test]
    fn test_task_completed_roundtrip() {
        let (_dir, s) = test_storage();
        let mut t = task("1", "Done");
        t.mark_complete();
        s.save_task(&t).unwrap();
        assert!(s.load_task("1").unwrap().completed);
    }

    #[test]
    fn test_task_scheduled_days_roundtrip() {
        let (_dir, s) = test_storage();
        let t = task("1", "Yoga").with_daily(true).with_scheduled_days(vec![0, 2, 4]);
        s.save_task(&t).unwrap();
        assert_eq!(s.load_task("1").unwrap().scheduled_days, Some(vec![0, 2, 4]));
    }

    #[test]
    fn test_task_no_scheduled_days_roundtrip() {
        let (_dir, s) = test_storage();
        let t = task("1", "Daily").with_daily(true);
        s.save_task(&t).unwrap();
        assert!(s.load_task("1").unwrap().scheduled_days.is_none());
    }

    // --- day CRUD ---

    #[test]
    fn test_save_and_load_day() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        let mut day = Day::new(d);
        day.add_task("1".to_string());
        day.add_task("2".to_string());
        s.save_day(&day).unwrap();
        let loaded = s.load_day(d).unwrap();
        assert_eq!(loaded.date, d);
        assert!(loaded.task_ids.contains(&"1".to_string()));
        assert!(loaded.task_ids.contains(&"2".to_string()));
    }

    #[test]
    fn test_load_day_missing_returns_empty() {
        let (_dir, s) = test_storage();
        let d = date(2026, 1, 1);
        let day = s.load_day(d).unwrap();
        assert_eq!(day.date, d);
        assert!(day.task_ids.is_empty());
    }

    // --- category CRUD ---

    #[test]
    fn test_save_and_list_category() {
        let (_dir, s) = test_storage();
        s.save_category(&Category::new("work".to_string())).unwrap();
        let cats = s.list_categories().unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "work");
    }

    #[test]
    fn test_list_categories_empty() {
        let (_dir, s) = test_storage();
        assert!(s.list_categories().unwrap().is_empty());
    }

    #[test]
    fn test_category_description_roundtrip() {
        let (_dir, s) = test_storage();
        let cat = Category::new("health".to_string()).with_description("Wellbeing".to_string());
        s.save_category(&cat).unwrap();
        let cats = s.list_categories().unwrap();
        assert_eq!(cats[0].description, Some("Wellbeing".to_string()));
    }

    #[test]
    fn test_category_identity_roundtrip() {
        let (_dir, s) = test_storage();
        let cat = Category::new("fitness".to_string())
            .with_identity("I am someone who moves every day".to_string());
        s.save_category(&cat).unwrap();
        let cats = s.list_categories().unwrap();
        assert_eq!(cats[0].identity, Some("I am someone who moves every day".to_string()));
    }

    #[test]
    fn test_category_all_fields_roundtrip() {
        let (_dir, s) = test_storage();
        let cat = Category::new("x".to_string())
            .with_description("desc".to_string())
            .with_identity("identity".to_string());
        s.save_category(&cat).unwrap();
        let cats = s.list_categories().unwrap();
        assert_eq!(cats[0].description, Some("desc".to_string()));
        assert_eq!(cats[0].identity, Some("identity".to_string()));
    }

    // --- daily log ---

    #[test]
    fn test_daily_not_completed_no_log_file() {
        let (_dir, s) = test_storage();
        assert!(!s.is_daily_completed_on_date("t1", date(2026, 4, 13)).unwrap());
    }

    #[test]
    fn test_log_and_check_daily_completion() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        assert!(s.is_daily_completed_on_date("t1", d).unwrap());
    }

    #[test]
    fn test_daily_completion_wrong_id_not_matched() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        assert!(!s.is_daily_completed_on_date("t2", d).unwrap());
    }

    #[test]
    fn test_daily_completion_wrong_date_not_matched() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        assert!(!s.is_daily_completed_on_date("t1", date(2026, 4, 14)).unwrap());
    }

    #[test]
    fn test_daily_completion_multiple_tasks_same_day() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        s.log_daily_completion("t2", "Read", d).unwrap();
        assert!(s.is_daily_completed_on_date("t1", d).unwrap());
        assert!(s.is_daily_completed_on_date("t2", d).unwrap());
    }

    #[test]
    fn test_log_task_completion_creates_history_log() {
        let (_dir, s) = test_storage();
        s.log_task_completion("t1", "Finish report").unwrap();
        let content = std::fs::read_to_string(s.data_dir.join("history.log")).unwrap();
        assert!(content.contains("t1"));
        assert!(content.contains("Finish report"));
    }

    // --- ID counter ---

    #[test]
    fn test_get_next_task_id_starts_at_one() {
        let (_dir, s) = test_storage();
        assert_eq!(s.get_next_task_id().unwrap(), "1");
    }

    #[test]
    fn test_get_next_task_id_sequential() {
        let (_dir, s) = test_storage();
        assert_eq!(s.get_next_task_id().unwrap(), "1");
        assert_eq!(s.get_next_task_id().unwrap(), "2");
        assert_eq!(s.get_next_task_id().unwrap(), "3");
    }

    // --- streak ---

    #[test]
    fn test_streak_zero_with_no_completions() {
        let (_dir, s) = test_storage();
        assert_eq!(s.get_streak_for_task("t1", date(2026, 4, 13)).unwrap(), 0);
    }

    #[test]
    fn test_streak_one_day() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        assert_eq!(s.get_streak_for_task("t1", d).unwrap(), 1);
    }

    #[test]
    fn test_streak_consecutive_days() {
        let (_dir, s) = test_storage();
        let today = date(2026, 4, 13);
        for i in 0..5 {
            s.log_daily_completion("t1", "Run", today - chrono::Duration::days(i)).unwrap();
        }
        assert_eq!(s.get_streak_for_task("t1", today).unwrap(), 5);
    }

    #[test]
    fn test_streak_broken_by_gap() {
        let (_dir, s) = test_storage();
        let today = date(2026, 4, 13);
        // Complete today and yesterday, gap on -2, then -3
        s.log_daily_completion("t1", "Run", today).unwrap();
        s.log_daily_completion("t1", "Run", today - chrono::Duration::days(1)).unwrap();
        s.log_daily_completion("t1", "Run", today - chrono::Duration::days(3)).unwrap();
        assert_eq!(s.get_streak_for_task("t1", today).unwrap(), 2);
    }

    #[test]
    fn test_streak_ignores_other_task() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t2", "Read", d).unwrap();
        assert_eq!(s.get_streak_for_task("t1", d).unwrap(), 0);
    }

    // --- habit grid ---

    #[test]
    fn test_habit_grid_all_false_no_completions() {
        let (_dir, s) = test_storage();
        let grid = s.get_habit_grid("t1", date(2026, 4, 13), 7).unwrap();
        assert_eq!(grid.len(), 7);
        assert!(grid.iter().all(|&b| !b));
    }

    #[test]
    fn test_habit_grid_single_day_true() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        s.log_daily_completion("t1", "Run", d).unwrap();
        let grid = s.get_habit_grid("t1", d, 1).unwrap();
        assert_eq!(grid, vec![true]);
    }

    #[test]
    fn test_habit_grid_correct_positions() {
        let (_dir, s) = test_storage();
        let today = date(2026, 4, 13);
        // Complete today (index 6) and 2 days ago (index 4) in a 7-day window
        s.log_daily_completion("t1", "Run", today).unwrap();
        s.log_daily_completion("t1", "Run", today - chrono::Duration::days(2)).unwrap();
        let grid = s.get_habit_grid("t1", today, 7).unwrap();
        assert_eq!(grid.len(), 7);
        assert!(grid[6]);  // today
        assert!(!grid[5]); // yesterday
        assert!(grid[4]);  // 2 days ago
        assert!(!grid[3]); // 3 days ago
    }

    #[test]
    fn test_habit_grid_length_matches_days_param() {
        let (_dir, s) = test_storage();
        let d = date(2026, 4, 13);
        assert_eq!(s.get_habit_grid("t1", d, 14).unwrap().len(), 14);
        assert_eq!(s.get_habit_grid("t1", d, 21).unwrap().len(), 21);
        assert_eq!(s.get_habit_grid("t1", d, 1).unwrap().len(), 1);
    }
}
