use std::collections::HashMap;
use std::time::{Duration, Instant};

// ----------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct RateLimit {
    limit: usize,
    duration: Duration,
}

impl RateLimit {
    pub const fn new(limit: usize, duration: Duration) -> Self {
        Self { limit, duration }
    }
}

// ----------------------------------------------------------------------------

struct Cooldown {
    attempt_count: usize,
    last_reset: Instant,
}

impl Cooldown {
    fn new() -> Self {
        Self {
            attempt_count: 0,
            last_reset: Instant::now(),
        }
    }

    /// Returns the remaining cooldown duration if the rate limit is exceeded
    fn update(&mut self, rate_limit: &RateLimit) -> Option<Duration> {
        if rate_limit.limit == 0 {
            // If the limit is 0, no cooldown
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_reset);

        if elapsed >= rate_limit.duration {
            // Reset attempt count if the cooldown expired
            self.attempt_count = 1;
            self.last_reset = now;
            return None;
        }

        if self.attempt_count < rate_limit.limit {
            self.attempt_count += 1;
            return None;
        }

        Some(rate_limit.duration - elapsed)
    }
}

// ----------------------------------------------------------------------------

struct CooldownTracker<K> {
    rate_limit: RateLimit,
    cooldowns: HashMap<K, Cooldown>,
}

impl<K: Eq + std::hash::Hash> CooldownTracker<K> {
    fn new(rate_limit: RateLimit) -> Self {
        Self {
            rate_limit,
            cooldowns: HashMap::new(),
        }
    }

    fn get_and<R>(&mut self, key: K, f: impl FnOnce(&RateLimit, &mut Cooldown) -> R) -> R {
        let cooldown = self.cooldowns.entry(key).or_insert(Cooldown::new());
        f(&self.rate_limit, cooldown)
    }
}

// ----------------------------------------------------------------------------

pub struct Spam {
    /// Rate limiting per-user for any command
    user_limiter: CooldownTracker<String>,
    /// Rate limiting per-command for all users
    global_command_limiter: CooldownTracker<String>,
    /// Rate limiting per-user for failed command error messages of any command
    failed_command_limiter: CooldownTracker<String>,
}

impl Spam {
    pub fn new(
        user_limit: RateLimit,
        global_command_limit: RateLimit,
        failed_command_limit: RateLimit,
    ) -> Self {
        Self {
            user_limiter: CooldownTracker::new(user_limit),
            global_command_limiter: CooldownTracker::new(global_command_limit),
            failed_command_limiter: CooldownTracker::new(failed_command_limit),
        }
    }

    /// Returns the remaining cooldown for the given user if applicable
    /// (Checks if the command is being used by THE USER too quickly)
    pub fn update_user_cooldown(&mut self, user_id: &str) -> Option<Duration> {
        self.user_limiter
            .get_and(user_id.into(), |rate_limit, cooldown| {
                cooldown.update(rate_limit)
            })
    }

    /// Returns the remaining cooldown for the given command if applicable
    /// (Checks if the command is being used by ANY USER too quickly)
    pub fn update_global_command_cooldown(
        &mut self,
        command_name: &str,
        rate_limit: &RateLimit,
    ) -> Option<Duration> {
        self.global_command_limiter
            .get_and(command_name.into(), |_, cooldown| {
                cooldown.update(rate_limit)
            })
    }

    /// Returns the remaining cooldown of the failed command for the given user if applicable
    /// (Checks if the failed command message is being encountered by THE USER too quickly)
    pub fn update_failed_command_cooldown(&mut self, user_id: &str) -> Option<Duration> {
        self.failed_command_limiter
            .get_and(user_id.into(), |rate_limit, cooldown| {
                cooldown.update(rate_limit)
            })
    }
}

impl Default for Spam {
    fn default() -> Self {
        Self::new(
            // max any 1 bot command per-user every 5 seconds
            RateLimit::new(1, Duration::from_secs(5)),
            // max any 1 bot command by any user every 5 seconds
            // use any sensible defaults, will be overridden by the user provided value
            RateLimit::new(1, Duration::from_secs(5)),
            // max any 2 failed command messages in chat, per-user, every 30 seconds
            RateLimit::new(2, Duration::from_secs(30)),
        )
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::{RateLimit, Spam};
    use std::thread::sleep;
    use std::time::Duration;

    const USER_RATE_LIMIT: RateLimit = RateLimit::new(3, Duration::from_millis(50));
    const GLOBAL_COMMAND_RATE_LIMIT: RateLimit = USER_RATE_LIMIT;
    const FAILED_COMMAND_RATE_LIMIT: RateLimit = RateLimit::new(2, Duration::from_millis(100));

    const USER_A: &str = "user_a";
    const USER_B: &str = "user_b";
    const CMD_A: &str = "cmd_a";

    /// Helper function to execute cooldown behavior for a rate limit
    fn test_cooldown(
        rate_limit: &RateLimit,
        mut cooldown_update: impl FnMut() -> Option<Duration>,
    ) {
        // Reset cooldown
        sleep(rate_limit.duration);

        // Ensure the action is allowed for the number of attempts equal to the rate limit
        for i in 0..rate_limit.limit {
            assert!(
                cooldown_update().is_none(),
                "Expected no cooldown on attempt {}, but cooldown was returned",
                i + 1
            );
        }

        // Ensure cooldown triggers on the next attempt after the limit
        let remaining_cooldown = cooldown_update();
        assert!(
            remaining_cooldown.is_some(),
            "Expected cooldown after hitting the rate limit, but none was returned"
        );

        // Ensure the cooldown period works correctly by checking remaining time
        assert!(
            remaining_cooldown.unwrap() <= rate_limit.duration,
            "Cooldown should be less than or equal to the rate limit duration"
        );

        // Sleep for the cooldown duration
        sleep(rate_limit.duration);

        // Ensure cooldown has expired and actions can resume
        for i in 0..rate_limit.limit {
            assert!(
                cooldown_update().is_none(),
                "Expected no cooldown after reset, but cooldown was returned on attempt {}",
                i + 1
            );
        }

        // Ensure cooldown triggers again after hitting the limit
        assert!(
            cooldown_update().is_some(),
            "Expected cooldown to trigger again after hitting rate limit"
        );
    }

    #[test]
    fn test_user_rate_limiter() {
        let mut spam = Spam::new(
            USER_RATE_LIMIT,
            GLOBAL_COMMAND_RATE_LIMIT,
            FAILED_COMMAND_RATE_LIMIT,
        );

        // Test for USER_A
        test_cooldown(&USER_RATE_LIMIT, || spam.update_user_cooldown(USER_A));
        // Test for USER_B, ensure independent user limits
        test_cooldown(&USER_RATE_LIMIT, || spam.update_user_cooldown(USER_B));
        // Re-test USER_A to ensure state is maintained between tests
        test_cooldown(&USER_RATE_LIMIT, || spam.update_user_cooldown(USER_A));
    }

    #[test]
    fn test_global_command_rate_limiter() {
        let mut spam = Spam::new(
            USER_RATE_LIMIT,
            GLOBAL_COMMAND_RATE_LIMIT,
            FAILED_COMMAND_RATE_LIMIT,
        );
        test_cooldown(&GLOBAL_COMMAND_RATE_LIMIT, || {
            spam.update_global_command_cooldown(CMD_A, &GLOBAL_COMMAND_RATE_LIMIT)
        });
    }

    #[test]
    fn test_zero_limit() {
        let rate_limit = RateLimit::new(0, Duration::from_millis(50));
        let mut spam = Spam::new(rate_limit, rate_limit, rate_limit);

        // No cooldown with zero limit
        for _ in 0..5 {
            assert!(
                spam.update_user_cooldown(USER_A).is_none(),
                "Expected no cooldown when limit is 0, but got some"
            )
        }
    }
}
