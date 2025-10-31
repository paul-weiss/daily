use anyhow::Result;
use chrono::{Local, NaiveTime, Timelike};
use tokio::time::{sleep, Duration};
use std::process::Command;

pub struct Scheduler {
    target_time: NaiveTime,
}

impl Scheduler {
    pub fn new(time_str: &str) -> Result<Self> {
        let target_time = NaiveTime::parse_from_str(time_str, "%H:%M")?;
        Ok(Self { target_time })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let now = Local::now();
            let current_time = now.time();

            // Calculate seconds until target time
            let seconds_until_target = if current_time < self.target_time {
                // Target time is later today
                let target_seconds = self.target_time.num_seconds_from_midnight();
                let current_seconds = current_time.num_seconds_from_midnight();
                target_seconds - current_seconds
            } else {
                // Target time is tomorrow
                let seconds_remaining_today = 86400 - current_time.num_seconds_from_midnight();
                let target_seconds = self.target_time.num_seconds_from_midnight();
                seconds_remaining_today + target_seconds
            };

            // Sleep until target time
            sleep(Duration::from_secs(seconds_until_target as u64)).await;

            // Show daily prompt
            self.show_daily_prompt()?;

            // Sleep a bit to avoid triggering multiple times
            sleep(Duration::from_secs(60)).await;
        }
    }

    fn show_daily_prompt(&self) -> Result<()> {
        println!("\n=== Daily Task Prompt ===");
        println!("Good morning! Here are your tasks for today.");
        println!("\nRun 'daily today' to see your tasks.");
        println!("Run 'daily add \"task name\"' to add a new task.");
        println!("========================\n");

        // On macOS, we can use osascript to show a notification
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("osascript")
                .arg("-e")
                .arg("display notification \"Time to review your daily tasks!\" with title \"Daily Task Manager\"")
                .output();
        }

        Ok(())
    }
}
