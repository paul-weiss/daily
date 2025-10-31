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
        ];

        if let Some(desc) = &task.description {
            lines.push(format!("description: {}", desc));
        }

        if let Some(due) = &task.due_date {
            lines.push(format!("due_date: {}", due.to_rfc3339()));
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

        for line in text.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                match key {
                    "name" => name = value.to_string(),
                    "description" => description = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(Category { name, description })
    }
}
