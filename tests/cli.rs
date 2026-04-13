/// Integration tests for every CLI command.
///
/// Each test uses a fresh TempDir via `--data-dir` so nothing touches ~/.daily.
use assert_cmd::Command;
use chrono::Local;
use predicates::prelude::*;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn daily(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("daily").unwrap();
    cmd.arg("--data-dir").arg(dir.path().to_str().unwrap());
    cmd
}

fn add_task(dir: &TempDir, title: &str) -> assert_cmd::assert::Assert {
    daily(dir).args(["add", title]).assert()
}

/// Add a task and return its ID from stdout.
fn add_task_get_id(dir: &TempDir, title: &str) -> String {
    let output = daily(dir).args(["add", title]).assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in stdout.lines() {
        if let Some(id) = line.strip_prefix("ID: ") {
            return id.trim().to_string();
        }
    }
    panic!("could not find ID in output:\n{}", stdout);
}

// ---------------------------------------------------------------------------
// add
// ---------------------------------------------------------------------------

#[test]
fn test_add_basic() {
    let dir = TempDir::new().unwrap();
    add_task(&dir, "Buy groceries")
        .success()
        .stdout(predicate::str::contains("Task added successfully!"))
        .stdout(predicate::str::contains("Buy groceries"));
}

#[test]
fn test_add_with_priority() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Urgent task", "-p", "critical"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Critical"));
}

#[test]
fn test_add_invalid_priority_fails() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Bad priority", "-p", "extreme"])
        .assert()
        .failure();
}

#[test]
fn test_add_with_category() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Work task", "-c", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work"));
}

#[test]
fn test_add_with_description() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Task with desc", "-d", "Some details"])
        .assert()
        .success();
}

#[test]
fn test_add_with_due_date() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Due task", "-D", "2026-12-31"])
        .assert()
        .success();
}

#[test]
fn test_add_invalid_due_date_fails() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Bad date", "-D", "not-a-date"])
        .assert()
        .failure();
}

#[test]
fn test_add_as_daily() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Daily habit", "--daily"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Daily recurring task"));
}

#[test]
fn test_add_with_scheduled_time() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Morning run", "--daily", "-t", "06:30"])
        .assert()
        .success()
        .stdout(predicate::str::contains("06:30"));
}

#[test]
fn test_add_with_location() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Yoga", "--daily", "-l", "living room"])
        .assert()
        .success()
        .stdout(predicate::str::contains("living room"));
}

#[test]
fn test_add_with_two_minute() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["add", "Open journal", "--daily", "--two-minute"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Two-minute"));
}

#[test]
fn test_add_with_habit_stack_after() {
    let dir = TempDir::new().unwrap();
    let anchor_id = add_task_get_id(&dir, "Make coffee");
    daily(&dir)
        .args(["add", "Review goals", "--daily", "--after", &anchor_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&anchor_id));
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

#[test]
fn test_list_empty() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found."));
}

#[test]
fn test_list_shows_added_task() {
    let dir = TempDir::new().unwrap();
    add_task(&dir, "My task").success();
    daily(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My task"));
}

#[test]
fn test_list_filter_by_category() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["add", "Work task", "-c", "work"]).assert().success();
    daily(&dir).args(["add", "Personal task", "-c", "personal"]).assert().success();
    daily(&dir)
        .args(["list", "-c", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Work task"))
        .stdout(predicate::str::contains("Personal task").not());
}

#[test]
fn test_list_filter_by_priority() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["add", "High task", "-p", "high"]).assert().success();
    daily(&dir).args(["add", "Low task", "-p", "low"]).assert().success();
    daily(&dir)
        .args(["list", "-p", "high"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High task"))
        .stdout(predicate::str::contains("Low task").not());
}

#[test]
fn test_list_incomplete_only() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Done task");
    add_task(&dir, "Pending task").success();
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["list", "-i"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pending task"))
        .stdout(predicate::str::contains("Done task").not());
}

#[test]
fn test_list_completed_only() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Done task");
    add_task(&dir, "Pending task").success();
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["list", "-C"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done task"))
        .stdout(predicate::str::contains("Pending task").not());
}

#[test]
fn test_list_random_returns_one_per_category() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["add", "Work A", "-c", "work"]).assert().success();
    daily(&dir).args(["add", "Work B", "-c", "work"]).assert().success();
    // With random flag on a single category, exactly 1 task should appear
    let output = daily(&dir)
        .args(["list", "-r"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1 task(s) found"));
}

// ---------------------------------------------------------------------------
// complete / uncomplete
// ---------------------------------------------------------------------------

#[test]
fn test_complete_regular_task() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Finish report");
    daily(&dir)
        .args(["complete", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("marked as complete"));
}

#[test]
fn test_complete_daily_task_shows_streak() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Morning run");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["complete", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed for"));
}

#[test]
fn test_complete_task_not_found() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["complete", "999"]).assert().failure();
}

#[test]
fn test_complete_by_id_prefix() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Prefix task");
    // Use just first character of id as prefix
    let prefix = &id[..1];
    daily(&dir)
        .args(["complete", prefix])
        .assert()
        .success();
}

#[test]
fn test_uncomplete_task() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Task to undo");
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["uncomplete", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("marked as incomplete"));
}

#[test]
fn test_uncomplete_task_not_found() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["uncomplete", "999"]).assert().failure();
}

#[test]
fn test_uncomplete_all() {
    let dir = TempDir::new().unwrap();
    let id1 = add_task_get_id(&dir, "Task 1");
    let id2 = add_task_get_id(&dir, "Task 2");
    daily(&dir).args(["complete", &id1]).assert().success();
    daily(&dir).args(["complete", &id2]).assert().success();
    daily(&dir)
        .args(["uncomplete-all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 task(s) marked as incomplete"));
}

#[test]
fn test_uncomplete_all_when_none_completed() {
    let dir = TempDir::new().unwrap();
    add_task(&dir, "Task 1").success();
    daily(&dir)
        .args(["uncomplete-all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 task(s)"));
}

// ---------------------------------------------------------------------------
// delete
// ---------------------------------------------------------------------------

#[test]
fn test_delete_task() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Delete me");
    daily(&dir)
        .args(["delete", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
    // Confirm it's gone
    daily(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found."));
}

#[test]
fn test_delete_task_not_found() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["delete", "999"]).assert().failure();
}

// ---------------------------------------------------------------------------
// priority
// ---------------------------------------------------------------------------

#[test]
fn test_priority_update() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Needs upgrade");
    daily(&dir)
        .args(["priority", &id, "critical"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Critical"));
}

#[test]
fn test_priority_invalid() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Task");
    daily(&dir)
        .args(["priority", &id, "extreme"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// move
// ---------------------------------------------------------------------------

#[test]
fn test_move_task_category() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Moveable");
    daily(&dir)
        .args(["move", &id, "personal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"));
}

// ---------------------------------------------------------------------------
// daily (toggle)
// ---------------------------------------------------------------------------

#[test]
fn test_set_task_daily() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Habit candidate");
    daily(&dir)
        .args(["daily", &id, "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daily recurring task"));
}

#[test]
fn test_unset_task_daily() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "No longer daily");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["daily", &id, "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no longer a daily"));
}

// ---------------------------------------------------------------------------
// category / categories
// ---------------------------------------------------------------------------

#[test]
fn test_create_category() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["category", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Category 'work' created!"));
}

#[test]
fn test_create_category_with_description() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["category", "health", "-d", "Physical wellbeing"])
        .assert()
        .success();
}

#[test]
fn test_create_category_with_identity() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["category", "fitness", "--identity", "I am someone who moves every day"])
        .assert()
        .success();
}

#[test]
fn test_list_categories_empty() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["categories"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No categories found."));
}

#[test]
fn test_list_categories_shows_created() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["category", "work"]).assert().success();
    daily(&dir).args(["category", "health"]).assert().success();
    daily(&dir)
        .args(["categories"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work"))
        .stdout(predicate::str::contains("health"));
}

// ---------------------------------------------------------------------------
// today / day
// ---------------------------------------------------------------------------

#[test]
fn test_today_empty() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks scheduled"));
}

#[test]
fn test_today_shows_daily_tasks() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Daily habit");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Daily habit"));
}

#[test]
fn test_today_shows_due_today() {
    let dir = TempDir::new().unwrap();
    let today = Local::now().date_naive().to_string();
    daily(&dir)
        .args(["add", "Due today", "-D", &today])
        .assert()
        .success();
    daily(&dir)
        .args(["today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Due today"));
}

#[test]
fn test_today_completed_daily_shows_done() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Done daily");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[+]"));
}

#[test]
fn test_day_empty() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["day", "2026-01-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks scheduled"));
}

#[test]
fn test_day_invalid_date_fails() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["day", "not-a-date"]).assert().failure();
}

// ---------------------------------------------------------------------------
// schedule
// ---------------------------------------------------------------------------

#[test]
fn test_schedule_task_for_date() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Scheduled task");
    daily(&dir)
        .args(["schedule", &id, "2026-06-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled for 2026-06-15"));
    daily(&dir)
        .args(["day", "2026-06-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scheduled task"));
}

#[test]
fn test_schedule_task_not_found() {
    let dir = TempDir::new().unwrap();
    daily(&dir).args(["schedule", "999", "2026-06-15"]).assert().failure();
}

// ---------------------------------------------------------------------------
// streak
// ---------------------------------------------------------------------------

#[test]
fn test_streak_no_daily_tasks() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["streak"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No daily habits found"));
}

#[test]
fn test_streak_shows_daily_task() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Morning run");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["streak"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Morning run"));
}

#[test]
fn test_streak_after_completion() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Meditate");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["streak"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[done]"));
}

#[test]
fn test_streak_specific_task_id() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Read");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["streak", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Read"));
}

// ---------------------------------------------------------------------------
// habits
// ---------------------------------------------------------------------------

#[test]
fn test_habits_no_daily_tasks() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .args(["habits"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No daily habits found"));
}

#[test]
fn test_habits_shows_grid() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Yoga");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["habits"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Yoga"))
        .stdout(predicate::str::contains("[ ]"));
}

#[test]
fn test_habits_completed_shows_plus() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Journal");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir).args(["complete", &id]).assert().success();
    daily(&dir)
        .args(["habits"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[+]"));
}

#[test]
fn test_habits_custom_days() {
    let dir = TempDir::new().unwrap();
    let id = add_task_get_id(&dir, "Run");
    daily(&dir).args(["daily", &id, "true"]).assert().success();
    daily(&dir)
        .args(["habits", "--days", "7"])
        .assert()
        .success()
        .stdout(predicate::str::contains("last 7 days"));
}

// ---------------------------------------------------------------------------
// today-pdf
// ---------------------------------------------------------------------------

#[test]
fn test_today_pdf_generates_file() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("test.pdf");
    daily(&dir)
        .args(["today-pdf", "-o", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("PDF generated"));
    assert!(out.exists());
}

// ---------------------------------------------------------------------------
// claude / plan — require API key, just verify error without it
// ---------------------------------------------------------------------------

#[test]
fn test_claude_without_api_key_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .env_remove("ANTHROPIC_API_KEY")
        .args(["claude", "What should I do today?"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY").or(predicate::str::contains("Failed")));
}

#[test]
fn test_plan_without_api_key_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    daily(&dir)
        .env_remove("ANTHROPIC_API_KEY")
        .args(["plan", "do yoga mon, wed, fri at 7am"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY").or(predicate::str::contains("Failed")));
}
