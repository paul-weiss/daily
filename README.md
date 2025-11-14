# Daily - CLI Task Management with AI Integration

A powerful, locally-run task management application built in Rust that combines simple CLI operations with AI-powered insights through Claude integration.

## Features

- **Easy CLI Interface**: Add, list, complete, and manage tasks from the command line
- **Task Priorities**: Four priority levels (Low, Medium, High, Critical)
- **Categories**: Organize tasks into custom categories
- **Daily Recurring Tasks**: Track habits and daily activities that repeat every day
- **Plain Text Storage**: All data stored in human-readable text files at `~/.daily/`
- **Task Scheduling**: Schedule tasks for specific dates and view daily task lists
- **Due Dates**: Set due dates for tasks and see them in daily views
- **PDF Export**: Generate printable PDFs of your task lists
- **Daily Prompts**: Automated reminders at specified times via daemon
- **Completion History**: Track task completions with timestamped logs
- **Partial ID Matching**: Reference tasks by ID prefix for faster operations
- **Claude Integration**: Get AI-powered insights and assistance with your tasks

## Installation

### Build from Source

```bash
cargo build --release
```

The binary will be available at `./target/release/daily`

### Optional: Add to PATH

```bash
# Copy to a directory in your PATH
sudo cp target/release/daily /usr/local/bin/
```

## Usage

### Task Management

#### Add a Task

```bash
# Basic task
daily add "Task title"

# With priority, category, and description
daily add "Fix bug" -p high -c development -d "Critical authentication issue"

# With due date
daily add "Submit report" -p medium -c work -D 2025-11-15

# As a daily recurring task
daily add "Morning standup" -p medium -c work --daily
```

**Priority levels**: `low`, `medium`, `high`, `critical`

**Note**: Daily recurring tasks don't get marked as "completed" permanently. Instead, each completion is logged to track your daily habits. They reappear each day until you remove them.

#### List Tasks

```bash
# List all tasks
daily list

# Filter by category
daily list -c work

# Filter by priority
daily list -p high

# Filter by both category and priority
daily list -c work -p critical

# Show only incomplete tasks
daily list -i

# Show only completed tasks
daily list -C
```

**Note**: Tasks are displayed grouped by category and sorted by priority (highest first). Tasks with ID prefixes can be referenced using partial IDs (e.g., task `abc123` can be referenced as `abc`).

#### Complete/Uncomplete Tasks

```bash
# Complete a task
daily complete 1

# Mark as incomplete
daily uncomplete 1

# Mark all tasks as incomplete
daily uncomplete-all
```

**Note**: Completing a daily recurring task logs the completion for that day but doesn't mark it as permanently complete. Regular tasks are marked as complete and logged to the history.

#### Update Task Priority

```bash
daily priority 1 critical
```

#### Move Task to Different Category

```bash
daily move 1 urgent
```

#### Toggle Daily Recurring Status

```bash
# Make a task daily recurring
daily daily 1 true

# Remove daily recurring status
daily daily 1 false
```

#### Delete a Task

```bash
daily delete 1
```

### Category Management

#### Create a Category

```bash
# Basic category
daily category work

# With description
daily category personal -d "Personal tasks and errands"
```

#### List All Categories

```bash
daily categories
```

### Daily Scheduling

#### View Today's Tasks

```bash
daily today
```

#### View Tasks for Specific Date

```bash
daily day 2025-11-01
```

#### Generate PDF of Today's Tasks

```bash
# Generate PDF with default filename (~/daily-YYYY-MM-DD.pdf)
daily today-pdf

# Generate PDF with custom path
daily today-pdf -o /path/to/output.pdf
```

#### Schedule a Task for a Date

```bash
daily schedule 1 2025-11-01
```

**Note**: The `today` and `day` commands show:
- Tasks scheduled for that specific date
- Tasks with due dates matching that date
- Daily recurring tasks that haven't been completed that day

### Daily Prompt Daemon

Start a background process that shows daily reminders:

```bash
# Default time (9:00 AM)
daily daemon

# Custom time (24-hour format)
daily daemon -t 08:30
```

### Claude AI Integration

Get AI-powered insights about your tasks:

```bash
daily claude "What should I prioritize today?"
daily claude "Suggest a better way to organize my tasks"
daily claude "Help me break down the authentication bug task"
```

**Setup**: Set your Anthropic API key as an environment variable:

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

## Data Storage

All data is stored as plain text files in `~/.daily/`:

- `tasks/` - Individual task files (one file per task)
- `days/` - Daily schedule files (one file per date)
- `categories/` - Category definitions
- `daily.log` - Completion log for daily recurring tasks
- `history.log` - Completion log for regular tasks
- `id_counter.txt` - Counter for generating unique task IDs

### Example Task File

```
id: 1
title: Fix authentication bug
priority: Critical
category: development
completed: false
created_at: 2025-10-30T20:30:38.827209+00:00
updated_at: 2025-10-30T20:30:38.827209+00:00
is_daily: false
description: Critical authentication issue
due_date: 2025-11-15T23:59:59+00:00
```

### Example Daily Task File

```
id: 2
title: Morning standup
priority: Medium
category: work
completed: false
created_at: 2025-10-30T09:00:00.000000+00:00
updated_at: 2025-10-30T09:00:00.000000+00:00
is_daily: true
description: Daily team standup meeting
```

## Project Structure

```
daily/
├── src/
│   ├── models/          # Data structures (Task, Category, Day)
│   ├── storage/         # File I/O operations
│   ├── cli/             # Command-line interface definitions
│   ├── scheduler/       # Daily prompt scheduling
│   ├── claude/          # Claude AI integration
│   └── main.rs          # Application entry point
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

## Dependencies

- **clap**: Command-line argument parsing
- **serde**: Serialization framework
- **chrono**: Date and time handling
- **tokio**: Async runtime for scheduling
- **reqwest**: HTTP client for API calls
- **anyhow**: Error handling
- **uuid**: Unique ID generation
- **dirs**: Home directory detection
- **printpdf**: PDF generation for task lists

## Development

### Run in Development Mode

```bash
cargo run -- <command>
```

### Run Tests

```bash
cargo test
```

### Check Code

```bash
cargo check
cargo clippy
```

## Examples

### Daily Workflow

```bash
# Morning: Check today's tasks
daily today

# Add a new urgent task
daily add "Review security audit" -p high -c work

# Complete a task
daily complete 1

# Get AI suggestions
daily claude "What are my highest priority tasks?"

# Schedule tasks for tomorrow
daily schedule 2 2025-10-31

# Evening: Review progress
daily list -C
```

### Weekly Planning

```bash
# Create categories for the week
daily category weekly-goals -d "Goals for this week"

# Add tasks for each day
daily add "Team standup" -p medium -c meetings
daily schedule <task-id> 2025-11-04

# Ask Claude for planning help
daily claude "Help me plan my week based on these tasks"
```

### Habit Tracking with Daily Tasks

```bash
# Create daily recurring tasks
daily add "Exercise" -p high -c health --daily
daily add "Read for 30 minutes" -p medium -c personal --daily
daily add "Review goals" -p low -c planning --daily

# Check today's tasks (includes all daily tasks)
daily today

# Complete daily tasks (they'll reappear tomorrow)
daily complete <task-id>

# View completion history
cat ~/.daily/daily.log
```

## Inspiration

This application draws inspiration from:

- **Atomic Habits** by James Clear - The philosophy of building better habits through small, consistent daily actions and making task completion visible and rewarding.
- **The Checklist Manifesto** by Atul Gawande - The power of simple checklists to manage complexity and ensure consistent execution of important tasks.

These principles guide the design of Daily: making it easy to track tasks, prioritize effectively on a daily basis, and build sustainable productivity habits.

## License

Apache 2.0

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.
