use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "daily")]
#[command(about = "A CLI task management tool with AI integration", long_about = None)]
pub struct Cli {
    /// Override the data directory (default: ~/.daily). Mainly used for testing.
    #[arg(long, global = true, hide = true)]
    pub data_dir: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Task priority (low, medium, high, critical)
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// Task category
        #[arg(short, long, default_value = "default")]
        category: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// Due date (YYYY-MM-DD)
        #[arg(short = 'D', long)]
        due: Option<String>,

        /// Daily recurring task
        #[arg(long)]
        daily: bool,

        /// [Atomic Habits] Implementation intention: scheduled time (HH:MM)
        #[arg(short = 't', long)]
        time: Option<String>,

        /// [Atomic Habits] Implementation intention: location or context (e.g. "kitchen", "gym")
        #[arg(short = 'l', long)]
        location: Option<String>,

        /// [Atomic Habits] Habit stacking: task ID this habit follows
        #[arg(long)]
        after: Option<String>,

        /// [Atomic Habits] Two-minute rule: mark as the starter version of a habit
        #[arg(long)]
        two_minute: bool,

        /// Days of the week (comma-separated: mon,tue,wed,thu,fri,sat,sun)
        #[arg(long)]
        days: Option<String>,
    },

    /// List tasks
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by priority (low, medium, high, critical)
        #[arg(short, long)]
        priority: Option<String>,

        /// Show only incomplete tasks
        #[arg(short, long)]
        incomplete: bool,

        /// Show only completed tasks
        #[arg(short = 'C', long)]
        completed: bool,

        /// Randomly select one task from each category
        #[arg(short, long)]
        random: bool,
    },

    /// Complete a task
    Complete {
        /// Task ID
        id: String,
    },

    /// Uncomplete a task
    Uncomplete {
        /// Task ID
        id: String,
    },

    /// Mark all tasks as incomplete
    UncompleteAll,

    /// Delete a task
    Delete {
        /// Task ID
        id: String,
    },

    /// Delete all tasks
    DeleteAll,

    /// Update task priority
    Priority {
        /// Task ID
        id: String,

        /// New priority (low, medium, high, critical)
        priority: String,
    },

    /// Move task to different category
    Move {
        /// Task ID
        id: String,

        /// Target category
        category: String,
    },

    /// Update task daily status
    Daily {
        /// Task ID
        id: String,

        /// Set as daily task (true/false)
        #[arg(value_parser = clap::builder::BoolishValueParser::new(), action = clap::ArgAction::Set)]
        daily: bool,
    },

    /// Create a new category
    Category {
        /// Category name
        name: String,

        /// Category description
        #[arg(short, long)]
        description: Option<String>,

        /// [Atomic Habits] Identity statement for this category (e.g. "I am someone who exercises daily")
        #[arg(short, long)]
        identity: Option<String>,
    },

    /// List all categories
    Categories,

    /// Show tasks for today
    Today {
        /// Show only completed tasks
        #[arg(short = 'c', long)]
        completed: bool,

        /// Show all tasks (including completed)
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// Show tasks for a specific date
    Day {
        /// Date (YYYY-MM-DD)
        date: String,
    },

    /// Generate PDF of today's tasks
    TodayPdf {
        /// Output file path (optional, defaults to ~/daily-YYYY-MM-DD.pdf)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Add task to a specific day
    Schedule {
        /// Task ID
        task_id: String,

        /// Date (YYYY-MM-DD)
        date: String,
    },

    /// Start the daily prompt daemon
    Daemon {
        /// Time to show daily prompt (HH:MM format, 24-hour)
        #[arg(short, long, default_value = "09:00")]
        time: String,
    },

    /// [Atomic Habits] Show streak counts for daily habits
    Streak {
        /// Task ID (optional — shows all daily tasks if omitted)
        id: Option<String>,
    },

    /// [Atomic Habits] Show visual habit grid for daily tasks
    Habits {
        /// Number of days to display (default: 21)
        #[arg(short, long, default_value = "21")]
        days: u32,
    },

    /// [Atomic Habits] Natural language habit/task planning via Claude AI
    ///
    /// Examples:
    ///   daily plan "do yoga mon, wed, fri at 7am in the living room"
    ///   daily plan "meditate every day at 6:30am after making coffee"
    ///   daily plan "read 30 minutes on weeknights at 9pm"
    Plan {
        /// Natural language description of the habit or task to create
        prompt: String,
    },

    /// Interact with Claude AI
    Claude {
        /// Prompt for Claude
        prompt: String,
    },
}
