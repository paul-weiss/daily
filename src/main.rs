mod models;
mod storage;
mod cli;
mod scheduler;
mod claude;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::Parser;
use models::{Priority, Task, Category};
use storage::Storage;
use cli::{Cli, Commands};
use scheduler::Scheduler;
use claude::ClaudeClient;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let storage = Storage::new(Storage::default_dir()?)?;

    match cli.command {
        Commands::Add {
            title,
            priority,
            category,
            description,
            due,
        } => {
            let priority = Priority::from_str(&priority)
                .context("Invalid priority. Use: low, medium, high, or critical")?;

            let mut task = Task::new(title, priority, category);

            if let Some(desc) = description {
                task = task.with_description(desc);
            }

            if let Some(due_str) = due {
                let due_date = NaiveDate::parse_from_str(&due_str, "%Y-%m-%d")?;
                let due_datetime = due_date.and_hms_opt(23, 59, 59)
                    .context("Invalid date")?
                    .and_utc();
                task = task.with_due_date(due_datetime);
            }

            storage.save_task(&task)?;
            println!("Task added successfully!");
            println!("ID: {}", task.id);
            println!("Title: {}", task.title);
            println!("Priority: {}", task.priority.to_string());
            println!("Category: {}", task.category);
        }

        Commands::List { category, incomplete, completed } => {
            let mut tasks = if let Some(cat) = category {
                storage.list_tasks_by_category(&cat)?
            } else {
                storage.list_all_tasks()?
            };

            // Apply filters
            if incomplete {
                tasks.retain(|t| !t.completed);
            } else if completed {
                tasks.retain(|t| t.completed);
            }

            // Sort by priority (high to low) and then by category
            tasks.sort_by(|a, b| {
                b.priority.value()
                    .cmp(&a.priority.value())
                    .then(a.category.cmp(&b.category))
                    .then(a.title.cmp(&b.title))
            });

            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                println!("\n{} task(s) found:\n", tasks.len());

                let mut current_category = String::new();
                for task in tasks {
                    // Print category header if changed
                    if task.category != current_category {
                        println!("\n=== {} ===", task.category.to_uppercase());
                        current_category = task.category.clone();
                    }

                    let status = if task.completed { "[✓]" } else { "[ ]" };
                    println!(
                        "{} {} - {} (Priority: {})",
                        status,
                        task.id[..8].to_string(),
                        task.title,
                        task.priority.to_string()
                    );

                    if let Some(desc) = &task.description {
                        println!("    {}", desc);
                    }

                    if let Some(due) = &task.due_date {
                        println!("    Due: {}", due.format("%Y-%m-%d"));
                    }
                }
                println!();
            }
        }

        Commands::Complete { id } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            task.mark_complete();
            storage.save_task(&task)?;
            println!("Task '{}' marked as complete!", task.title);
        }

        Commands::Uncomplete { id } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            task.mark_incomplete();
            storage.save_task(&task)?;
            println!("Task '{}' marked as incomplete!", task.title);
        }

        Commands::Delete { id } => {
            let task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            let title = task.title.clone();
            storage.delete_task(&task.id)?;
            println!("Task '{}' deleted!", title);
        }

        Commands::Priority { id, priority } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            let new_priority = Priority::from_str(&priority)
                .context("Invalid priority. Use: low, medium, high, or critical")?;
            task.update_priority(new_priority);
            storage.save_task(&task)?;
            println!("Task '{}' priority updated to {}!", task.title, task.priority.to_string());
        }

        Commands::Move { id, category } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            task.update_category(category.clone());
            storage.save_task(&task)?;
            println!("Task '{}' moved to category '{}'!", task.title, category);
        }

        Commands::Category { name, description } => {
            let mut category = Category::new(name.clone());
            if let Some(desc) = description {
                category = category.with_description(desc);
            }
            storage.save_category(&category)?;
            println!("Category '{}' created!", name);
        }

        Commands::Categories => {
            let categories = storage.list_categories()?;
            if categories.is_empty() {
                println!("No categories found.");
            } else {
                println!("\nCategories:\n");
                for cat in categories {
                    println!("- {}", cat.name);
                    if let Some(desc) = cat.description {
                        println!("  {}", desc);
                    }
                }
            }
        }

        Commands::Today => {
            let today = Local::now().date_naive();
            show_day_tasks(&storage, today)?;
        }

        Commands::Day { date } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
            show_day_tasks(&storage, date)?;
        }

        Commands::Schedule { task_id, date } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
            let task = storage.load_task(&task_id)
                .or_else(|_| find_task_by_prefix(&storage, &task_id))?;

            let mut day = storage.load_day(date)?;
            day.add_task(task.id.clone());
            storage.save_day(&day)?;

            println!("Task '{}' scheduled for {}!", task.title, date);
        }

        Commands::Daemon { time } => {
            println!("Starting daily prompt daemon...");
            println!("Daily prompt will appear at {}", time);
            println!("Press Ctrl+C to stop.");

            let scheduler = Scheduler::new(&time)?;
            scheduler.run().await?;
        }

        Commands::Claude { prompt } => {
            let client = ClaudeClient::new()
                .context("Failed to initialize Claude client. Make sure ANTHROPIC_API_KEY is set.")?;

            // Load all tasks to provide context
            let tasks = storage.list_all_tasks()?;
            let context = client.build_task_context(&tasks);

            println!("Thinking...");

            let response = client.chat(&prompt, Some(&context)).await?;

            println!("\nClaude's response:\n");
            println!("{}", response);
        }
    }

    Ok(())
}

fn find_task_by_prefix(storage: &Storage, prefix: &str) -> Result<Task> {
    let tasks = storage.list_all_tasks()?;
    let matching: Vec<_> = tasks.into_iter()
        .filter(|t| t.id.starts_with(prefix))
        .collect();

    match matching.len() {
        0 => anyhow::bail!("No task found with ID starting with '{}'", prefix),
        1 => Ok(matching.into_iter().next().unwrap()),
        _ => anyhow::bail!("Multiple tasks found with ID starting with '{}'. Please be more specific.", prefix),
    }
}

fn show_day_tasks(storage: &Storage, date: NaiveDate) -> Result<()> {
    let day = storage.load_day(date)?;

    println!("\nTasks for {}:\n", date);

    // Collect tasks scheduled for this day
    let mut tasks: Vec<Task> = day.task_ids
        .iter()
        .filter_map(|id| storage.load_task(id).ok())
        .collect();

    // Also collect tasks with due_date matching this date
    let all_tasks = storage.list_all_tasks()?;
    for task in all_tasks {
        // Check if the due_date matches this date
        if let Some(due) = task.due_date {
            if due.date_naive() == date {
                // Add if not already in the list
                if !tasks.iter().any(|t| t.id == task.id) {
                    tasks.push(task);
                }
            }
        }
    }

    if tasks.is_empty() {
        println!("No tasks scheduled for this day.");
    } else {
        // Sort by category and priority
        tasks.sort_by(|a, b| {
            a.category.cmp(&b.category)
                .then(b.priority.value().cmp(&a.priority.value()))
        });

        let mut current_category = String::new();
        for task in tasks {
            if task.category != current_category {
                println!("\n=== {} ===", task.category.to_uppercase());
                current_category = task.category.clone();
            }

            let status = if task.completed { "[✓]" } else { "[ ]" };
            println!(
                "{} {} - {} (Priority: {})",
                status,
                &task.id[..8],
                task.title,
                task.priority.to_string()
            );
        }
        println!();
    }

    if let Some(notes) = day.notes {
        println!("Notes: {}\n", notes);
    }

    Ok(())
}
