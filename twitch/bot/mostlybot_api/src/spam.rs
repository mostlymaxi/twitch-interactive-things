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

    fn is_unlimited(&self) -> bool {
        self.max_attempts == 0 || self.duration == Duration::ZERO
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
        // no cooldown if no limit, allow all attempts
        if limit.is_unlimited() {
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
    use super::{RateLimit, RateLimiter};
    use std::time::{Duration, Instant};

    const USER1: &str = "user1";
    const USER2: &str = "user2";

    /// Tests cooldown behavior, cooldown resets, and respects rate limit config
    fn test_rate_limiter<'a>(
        limiter: &mut RateLimiter<&'a str>,
        key: &'a str,
        custom_limit: Option<&RateLimit>,
    ) {
        let limit = custom_limit.unwrap_or(&limiter.default_limit).clone();
        let mut enforce_limit = || limiter.enforce_limit(key, custom_limit);

        // Expire any previous cooldown before testing
        std::thread::sleep(limit.duration);

        // Stage 1: Ensure actions are allowed up to `max_attempts`
        for i in 0..limit.max_attempts {
            assert!(
                enforce_limit().is_none(),
                "Failure in Stage 1 - Attempt {}: Expected no cooldown, but cooldown was returned",
                i + 1
            );
        }

        // Stage 2: Verify cooldown triggers once attempts exceed the limit
        let remaining_cooldown = enforce_limit();
        if limit.is_unlimited() {
            assert!(
                remaining_cooldown.is_none(),
                "Failure in Stage 2 - Expected no cooldown for an unlimited limit, but cooldown was returned"
            );
        } else {
            assert!(
                remaining_cooldown.is_some(),
                "Failure in Stage 2 - Expected cooldown after reaching limit, but no cooldown was returned"
            );

            let cooldown_time = remaining_cooldown.unwrap();
            assert!(
                cooldown_time <= limit.duration,
                "Failure in Stage 2 - Expected cooldown <= limit duration, but got {:?}",
                cooldown_time
            );
        }

        // Stage 3: Wait for cooldown to expire and verify actions can resume
        let start = Instant::now();
        while start.elapsed() < limit.duration {}

        for i in 0..limit.max_attempts {
            assert!(
                enforce_limit().is_none(),
                "Failure in Stage 3 - After cooldown reset, expected no cooldown, but cooldown was returned on attempt {}",
                i + 1
            );
        }

        // Stage 4: Ensure cooldown triggers again after hitting the limit post-reset
        let post_reset_cooldown = enforce_limit();
        if limit.is_unlimited() {
            assert!(
                post_reset_cooldown.is_none(),
                "Failure in Stage 4 - Expected no cooldown again for an unlimited limit, but cooldown was returned"
            );
        } else {
            assert!(
                post_reset_cooldown.is_some(),
                "Failure in Stage 4 - Expected cooldown to trigger after hitting rate limit post-reset, but no cooldown was returned"
            );
        }
    }

    fn get_test_limits() -> Vec<RateLimit> {
        vec![
            RateLimit::new(3, Duration::from_millis(100)),
            RateLimit::new(2, Duration::from_millis(100)),
            RateLimit::new(1, Duration::from_millis(100)),
            RateLimit::new(0, Duration::from_millis(100)),
            RateLimit::new(0, Duration::ZERO),
            RateLimit::new(1, Duration::ZERO),
            RateLimit::new(2, Duration::ZERO),
            RateLimit::new(3, Duration::ZERO),
            RateLimit::new(69, Duration::from_millis(1)),
        ]
    }

    #[test]
    fn test_rate_limiter_with_various_limits() {
        for (index, limit) in get_test_limits().into_iter().enumerate() {
            let mut limiter = RateLimiter::new(limit);

            println!("Testing limit configuration #{}: {:?}", index + 1, limit);

            // Test with multiple users and multiple invocations to ensure rate limiting applies across different cases
            test_rate_limiter(&mut limiter, USER1, None);
            test_rate_limiter(&mut limiter, USER2, None);
            test_rate_limiter(&mut limiter, USER1, None);

            test_rate_limiter(&mut limiter, USER1, None);
            test_rate_limiter(&mut limiter, USER2, None);
            test_rate_limiter(&mut limiter, USER2, None);
        }
    }

    // #[test]
    // fn test_unlimited_limit_behavior() {
    //     let limit = RateLimit::new(0, Duration::ZERO);
    //     let mut limiter = RateLimiter::new(limit);
    //     assert!(limit.is_unlimited(), "Limit should be unlimited");

    //     for _ in 0..10 {
    //         assert!(
    //             limiter.enforce_limit(USER1, None).is_none(),
    //             "Expected no cooldown for unlimited limit"
    //         );
    //     }
    // }
}
