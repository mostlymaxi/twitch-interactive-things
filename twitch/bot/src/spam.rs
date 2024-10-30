use std::collections::HashMap;
use std::time::{Duration, Instant};

// Rate Limiter ---------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct RateLimit {
    max_attempts: usize,
    duration: Duration,
}

impl RateLimit {
    pub const fn new(max_attempts: usize, duration: Duration) -> Self {
        Self {
            max_attempts,
            duration,
        }
    }
}

struct UsageState {
    attempts: usize,
    last_reset: Instant,
}

impl UsageState {
    fn new() -> Self {
        Self {
            attempts: 0,
            last_reset: Instant::now(),
        }
    }
}

struct RateLimiter<K> {
    default_limit: RateLimit,
    usage: HashMap<K, UsageState>,
}

impl<K: Eq + std::hash::Hash> RateLimiter<K> {
    fn new(default_limit: RateLimit) -> Self {
        Self {
            default_limit,
            usage: HashMap::new(),
        }
    }

    /// Enforces the rate limit for a key, returning the remaining cooldown if the limit is exceeded
    fn enforce_limit(&mut self, key: K, custom_limit: Option<&RateLimit>) -> Option<Duration> {
        let limit = custom_limit.unwrap_or(&self.default_limit);
        // no cooldown if no attempt limit, allow all attempts
        if limit.max_attempts == 0 {
            return None;
        }

        let state = self.usage.entry(key).or_insert(UsageState::new());
        let elapsed = state.last_reset.elapsed();

        // if the cooldown period has expired
        if elapsed >= limit.duration {
            state.attempts = 1;
            state.last_reset = Instant::now();
            return None;
        }

        // if we're within the cooldown period
        if state.attempts < limit.max_attempts {
            state.attempts += 1;
            return None;
        }

        Some(limit.duration - elapsed)
    }
}

// Spam Handler ---------------------------------------------------------------

type UserId = String;
type CommandId = String;

pub struct Spam {
    user_limiter: RateLimiter<UserId>,
    global_command_limiter: RateLimiter<CommandId>,
    failed_command_limiter: RateLimiter<UserId>,
}

impl Spam {
    pub fn new(
        user_limit: RateLimit,
        global_command_limit: RateLimit,
        failed_command_limit: RateLimit,
    ) -> Self {
        Self {
            user_limiter: RateLimiter::new(user_limit),
            global_command_limiter: RateLimiter::new(global_command_limit),
            failed_command_limiter: RateLimiter::new(failed_command_limit),
        }
    }

    /// Checks the cooldown for failed commands per user and returns remaining time if limit exceeded
    pub fn check_failed_command_cooldown(&mut self, user_id: &UserId) -> Option<Duration> {
        self.failed_command_limiter
            .enforce_limit(user_id.into(), None)
    }

    /// Checks the cooldown for commands per user and returns remaining time if limit exceeded
    pub fn check_user_command_cooldown(&mut self, user_id: &UserId) -> Option<Duration> {
        self.user_limiter.enforce_limit(user_id.into(), None)
    }

    /// Checks the cooldown for a global command and returns remaining time if limit exceeded
    pub fn check_global_command_cooldown(
        &mut self,
        command: &CommandId,
        custom_limit: Option<&RateLimit>,
    ) -> Option<Duration> {
        self.global_command_limiter
            .enforce_limit(command.into(), custom_limit)
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

// Tests ----------------------------------------------------------------------

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
    fn test_cooldown(limit: &RateLimit, mut cooldown_update: impl FnMut() -> Option<Duration>) {
        // Reset cooldown
        sleep(limit.duration);

        // Ensure the action is allowed for the number of attempts equal to the rate limit
        for i in 0..limit.max_attempts {
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
            remaining_cooldown.unwrap() <= limit.duration,
            "Cooldown should be less than or equal to the rate limit duration"
        );

        // Sleep for the cooldown duration
        sleep(limit.duration);

        // Ensure cooldown has expired and actions can resume
        for i in 0..limit.max_attempts {
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
        test_cooldown(&USER_RATE_LIMIT, || {
            spam.check_user_command_cooldown(&USER_A.into())
        });
        // Test for USER_B, ensure independent user limits
        test_cooldown(&USER_RATE_LIMIT, || {
            spam.check_user_command_cooldown(&USER_B.into())
        });
        // Re-test USER_A to ensure state is maintained between tests
        test_cooldown(&USER_RATE_LIMIT, || {
            spam.check_user_command_cooldown(&USER_A.into())
        });
    }

    #[test]
    fn test_global_command_rate_limiter() {
        let mut spam = Spam::new(
            USER_RATE_LIMIT,
            GLOBAL_COMMAND_RATE_LIMIT,
            FAILED_COMMAND_RATE_LIMIT,
        );
        test_cooldown(&GLOBAL_COMMAND_RATE_LIMIT, || {
            spam.check_global_command_cooldown(&CMD_A.into(), Some(&GLOBAL_COMMAND_RATE_LIMIT))
        });
    }

    #[test]
    fn test_zero_limit() {
        let test_zero = |limit| {
            let mut spam = Spam::new(limit, limit, limit);
            // No cooldown with zero limit
            for _ in 0..5 {
                assert!(
                    spam.check_user_command_cooldown(&USER_A.into()).is_none(),
                    "Expected no cooldown when limit is 0, but got some"
                )
            }
        };

        test_zero(RateLimit::new(0, Duration::from_millis(100)));
        test_zero(RateLimit::new(1, Duration::ZERO));
        test_zero(RateLimit::new(2, Duration::ZERO));
        test_zero(RateLimit::new(0, Duration::ZERO));
    }
}
