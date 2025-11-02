use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "daily")]
#[command(about = "A CLI task management tool with AI integration", long_about = None)]
pub struct Cli {
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
    },

    /// List tasks
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Show only incomplete tasks
        #[arg(short, long)]
        incomplete: bool,

        /// Show only completed tasks
        #[arg(short = 'C', long)]
        completed: bool,
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
        daily: bool,
    },

    /// Create a new category
    Category {
        /// Category name
        name: String,

        /// Category description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all categories
    Categories,

    /// Show tasks for today
    Today,

    /// Show tasks for a specific date
    Day {
        /// Date (YYYY-MM-DD)
        date: String,
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

    /// Interact with Claude AI
    Claude {
        /// Prompt for Claude
        prompt: String,
    },
}
