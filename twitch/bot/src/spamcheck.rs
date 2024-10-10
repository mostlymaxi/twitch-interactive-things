use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct SpamCheck {
    // Duration within which commands are counted
    time_window: Duration,
    // Number of commands allowed in the time window
    threshold: usize,
    // User ID -> (Total Command Count, Last Timestamp)
    user_timestamps: HashMap<String, (usize, Instant)>,
    // Command name -> (Last executed time)
    command_cooldowns: HashMap<String, Instant>,
}

impl SpamCheck {
    pub fn new(threshold: usize, time_window: Duration) -> Self {
        Self {
            time_window,
            threshold,
            user_timestamps: HashMap::new(),
            command_cooldowns: HashMap::new(),
        }
    }

    /// Checks if the user is spamming commands globally
    pub fn check_spam(&mut self, user_id: &str) -> bool {
        let current_time = Instant::now();
        let entry = self
            .user_timestamps
            .entry(user_id.into())
            .or_insert((0, current_time));

        let (count, last_time) = entry;

        if current_time.duration_since(*last_time) > self.time_window {
            *count = 0;
            *last_time = current_time;
        }

        *count += 1;

        if *count > self.threshold {
            return true; // Spam detected
        }

        false // Not spam
    }

    /// Checks if the command is under cooldown, returning the remaining time if it is
    pub fn check_command_cooldown(
        &mut self,
        cmd_name: &str,
        cmd_cooldown: Duration,
    ) -> Option<Duration> {
        let current_time = Instant::now();
        if let Some(last_executed) = self.command_cooldowns.get(cmd_name) {
            let elapsed = current_time.duration_since(*last_executed);
            if elapsed < cmd_cooldown {
                return Some(cmd_cooldown - elapsed); // Still under cooldown
            }
        }

        // Reset the cooldown timer for this command
        self.command_cooldowns
            .insert(cmd_name.to_string(), current_time);
        None // No cooldown or cooldown expired
    }
}
