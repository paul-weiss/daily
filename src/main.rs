mod models;
mod storage;
mod cli;
mod scheduler;
mod claude;

use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use models::{Priority, Task, Category};
use storage::Storage;
use cli::{Cli, Commands};
use scheduler::Scheduler;
use claude::ClaudeClient;
use rand::seq::SliceRandom;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = if let Some(ref dir) = cli.data_dir {
        std::path::PathBuf::from(dir)
    } else {
        Storage::default_dir()?
    };
    let storage = Storage::new(data_dir)?;

    match cli.command {
        Commands::Add {
            title,
            priority,
            category,
            description,
            due,
            daily,
            time,
            location,
            after,
            two_minute,
            days,
        } => {
            let priority = Priority::from_str(&priority)
                .context("Invalid priority. Use: low, medium, high, or critical")?;

            let task_id = storage.get_next_task_id()?;
            let mut task = Task::new(task_id, title, priority, category);

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

            if daily {
                task = task.with_daily(true);
            }

            if let Some(t) = time {
                task = task.with_scheduled_time(t);
            }

            if let Some(loc) = location {
                task = task.with_location(loc);
            }

            if let Some(after_id) = after {
                task = task.with_habit_stack_after(after_id);
            }

            if two_minute {
                task = task.with_two_minute(true);
            }

            if let Some(days_str) = days {
                let nums: Vec<u8> = days_str.split(',')
                    .filter_map(|d| claude::day_str_to_num(d.trim()))
                    .collect();
                task = task.with_scheduled_days(nums);
            }

            storage.save_task(&task)?;
            println!("Task added successfully!");
            println!("ID: {}", task.id);
            println!("Title: {}", task.title);
            println!("Priority: {}", task.priority.to_string());
            println!("Category: {}", task.category);
            if task.is_daily {
                println!("Type: Daily recurring task");
            }
            if let Some(ref t) = task.scheduled_time {
                println!("When: {}", t);
            }
            if let Some(ref loc) = task.location {
                println!("Where: {}", loc);
            }
            if let Some(ref after) = task.habit_stack_after {
                println!("After: task {}", after);
            }
            if task.two_minute {
                println!("Two-minute rule: yes (starter version)");
            }
        }

        Commands::List { category, priority, incomplete, completed, random } => {
            let mut tasks = if let Some(cat) = category {
                storage.list_tasks_by_category(&cat)?
            } else {
                storage.list_all_tasks()?
            };

            // Apply filters
            if let Some(priority_str) = priority {
                let priority_filter = Priority::from_str(&priority_str)
                    .context("Invalid priority. Use: low, medium, high, or critical")?;
                tasks.retain(|t| t.priority == priority_filter);
            }

            if incomplete {
                tasks.retain(|t| !t.completed);
            } else if completed {
                tasks.retain(|t| t.completed);
            }

            // If random flag is set, select one random task from each category
            if random {
                use std::collections::HashMap;
                let mut rng = rand::thread_rng();

                // Group tasks by category
                let mut by_category: HashMap<String, Vec<Task>> = HashMap::new();
                for task in tasks {
                    by_category.entry(task.category.clone())
                        .or_insert_with(Vec::new)
                        .push(task);
                }

                // Select one random task from each category
                tasks = by_category.into_iter()
                    .filter_map(|(_, mut cat_tasks)| {
                        cat_tasks.shuffle(&mut rng);
                        cat_tasks.into_iter().next()
                    })
                    .collect();
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
                    let daily_indicator = if task.is_daily { " [Daily]" } else { "" };
                    println!(
                        "{} {} - {}{} (Priority: {})",
                        status,
                        task.id,
                        task.title,
                        daily_indicator,
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

            if task.is_daily {
                // For daily tasks, log completion to daily.log instead of marking as complete
                let today = Local::now().date_naive();
                storage.log_daily_completion(&task.id, &task.title, today)?;
                println!("Daily task '{}' completed for {}!", task.title, today);

                // Atomic Habits: Make it Satisfying — show streak
                let streak = storage.get_streak_for_task(&task.id, today)?;
                if streak == 1 {
                    println!("Day 1 — every streak starts here. Keep going!");
                } else if streak > 1 {
                    println!("Streak: {} days — don't break the chain!", streak);
                }

                // Show identity reinforcement if category has one
                if let Ok(cats) = storage.list_categories() {
                    if let Some(cat) = cats.iter().find(|c| c.name == task.category) {
                        if let Some(ref identity) = cat.identity {
                            println!("Identity: {}", identity);
                        }
                    }
                }
            } else {
                // For regular tasks, mark as complete and log to history
                task.mark_complete();
                storage.save_task(&task)?;
                storage.log_task_completion(&task.id, &task.title)?;
                println!("Task '{}' marked as complete!", task.title);
            }
        }

        Commands::Uncomplete { id } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            task.mark_incomplete();
            storage.save_task(&task)?;
            println!("Task '{}' marked as incomplete!", task.title);
        }

        Commands::UncompleteAll => {
            let mut tasks = storage.list_all_tasks()?;
            let mut count = 0;
            for task in tasks.iter_mut() {
                if task.completed {
                    task.mark_incomplete();
                    storage.save_task(task)?;
                    count += 1;
                }
            }
            println!("{} task(s) marked as incomplete!", count);
        }

        Commands::Delete { id } => {
            let task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            let title = task.title.clone();
            storage.delete_task(&task.id)?;
            println!("Task '{}' deleted!", title);
        }

        Commands::DeleteAll => {
            let tasks = storage.list_all_tasks()?;
            let count = tasks.len();
            for task in tasks {
                storage.delete_task(&task.id)?;
            }
            println!("{} task(s) deleted.", count);
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

        Commands::Daily { id, daily } => {
            let mut task = storage.load_task(&id)
                .or_else(|_| find_task_by_prefix(&storage, &id))?;
            task.update_daily(daily);
            storage.save_task(&task)?;
            if daily {
                println!("Task '{}' is now a daily recurring task!", task.title);
            } else {
                println!("Task '{}' is no longer a daily recurring task!", task.title);
            }
        }

        Commands::Category { name, description, identity } => {
            let mut category = Category::new(name.clone());
            if let Some(desc) = description {
                category = category.with_description(desc);
            }
            if let Some(id_stmt) = identity {
                category = category.with_identity(id_stmt);
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

        Commands::Today { completed, all } => {
            let today = Local::now().date_naive();
            let filter = if all { DayFilter::All } else if completed { DayFilter::Completed } else { DayFilter::Incomplete };
            show_day_tasks(&storage, today, filter)?;
        }

        Commands::Day { date } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
            show_day_tasks(&storage, date, DayFilter::Incomplete)?;
        }

        Commands::TodayPdf { output } => {
            let today = Local::now().date_naive();
            let output_path = if let Some(path) = output {
                path
            } else {
                let home = dirs::home_dir().context("Could not find home directory")?;
                home.join(format!("daily-{}.pdf", today)).to_string_lossy().to_string()
            };

            generate_daily_pdf(&storage, today, &output_path)?;
            println!("PDF generated: {}", output_path);
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

        Commands::Streak { id } => {
            let today = Local::now().date_naive();
            let daily_tasks: Vec<_> = storage.list_all_tasks()?
                .into_iter()
                .filter(|t| t.is_daily)
                .collect();

            if daily_tasks.is_empty() {
                println!("No daily habits found. Add one with: daily add \"habit\" --daily");
            } else {
                let tasks_to_show: Vec<_> = if let Some(ref task_id) = id {
                    daily_tasks.iter()
                        .filter(|t| t.id.starts_with(task_id.as_str()))
                        .collect()
                } else {
                    daily_tasks.iter().collect()
                };

                println!("\n=== HABIT STREAKS ===\n");
                for task in tasks_to_show {
                    // Streak as of yesterday (today may not be done yet)
                    let yesterday = today.pred_opt().unwrap_or(today);
                    let streak = storage.get_streak_for_task(&task.id, yesterday)?;
                    let done_today = storage.is_daily_completed_on_date(&task.id, today)?;

                    let streak_label = if done_today {
                        let today_streak = storage.get_streak_for_task(&task.id, today)?;
                        format!("{} days", today_streak)
                    } else if streak > 0 {
                        format!("{} days (not done today)", streak)
                    } else {
                        "no streak yet".to_string()
                    };

                    let status = if done_today { "[done]" } else { "[todo]" };
                    println!("{} {} — Streak: {}", status, task.title, streak_label);
                }
                println!();
            }
        }

        Commands::Habits { days } => {
            let today = Local::now().date_naive();
            let daily_tasks: Vec<_> = storage.list_all_tasks()?
                .into_iter()
                .filter(|t| t.is_daily)
                .collect();

            if daily_tasks.is_empty() {
                println!("No daily habits found. Add one with: daily add \"habit\" --daily");
            } else {
                // Build date header
                println!("\n=== HABIT TRACKER (last {} days) ===\n", days);

                // Find longest title for alignment
                let max_len = daily_tasks.iter().map(|t| t.title.len()).max().unwrap_or(10);

                for task in &daily_tasks {
                    let grid = storage.get_habit_grid(&task.id, today, days)?;
                    let grid_str: String = grid.iter()
                        .map(|&done| if done { "[+]" } else { "[ ]" })
                        .collect::<Vec<_>>()
                        .join("");

                    let yesterday = today.pred_opt().unwrap_or(today);
                    let streak = storage.get_streak_for_task(&task.id, yesterday)?;
                    let done_today = storage.is_daily_completed_on_date(&task.id, today)?;
                    let current_streak = if done_today {
                        storage.get_streak_for_task(&task.id, today)?
                    } else {
                        streak
                    };

                    let streak_badge = if current_streak >= 7 {
                        format!("  {} days", current_streak)
                    } else if current_streak > 0 {
                        format!("  {} day{}", current_streak, if current_streak == 1 { "" } else { "s" })
                    } else {
                        "  —".to_string()
                    };

                    println!(
                        "{:<width$}  {}{}",
                        task.title,
                        grid_str,
                        streak_badge,
                        width = max_len
                    );
                }
                println!();
                println!("[+] = completed  [ ] = missed");
                println!();
            }
        }

        Commands::Plan { prompt } => {
            let client = ClaudeClient::new()
                .context("Failed to initialize Claude client. Make sure ANTHROPIC_API_KEY is set.")?;

            let tasks = storage.list_all_tasks()?;
            let today = Local::now().date_naive();

            println!("Planning...\n");

            let plan = client.plan_from_natural_language(&prompt, &tasks, today).await?;

            println!("{}\n", plan.explanation);

            for action in plan.actions {
                match action.action_type.as_str() {
                    "create_habit" => {
                        let task_id = storage.get_next_task_id()?;
                        let priority = action.priority.as_deref()
                            .and_then(Priority::from_str)
                            .unwrap_or(Priority::Medium);
                        let category = action.category.clone().unwrap_or_else(|| "default".to_string());

                        let mut task = Task::new(task_id, action.title.clone(), priority, category);
                        task = task.with_daily(true);

                        if let Some(days_nums) = action.scheduled_days_as_nums() {
                            task = task.with_scheduled_days(days_nums);
                        }
                        if let Some(t) = action.scheduled_time {
                            task = task.with_scheduled_time(t);
                        }
                        if let Some(loc) = action.location {
                            task = task.with_location(loc);
                        }
                        if action.two_minute.unwrap_or(false) {
                            task = task.with_two_minute(true);
                        }
                        if let Some(desc) = action.description {
                            task = task.with_description(desc);
                        }
                        if let Some(after_id) = action.habit_stack_after {
                            task = task.with_habit_stack_after(after_id);
                        }

                        storage.save_task(&task)?;

                        println!("Created habit: '{}' (ID: {})", task.title, task.id);
                        if let Some(days_str) = task.scheduled_days_display() {
                            println!("  Days: {}", days_str);
                        } else {
                            println!("  Days: every day");
                        }
                        if let Some(ref t) = task.scheduled_time {
                            println!("  Time: {}", t);
                        }
                        if let Some(ref loc) = task.location {
                            println!("  Where: {}", loc);
                        }
                        println!("  Category: {}", task.category);
                    }
                    other => {
                        println!("Unknown action type '{}' — skipping.", other);
                    }
                }
            }
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

fn generate_daily_pdf(storage: &Storage, date: NaiveDate, output_path: &str) -> Result<()> {
    use printpdf::*;
    use std::fs::File;
    use std::io::BufWriter;

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new("Daily Tasks", Mm(210.0), Mm(297.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    // Set up fonts
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let mut y_position = 270.0; // Start from top

    // Title
    current_layer.use_text(
        format!("Daily Tasks - {}", date),
        24.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 15.0;

    // Get all tasks
    let mut tasks = storage.list_all_tasks()?;

    if tasks.is_empty() {
        current_layer.use_text(
            "No tasks found.",
            12.0,
            Mm(20.0),
            Mm(y_position),
            &font,
        );
    } else {
        tasks.sort_by(|a, b| {
            a.category.cmp(&b.category)
                .then(b.priority.value().cmp(&a.priority.value()))
        });

        let mut current_category = String::new();
        for task in tasks {
            // Check if we need a new page
            if y_position < 30.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_layer = doc.get_page(page).get_layer(layer);
                y_position = 270.0;
            }

            // Category header
            if task.category != current_category {
                y_position -= 5.0;
                current_layer.use_text(
                    format!("=== {} ===", task.category.to_uppercase()),
                    14.0,
                    Mm(20.0),
                    Mm(y_position),
                    &font_bold,
                );
                y_position -= 8.0;
                current_category = task.category.clone();
            }

            // Task details
            let status = if task.completed { "[✓]" } else { "[ ]" };
            let daily_indicator = if task.is_daily { " [Daily]" } else { "" };
            let task_line = format!(
                "{} {} - {}{} (Priority: {})",
                status,
                task.id,
                task.title,
                daily_indicator,
                task.priority.to_string()
            );

            current_layer.use_text(task_line, 11.0, Mm(25.0), Mm(y_position), &font);
            y_position -= 6.0;

            // Description if present
            if let Some(desc) = &task.description {
                current_layer.use_text(
                    format!("    {}", desc),
                    9.0,
                    Mm(30.0),
                    Mm(y_position),
                    &font,
                );
                y_position -= 5.0;
            }

            // Due date if present
            if let Some(due) = &task.due_date {
                current_layer.use_text(
                    format!("    Due: {}", due.format("%Y-%m-%d")),
                    9.0,
                    Mm(30.0),
                    Mm(y_position),
                    &font,
                );
                y_position -= 5.0;
            }

            y_position -= 2.0;
        }
    }

    // Save PDF
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

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

enum DayFilter { Incomplete, Completed, All }

fn show_day_tasks(storage: &Storage, date: NaiveDate, filter: DayFilter) -> Result<()> {
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
        // Skip if already in the list
        if tasks.iter().any(|t| t.id == task.id) {
            continue;
        }

        // Include daily tasks that are scheduled for this day of the week
        if task.is_daily {
            let include = match &task.scheduled_days {
                None => true, // every day
                Some(days) => {
                    let weekday = date.weekday().num_days_from_monday() as u8;
                    days.contains(&weekday)
                }
            };
            if include {
                tasks.push(task);
            }
            continue;
        }

        // Check if the due_date matches this date
        if let Some(due) = task.due_date {
            if due.date_naive() == date {
                tasks.push(task);
            }
        }
    }

    match filter {
        DayFilter::All => {}
        DayFilter::Completed => tasks.retain(|task| {
            if task.is_daily {
                storage.is_daily_completed_on_date(&task.id, date).unwrap_or(false)
            } else {
                task.completed
            }
        }),
        DayFilter::Incomplete => tasks.retain(|task| {
            if task.is_daily {
                !storage.is_daily_completed_on_date(&task.id, date).unwrap_or(false)
            } else {
                !task.completed
            }
        }),
    }

    if tasks.is_empty() {
        println!("No tasks scheduled for this day.");
    } else {
        // Sort: daily first by time, then by category and priority
        tasks.sort_by(|a, b| {
            b.is_daily.cmp(&a.is_daily)
                .then(a.scheduled_time.cmp(&b.scheduled_time))
                .then(a.category.cmp(&b.category))
                .then(b.priority.value().cmp(&a.priority.value()))
        });

        let mut current_category = String::new();
        for task in &tasks {
            if task.category != current_category {
                println!("\n=== {} ===", task.category.to_uppercase());
                current_category = task.category.clone();
            }

            let done_today = if task.is_daily {
                storage.is_daily_completed_on_date(&task.id, date).unwrap_or(false)
            } else {
                task.completed
            };
            let status = if done_today { "[+]" } else { "[ ]" };

            // Build implementation intention hint
            let mut intention = String::new();
            if let Some(ref t) = task.scheduled_time {
                intention.push_str(&format!(" @ {}", t));
            }
            if let Some(ref loc) = task.location {
                intention.push_str(&format!(" in {}", loc));
            }

            let two_min_marker = if task.two_minute { " [2min]" } else { "" };
            let days_marker = task.scheduled_days_display()
                .map(|d| format!(" ({})", d))
                .unwrap_or_default();

            println!(
                "{} [{}] {}{}{}{}",
                status,
                task.id,
                task.title,
                intention,
                days_marker,
                two_min_marker,
            );

            // Habit stacking cue
            if let Some(ref after_id) = task.habit_stack_after {
                if let Ok(anchor) = storage.load_task(after_id) {
                    println!("     -> After: {}", anchor.title);
                }
            }

            // Streak for daily habits
            if task.is_daily {
                let yesterday = date.pred_opt().unwrap_or(date);
                let streak = storage.get_streak_for_task(&task.id, yesterday)?;
                if done_today {
                    let today_streak = storage.get_streak_for_task(&task.id, date)?;
                    if today_streak > 0 {
                        println!("     Streak: {} day{}", today_streak, if today_streak == 1 { "" } else { "s" });
                    }
                } else if streak > 0 {
                    println!("     Streak: {} day{} — keep it going!", streak, if streak == 1 { "" } else { "s" });
                }
            }
        }
        println!();
    }

    if let Some(notes) = day.notes {
        println!("Notes: {}\n", notes);
    }

    Ok(())
}
