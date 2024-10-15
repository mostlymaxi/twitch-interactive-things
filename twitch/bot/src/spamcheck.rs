use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Helps rate limit the spam of failed commands
pub struct ErrorSpamCheck {
    /// Maximum number of errors allowed before triggering a cooldown
    max_errors_allowed: usize,
    /// Duration of the error cooldown
    error_cooldown_duration: Duration,
    /// Maps user ID to the start time of their error cooldown
    user_error_cooldowns: HashMap<String, Instant>,
    /// Maps user ID to their current error count
    user_error_counts: HashMap<String, usize>,
}

impl ErrorSpamCheck {
    pub fn new(max_errors_allowed: usize, error_cooldown_duration: Duration) -> Self {
        Self {
            max_errors_allowed,
            error_cooldown_duration,
            user_error_cooldowns: HashMap::new(),
            user_error_counts: HashMap::new(),
        }
    }

    /// Updates the error cooldown state for a user, returns remaining cooldown time if active
    /// To be used only after a failed command, since this internally updates an error count
    pub fn update_error_cooldown(&mut self, user_id: &str) -> Option<Duration> {
        if self.max_errors_allowed == 0 {
            return None;
        }

        let now = Instant::now();

        if let Some(&cooldown_start) = self.user_error_cooldowns.get(user_id) {
            let elapsed = now.duration_since(cooldown_start);
            if elapsed < self.error_cooldown_duration {
                // User is still under error cooldown
                return Some(self.error_cooldown_duration - elapsed);
            } else {
                // Cooldown expired, reset error count and remove cooldown tracking
                self.user_error_counts.remove(user_id);
                self.user_error_cooldowns.remove(user_id);
            }
        }

        let error_count = self.user_error_counts.entry(user_id.into()).or_insert(0);
        *error_count += 1;

        if *error_count > self.max_errors_allowed {
            // Trigger error cooldown
            self.user_error_cooldowns.insert(user_id.into(), now);
            return Some(self.error_cooldown_duration);
        }

        None // User is not under error cooldown
    }
}

impl Default for ErrorSpamCheck {
    fn default() -> Self {
        Self::new(2, Duration::from_secs(30))
    }
}

// ----------------------------------------------------------------------------

pub struct SpamCheck {
    /// Maximum number of commands a user is allowed to send within `command_time_window`
    command_threshold: usize,
    /// Duration within which the command threshold (command spam) is measured
    command_time_window: Duration,
    /// Maps a user ID to their total command count and the last command execution time
    user_command_timestamps: HashMap<String, (usize, Instant)>,
    /// Tracks the last execution time for each command by name
    command_cooldowns: HashMap<String, Instant>,
    /// For spam control of failed commands
    pub error_spam_check: ErrorSpamCheck,
}

impl SpamCheck {
    pub fn new(
        command_threshold: usize,
        command_time_window: Duration,
        error_spam_check: ErrorSpamCheck,
    ) -> Self {
        Self {
            command_threshold,
            command_time_window,
            user_command_timestamps: HashMap::new(),
            command_cooldowns: HashMap::new(),
            error_spam_check,
        }
    }

    /// Checks if the user is spamming commands globally
    pub fn check_command_spam(&mut self, user_id: &str) -> bool {
        // Allow all commands when threshold is 0
        if self.command_threshold == 0 {
            return false;
        }

        let now = Instant::now();
        let (total_count, last_command_time) = self
            .user_command_timestamps
            .entry(user_id.into())
            .or_insert((0, now));

        if now.duration_since(*last_command_time) > self.command_time_window {
            *total_count = 0;
            *last_command_time = now;
        }

        *total_count += 1;

        *total_count > self.command_threshold // true if spamming
    }

    /// Checks and updates the command cooldown state, returns remaining cooldown
    /// time if applicable
    pub fn update_command_cooldown(
        &mut self,
        command_name: &str,
        command_cooldown_duration: Duration,
    ) -> Option<Duration> {
        let now = Instant::now();
        if let Some(last_executed) = self.command_cooldowns.get(command_name) {
            let elapsed = now.duration_since(*last_executed);
            if elapsed < command_cooldown_duration {
                // Command is still under cooldown
                return Some(command_cooldown_duration - elapsed);
            }
        }

        // Reset the cooldown timer for the command
        self.command_cooldowns.insert(command_name.into(), now);
        None // No cooldown or cooldown expired
    }
}

impl Default for SpamCheck {
    fn default() -> Self {
        Self {
            command_threshold: 1,
            command_time_window: Duration::from_secs(5),
            user_command_timestamps: HashMap::new(),
            command_cooldowns: HashMap::new(),
            error_spam_check: ErrorSpamCheck::new(2, Duration::from_secs(30)),
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{ErrorSpamCheck, SpamCheck};
    use std::thread;
    use std::time::Duration;

    const SPAM_THRESHOLD: usize = 3;
    const SPAM_COOLDOWN: Duration = Duration::from_millis(50);

    const ERR_SPAM_THRESHOLD: usize = 2;
    const ERR_SPAM_COOLDOWN: Duration = Duration::from_millis(100);

    const USER_ID: &str = "user_id";
    const COMMAND_NAME: &str = "ping";

    #[test]
    fn test_command_spam_detection() {
        let mut spam_check = SpamCheck::new(SPAM_THRESHOLD, SPAM_COOLDOWN, Default::default());
        for _ in 0..SPAM_THRESHOLD {
            assert!(!spam_check.check_command_spam(USER_ID));
        }
        thread::sleep(SPAM_COOLDOWN);
        // First command after wait, should not be spam
        assert!(!spam_check.check_command_spam(USER_ID));
    }

    #[test]
    fn test_command_cooldown() {
        let mut spam_check = SpamCheck::new(SPAM_THRESHOLD, SPAM_COOLDOWN, Default::default());

        // Should be successful
        assert!(spam_check
            .update_command_cooldown(COMMAND_NAME, SPAM_COOLDOWN)
            .is_none());
        // Should be under cooldown
        assert!(spam_check
            .update_command_cooldown(COMMAND_NAME, SPAM_COOLDOWN)
            .is_some());
        // Wait for the cooldown to expire
        thread::sleep(SPAM_COOLDOWN);
        // Should be able to execute again
        assert!(spam_check
            .update_command_cooldown(COMMAND_NAME, SPAM_COOLDOWN)
            .is_none());
    }

    #[test]
    fn test_command_with_error_cooldown() {
        let mut spam_check = ErrorSpamCheck::new(ERR_SPAM_THRESHOLD, ERR_SPAM_COOLDOWN);

        for _ in 0..ERR_SPAM_THRESHOLD {
            assert!(spam_check.update_error_cooldown(USER_ID).is_none());
        }
        // Exceeding the threshold triggers cooldown
        assert!(spam_check.update_error_cooldown(USER_ID).is_some());

        // Attempting another command with error should still be under cooldown
        assert!(spam_check.update_error_cooldown(USER_ID).is_some());

        // Wait for the cooldown to expire
        thread::sleep(ERR_SPAM_COOLDOWN);

        // Should not be under cooldown anymore
        assert!(spam_check.update_error_cooldown(USER_ID).is_none());
    }
}
