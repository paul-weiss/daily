# Daily - CLI Task Management with AI Integration

A locally-run task management tool built in Rust that combines simple CLI operations with AI-powered insights and habit-building principles from *Atomic Habits* by James Clear.

## Features

- **Easy CLI Interface**: Add, list, complete, and manage tasks from the command line
- **Task Priorities**: Four priority levels (Low, Medium, High, Critical)
- **Categories with Identity**: Organize tasks into categories anchored to identity statements
- **Daily Recurring Tasks**: Track habits that repeat every day
- **Atomic Habits Integration**: Implementation intentions, habit stacking, two-minute rule, streak tracking, and visual habit grids
- **Plain Text Storage**: All data stored in human-readable text files at `~/.daily/`
- **Task Scheduling**: Schedule tasks for specific dates and view daily task lists
- **Due Dates**: Set due dates for tasks and see them in daily views
- **PDF Export**: Generate printable PDFs of your task lists
- **Daily Prompts**: Automated reminders at specified times via daemon
- **Completion History**: Track task completions with timestamped logs
- **Partial ID Matching**: Reference tasks by ID prefix for faster operations
- **Claude Integration**: Get AI-powered insights and assistance with your tasks

---

## Atomic Habits Integration

*Atomic Habits* by James Clear argues that lasting behavior change comes from four laws: make it obvious, make it attractive, make it easy, and make it satisfying. Each law is directly supported by features in this tool.

### Law 1: Make It Obvious

The most reliable way to form a habit is to give it a specific time and place. This is called an **implementation intention**: "I will [behavior] at [time] in [location]." The `--time` and `--location` flags encode this directly into the task.

```bash
daily add "Morning run" --daily -t 06:30 -l "front door"
daily add "Read" --daily -t 21:00 -l "armchair"
daily add "Meditate" --daily -t 07:00 -l "bedroom floor"
```

The `today` view surfaces these cues automatically:

```
=== HEALTH ===
[ ] [3] Morning run @ 06:30 in front door
     Streak: 4 days — keep it going!
[ ] [4] Meditate @ 07:00 in bedroom floor
     Streak: 12 days — keep it going!
```

**Habit stacking** links a new habit to an existing anchor. The cue for the new habit is the completion of the previous one ("After I [current habit], I will [new habit]").

```bash
# First establish the anchor habit
daily add "Make coffee" --daily

# Stack new habits after it
daily add "Review goals" --daily --after <make-coffee-task-id>
daily add "Journal" --daily --after <review-goals-task-id>
```

The `today` view shows the chain:

```
[ ] [1] Make coffee
[ ] [2] Review goals
     -> After: Make coffee
[ ] [5] Journal
     -> After: Review goals
```

### Law 2: Make It Attractive

You are more likely to sustain a habit when it is tied to the person you want to become. Categories support an **identity statement** that is echoed back each time you complete a habit in that category.

```bash
daily category health --identity "I am someone who prioritizes their body every day"
daily category learning --identity "I am someone who grows a little every day"
daily category deep-work --identity "I am someone who does focused, meaningful work"
```

When you complete a daily habit in that category:

```
Daily task 'Morning run' completed for 2026-04-13!
Streak: 8 days — don't break the chain!
Identity: I am someone who prioritizes their body every day
```

This shifts the motivation from outcome ("I want to lose weight") to identity ("I am a healthy person"), which James Clear identifies as the most durable form of motivation.

### Law 3: Make It Easy

The two-minute rule: when starting a new habit, begin with a version that takes two minutes or less. The goal is to show up, not to perform. Once showing up is automatic, extend the habit naturally.

```bash
# Two-minute starter versions
daily add "Put on running shoes" --daily --two-minute -l "front door" -t 06:30
daily add "Open book" --daily --two-minute -t 21:00 -l "armchair"
daily add "Sit on meditation cushion" --daily --two-minute -t 07:00

# Eventual full versions (replace the two-minute habit once it's automatic)
daily add "Run 5K" --daily -l "front door" -t 06:30
```

Two-minute tasks are marked `[2min]` in the `today` view so they stand out as entry points, not endpoints.

```
[ ] [3] Put on running shoes @ 06:30 in front door [2min]
```

### Law 4: Make It Satisfying

The final law is about immediate reward. Humans repeat behaviors that feel good right away. Two commands make progress visible:

**`daily streak`** — shows the current streak for every daily habit at a glance:

```
=== HABIT STREAKS ===

[done] Morning run      — Streak: 8 days
[done] Meditate         — Streak: 12 days
[todo] Read             — Streak: 3 days (not done today)
[todo] Journal          — Streak: 0 day (not done today)
```

**`daily habits`** — shows a visual grid of the last N days ("don't break the chain"):

```
=== HABIT TRACKER (last 21 days) ===

Morning run    [+][+][+][ ][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+]  20 days
Meditate       [+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+]  21 days
Read           [ ][ ][+][+][ ][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+]  19 days
Journal        [ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][ ][+][+][+]  3 days

[+] = completed  [ ] = missed
```

The visual chain makes gaps painful and completions rewarding — which is exactly the feedback loop habits need.

---

## Installation

### Build from Source

```bash
cargo build --release
```

The binary will be available at `./target/release/daily`.

### Optional: Add to PATH

```bash
sudo cp target/release/daily /usr/local/bin/
```

---

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

Daily recurring tasks are not permanently marked complete. Each completion is logged so streaks and grids can be computed. They reappear in `today` each day.

#### Add a Daily Habit with Atomic Habits Principles

```bash
# Implementation intention (when + where)
daily add "Meditate" --daily -t 07:00 -l "bedroom floor"

# Habit stacking (after an anchor)
daily add "Review goals" --daily --after <anchor-task-id>

# Two-minute rule (starter version)
daily add "Open journal" --daily --two-minute -t 21:00

# All together
daily add "Morning run" --daily -t 06:30 -l "front door" -c health
```

**Flags:**

| Flag | Description |
|------|-------------|
| `-t, --time HH:MM` | Implementation intention: scheduled time |
| `-l, --location <place>` | Implementation intention: location or context |
| `--after <task-id>` | Habit stacking: task ID this habit follows |
| `--two-minute` | Two-minute rule: marks the starter version of a habit |

#### List Tasks

```bash
# List all tasks
daily list

# Filter by category
daily list -c work

# Filter by priority
daily list -p high

# Show only incomplete tasks
daily list -i

# Show only completed tasks
daily list -C

# Random task from each category (for variety)
daily list -r
```

#### Complete / Uncomplete Tasks

```bash
# Complete a task (daily tasks log a streak entry)
daily complete 1

# Mark as incomplete
daily uncomplete 1

# Mark all tasks as incomplete
daily uncomplete-all
```

#### Other Task Operations

```bash
# Update priority
daily priority 1 critical

# Move to a different category
daily move 1 urgent

# Toggle daily recurring status
daily daily 1 true
daily daily 1 false

# Delete a task
daily delete 1
```

---

### Category Management

#### Create a Category

```bash
# Basic category
daily category work

# With description
daily category personal -d "Personal tasks and errands"

# With identity statement (Atomic Habits: Make it Attractive)
daily category health --identity "I am someone who prioritizes their body every day"
daily category learning --identity "I am someone who grows a little every day"
```

#### List All Categories

```bash
daily categories
```

---

### Habit Tracking

#### View Streaks

```bash
# All daily habits
daily streak

# Specific habit
daily streak <task-id>
```

Output:
```
=== HABIT STREAKS ===

[done] Morning run   — Streak: 8 days
[todo] Read          — Streak: 3 days (not done today)
```

#### View Visual Habit Grid

```bash
# Default: last 21 days
daily habits

# Custom window
daily habits --days 7
daily habits --days 30
```

Output:
```
=== HABIT TRACKER (last 21 days) ===

Morning run    [+][+][+][ ][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+]  20 days
Meditate       [+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+]  21 days

[+] = completed  [ ] = missed
```

---

### Daily Schedule

#### View Today's Tasks

```bash
daily today
```

The `today` view groups habits by category and shows:
- Implementation intentions (time and location)
- Habit stacking cues (→ After: ...)
- Two-minute markers (`[2min]`)
- Live streak counts

#### View Tasks for a Specific Date

```bash
daily day 2025-11-01
```

#### Generate PDF of Today's Tasks

```bash
# Default filename (~/daily-YYYY-MM-DD.pdf)
daily today-pdf

# Custom path
daily today-pdf -o /path/to/output.pdf
```

#### Schedule a Task for a Date

```bash
daily schedule 1 2025-11-01
```

---

### Daily Prompt Daemon

Start a background process that shows daily reminders:

```bash
# Default: 9:00 AM
daily daemon

# Custom time (24-hour format)
daily daemon -t 08:30
```

---

### Natural Language Habit Planning

Use `plan` to describe a habit in plain English and have Claude create it for you with all the right fields set — days of the week, time, location, category, and habit stacking.

```bash
daily plan "do yoga mon, wed, fri at 7am in the living room"
daily plan "meditate every day at 6:30am, after making coffee"
daily plan "read 30 minutes on weeknights at 9pm in the armchair"
daily plan "go for a run every morning at 6am"
daily plan "journal on weekends at 8am"
```

Claude interprets your request and creates the habit with all fields populated:

```
Planning...

Creating a yoga habit on Monday, Wednesday, and Friday at 7am in the living room.

Created habit: 'Yoga' (ID: 5)
  Days: Mon, Wed, Fri
  Time: 07:00
  Where: living room
  Category: health
```

The habit will then appear in `daily today` only on its scheduled days, and your streaks track only the days it is scheduled.

**Supported phrasings:**
- Specific days: "mon, wed, fri" / "monday, wednesday, friday" / "weekdays" / "weekends"
- Time: "at 7am", "at 14:30", "in the morning"
- Location: "in the gym", "at my desk", "outside"
- Every day: "every day", "daily" (or just omit days)
- Habit stacking: "after making coffee" (references an existing task by title)

**Setup**: Set your Anthropic API key:

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

---

### Claude AI Chat

Get AI-powered insights about your tasks:

```bash
daily claude "What should I prioritize today?"
daily claude "Suggest a better way to organize my tasks"
daily claude "Help me design a habit stack for my morning routine"
```

---

## Recommended Workflows

### Starting a New Habit (Atomic Habits approach)

```bash
# 1. Create a category with an identity statement
daily category fitness --identity "I am someone who moves their body every day"

# 2. Start with the two-minute version
daily add "Put on workout clothes" -c fitness --daily --two-minute -t 06:30 -l "bedroom"

# 3. Once that's automatic (~2 weeks), replace with the real habit
daily add "30-minute run" -c fitness --daily -t 06:30 -l "front door"

# 4. Stack related habits
daily add "Post-run stretch" -c fitness --daily --after <run-task-id>

# 5. Track the chain
daily habits
```

### Morning Routine

```bash
# Morning check-in
daily today

# Complete habits as you do them
daily complete <task-id>   # Streak feedback shown automatically

# See where you stand
daily streak
```

### Weekly Review

```bash
# See the full habit grid for the past week
daily habits --days 7

# Ask Claude to help reflect
daily claude "Based on my habits, what patterns do you see and what should I focus on?"

# Plan ahead
daily schedule <task-id> 2026-04-20
```

### Building a Habit Stack

```bash
# Anchor: existing habit (coffee)
daily add "Make coffee" --daily -t 07:00

# Stack new habits in sequence
daily add "Review goals" --daily --after <coffee-id> -t 07:05
daily add "Read for 10 minutes" --daily --after <goals-id> -t 07:10

# View the stack in today
daily today
# ->  [ ] Make coffee @ 07:00
#     [ ] Review goals @ 07:05
#          -> After: Make coffee
#     [ ] Read for 10 minutes @ 07:10
#          -> After: Review goals
```

---

## Data Storage

All data is stored as plain text files in `~/.daily/`:

| Path | Contents |
|------|----------|
| `tasks/` | One file per task |
| `days/` | One file per scheduled date |
| `categories/` | Category definitions |
| `daily.log` | Daily habit completion log (used for streaks) |
| `history.log` | Regular task completion log |
| `id_counter.txt` | Auto-incrementing task ID counter |

### Example Task File

```
id: 3
title: Morning run
priority: High
category: fitness
completed: false
created_at: 2026-04-01T06:00:00+00:00
updated_at: 2026-04-01T06:00:00+00:00
is_daily: true
two_minute: false
scheduled_time: 06:30
location: front door
scheduled_days: 0,2,4
```

`scheduled_days` stores weekday numbers: 0=Mon, 1=Tue, 2=Wed, 3=Thu, 4=Fri, 5=Sat, 6=Sun.
Omitting it means the habit runs every day.

### Example Category File

```
name: fitness
description: Physical health habits
identity: I am someone who prioritizes their body every day
```

---

## Project Structure

```
daily/
├── src/
│   ├── models/          # Data structures (Task, Category, Day)
│   ├── storage/         # File I/O, streak computation, habit grids
│   ├── cli/             # Command-line interface definitions
│   ├── scheduler/       # Daily prompt scheduling
│   ├── claude/          # Claude AI integration
│   └── main.rs          # Application entry point and command handlers
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

---

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
- **rand**: Random task selection

---

## Development

```bash
# Run in development mode
cargo run -- <command>

# Run tests
cargo test

# Check and lint
cargo check
cargo clippy
```

---

## Inspiration

**Atomic Habits** by James Clear is the core philosophy behind the habit-building features in this tool. The four laws of behavior change map directly to the feature set:

| Law | Principle | Feature |
|-----|-----------|---------|
| Make it Obvious | Implementation intentions, habit stacking | `--time`, `--location`, `--after` |
| Make it Attractive | Identity-based motivation | `--identity` on categories |
| Make it Easy | Reduce friction to starting | `--two-minute` flag |
| Make it Satisfying | Immediate, visible reward | `streak`, `habits` grid |

The core insight: you do not rise to the level of your goals, you fall to the level of your systems. This tool is a system.

---

## License

Apache 2.0

## Contributing

Contributions welcome. Please feel free to submit a Pull Request.
