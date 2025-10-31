# Daily - CLI Task Management with AI Integration

A powerful, locally-run task management application built in Rust that combines simple CLI operations with AI-powered insights through Claude integration.

## Features

- **Easy CLI Interface**: Add, list, complete, and manage tasks from the command line
- **Task Priorities**: Four priority levels (Low, Medium, High, Critical)
- **Categories**: Organize tasks into custom categories
- **Plain Text Storage**: All data stored in human-readable text files at `~/.daily/`
- **Daily Scheduling**: Structure each day with scheduled tasks
- **Daily Prompts**: Automated reminders at specified times
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
```

**Priority levels**: `low`, `medium`, `high`, `critical`

#### List Tasks

```bash
# List all tasks
daily list

# Filter by category
daily list -c work

# Show only incomplete tasks
daily list -i

# Show only completed tasks
daily list -C
```

#### Complete/Uncomplete Tasks

```bash
# Complete a task (use full ID or first 8 characters)
daily complete 6c6229cd

# Mark as incomplete
daily uncomplete 6c6229cd
```

#### Update Task Priority

```bash
daily priority 6c6229cd critical
```

#### Move Task to Different Category

```bash
daily move 6c6229cd urgent
```

#### Delete a Task

```bash
daily delete 6c6229cd
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

#### Schedule a Task for a Date

```bash
daily schedule 6c6229cd 2025-11-01
```

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

- `tasks/` - Individual task files
- `days/` - Daily schedule files
- `categories/` - Category definitions

### Example Task File

```
id: 6c6229cd-7bfa-4903-8ff0-c6a56e447f9e
title: Fix authentication bug
priority: Critical
category: development
completed: false
created_at: 2025-10-30T20:30:38.827209+00:00
updated_at: 2025-10-30T20:30:38.827209+00:00
description: Critical authentication issue
due_date: 2025-11-15T23:59:59+00:00
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
daily complete 6c6229cd

# Get AI suggestions
daily claude "What are my highest priority tasks?"

# Schedule tasks for tomorrow
daily schedule 982cc507 2025-10-31

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

## License

MIT

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.
