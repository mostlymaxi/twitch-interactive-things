use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct SpamCheck {
    // Duration within which commands are counted
    time_window: Duration,
    // Number of commands allowed in the time window
    threshold: usize,
    // User ID -> (Total Command Count, Last Timestamp)
    user_timestamps: HashMap<String, (usize, Instant)>,
}

impl SpamCheck {
    pub fn new(threshold: usize, time_window: Duration) -> Self {
        Self {
            time_window,
            threshold,
            user_timestamps: HashMap::new(),
        }
    }

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
}
