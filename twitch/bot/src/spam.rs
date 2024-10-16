use std::collections::HashMap;
use std::time::{Duration, Instant};

type CooldownDuration = Duration;
type MaxErrorCount = usize;
type CommandThreshold = usize;

// ----------------------------------------------------------------------------
pub enum SpamStatus {
    Allowed,
    OnCooldown(Duration),
}

// ----------------------------------------------------------------------------
struct UserErrorState {
    error_count: usize,
    cooldown_start: Option<Instant>,
}

// Error handling and error spam management
pub struct ErrorSpamManager {
    max_errors: MaxErrorCount,
    cooldown_duration: CooldownDuration,
    user_errors: HashMap<String, UserErrorState>,
}

impl ErrorSpamManager {
    pub fn new(max_errors: MaxErrorCount, cooldown_duration: CooldownDuration) -> Self {
        Self {
            max_errors,
            cooldown_duration,
            user_errors: HashMap::new(),
        }
    }

    /// Check if the user is under error cooldown and update their error count if not
    pub fn handle_user_error(&mut self, user_id: &str) -> SpamStatus {
        if self.max_errors == 0 {
            return SpamStatus::OnCooldown(self.cooldown_duration);
        }

        let now = Instant::now();
        let user_state = self
            .user_errors
            .entry(user_id.into())
            .or_insert(UserErrorState {
                error_count: 0,
                cooldown_start: None,
            });

        if let Some(start) = user_state.cooldown_start {
            let elapsed = now.duration_since(start);
            if elapsed < self.cooldown_duration {
                return SpamStatus::OnCooldown(self.cooldown_duration - elapsed);
            }
            // Reset after cooldown
            user_state.error_count = 0;
            user_state.cooldown_start = None;
        }

        user_state.error_count += 1;
        if user_state.error_count > self.max_errors {
            user_state.cooldown_start = Some(now);
            return SpamStatus::OnCooldown(self.cooldown_duration);
        }

        SpamStatus::Allowed
    }
}

// ----------------------------------------------------------------------------
struct UserCommandState {
    command_count: usize,
    last_command_time: Instant,
}

// General command spam management
pub struct CommandSpamManager {
    max_commands: CommandThreshold,
    time_window: CooldownDuration,
    user_command_data: HashMap<String, UserCommandState>,
}

impl CommandSpamManager {
    pub fn new(max_commands: CommandThreshold, time_window: CooldownDuration) -> Self {
        Self {
            max_commands,
            time_window,
            user_command_data: HashMap::new(),
        }
    }

    /// Check if the user is spamming commands
    pub fn check_command_spam(&mut self, user_id: &str) -> SpamStatus {
        if self.max_commands == 0 {
            return SpamStatus::OnCooldown(self.time_window);
        }

        let now = Instant::now();
        let user_state = self
            .user_command_data
            .entry(user_id.into())
            .or_insert(UserCommandState {
                command_count: 0,
                last_command_time: now,
            });

        if now.duration_since(user_state.last_command_time) > self.time_window {
            // Reset after window expiration
            user_state.command_count = 0;
            user_state.last_command_time = now;
            return SpamStatus::Allowed;
        }

        user_state.command_count += 1;
        if user_state.command_count > self.max_commands {
            return SpamStatus::OnCooldown(self.time_window);
        }

        user_state.last_command_time = now;
        SpamStatus::Allowed
    }
}

// ----------------------------------------------------------------------------
// Handles per-user, per-command cooldowns
pub struct CooldownManager {
    command_cooldowns: HashMap<(String, String), Instant>,
}

impl CooldownManager {
    pub fn new() -> Self {
        Self {
            command_cooldowns: HashMap::new(),
        }
    }

    /// Check and update the cooldown for a specific command of a user
    pub fn check_and_update_cooldown(
        &mut self,
        user_id: &str,
        command_name: &str,
        cooldown_duration: CooldownDuration,
    ) -> SpamStatus {
        let now = Instant::now();
        let key = (user_id.to_string(), command_name.to_string());

        if let Some(last_executed) = self.command_cooldowns.get(&key) {
            let elapsed = now.duration_since(*last_executed);
            if elapsed < cooldown_duration {
                return SpamStatus::OnCooldown(cooldown_duration - elapsed);
            }
        }
        // Cooldown is over, update the cooldown for this user and command
        self.command_cooldowns.insert(key, now);
        SpamStatus::Allowed
    }
}

// ----------------------------------------------------------------------------
// Centralized spam manager struct
pub struct SpamManager {
    command_spam_manager: CommandSpamManager,
    error_spam_manager: ErrorSpamManager,
    cooldown_manager: CooldownManager,
}

impl SpamManager {
    pub fn new(
        command_threshold: CommandThreshold,
        command_time_window: CooldownDuration,
        max_errors: MaxErrorCount,
        error_cooldown_duration: CooldownDuration,
    ) -> Self {
        Self {
            command_spam_manager: CommandSpamManager::new(command_threshold, command_time_window),
            error_spam_manager: ErrorSpamManager::new(max_errors, error_cooldown_duration),
            cooldown_manager: CooldownManager::new(),
        }
    }

    /// Check if a user is spamming commands
    pub fn check_user_command_spam(&mut self, user_id: &str) -> SpamStatus {
        self.command_spam_manager.check_command_spam(user_id)
    }

    /// Handle an error and check if a user is spamming errors
    pub fn handle_user_error(&mut self, user_id: &str) -> SpamStatus {
        self.error_spam_manager.handle_user_error(user_id)
    }

    /// Check if a command is under cooldown
    pub fn check_command_cooldown(
        &mut self,
        user_id: &str,
        command_name: &str,
        cooldown_duration: CooldownDuration,
    ) -> SpamStatus {
        self.cooldown_manager
            .check_and_update_cooldown(user_id, command_name, cooldown_duration)
    }
}

impl Default for SpamManager {
    fn default() -> Self {
        // 1 command in 3 seconds, 2 failed commands in 30 seconds
        Self::new(1, Duration::from_secs(3), 2, Duration::from_secs(30))
    }
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{CommandSpamManager, CooldownManager, ErrorSpamManager, SpamStatus};
    use std::thread;
    use std::time::Duration;

    const SPAM_THRESHOLD: usize = 1;
    const SPAM_COOLDOWN: Duration = Duration::from_millis(50);
    const ERR_SPAM_THRESHOLD: usize = 1;
    const ERR_SPAM_COOLDOWN: Duration = Duration::from_millis(100);

    const USER0: &str = "user0";
    const USER1: &str = "user1";
    const CMD0: &str = "command0";
    const CMD1: &str = "command1";

    #[test]
    fn test_command_cooldown() {
        let mut cm = CooldownManager::new();

        // Initial command execution should be allowed (no cooldown)
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD0, SPAM_COOLDOWN),
            SpamStatus::Allowed
        ));

        // Repeated command execution should hit the cooldown
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD0, SPAM_COOLDOWN),
            SpamStatus::OnCooldown(_)
        ));

        // Wait for the cooldown to expire
        thread::sleep(SPAM_COOLDOWN);

        // Command should be allowed again after the cooldown period
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD0, SPAM_COOLDOWN),
            SpamStatus::Allowed
        ));
    }

    #[test]
    fn test_command_spam() {
        let mut csm = CommandSpamManager::new(SPAM_THRESHOLD, SPAM_COOLDOWN);

        match SPAM_THRESHOLD {
            0 => {
                // For a threshold of 0, any command should immediately trigger cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Subsequent commands should still be under cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            1 => {
                // For a threshold of 1, the first command should be allowed
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // The next command should trigger spam detection
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Wait for the cooldown to expire
                thread::sleep(SPAM_COOLDOWN);

                // After cooldown, commands should be allowed again
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
            }
            _ => {
                // For thresholds greater than 1
                // Should not trigger spam detection until threshold is reached
                for _ in 0..SPAM_THRESHOLD {
                    assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                }

                // Exceeding the threshold should trigger detection
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Wait for the cooldown to expire
                thread::sleep(SPAM_COOLDOWN);

                // Commands should be allowed again after the cooldown period
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
            }
        }
    }

    #[test]
    fn test_failed_command_spam() {
        let mut esm = ErrorSpamManager::new(ERR_SPAM_THRESHOLD, ERR_SPAM_COOLDOWN);

        match ERR_SPAM_THRESHOLD {
            0 => {
                // For a threshold of 0, any error should immediately trigger cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Subsequent error calls should still be under cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            1 => {
                // For a threshold of 1, the first error should be allowed
                assert!(matches!(esm.handle_user_error(USER0), SpamStatus::Allowed));

                // The next error should trigger cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Further commands should still be under cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Wait for the cooldown to expire
                thread::sleep(ERR_SPAM_COOLDOWN);

                // After the cooldown, commands should be allowed again
                assert!(matches!(esm.handle_user_error(USER0), SpamStatus::Allowed));
            }
            _ => {
                // For thresholds greater than 1
                // Should not trigger cooldown until threshold is reached
                for _ in 0..ERR_SPAM_THRESHOLD {
                    assert!(matches!(esm.handle_user_error(USER0), SpamStatus::Allowed));
                }

                // After exceeding the threshold, it should trigger a cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Further commands should still be under cooldown
                assert!(matches!(
                    esm.handle_user_error(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Wait for the cooldown to expire
                thread::sleep(ERR_SPAM_COOLDOWN);

                // After the cooldown, commands should be allowed again
                assert!(matches!(esm.handle_user_error(USER0), SpamStatus::Allowed));
            }
        }
    }

    #[test]
    fn test_failed_command_spam_with_zero_threshold() {
        let mut esm = ErrorSpamManager::new(0, ERR_SPAM_COOLDOWN);

        // Immediately trigger cooldown on the first error
        assert!(matches!(
            esm.handle_user_error(USER0),
            SpamStatus::OnCooldown(_)
        ));

        // Subsequent error calls should still be under cooldown
        assert!(matches!(
            esm.handle_user_error(USER0),
            SpamStatus::OnCooldown(_)
        ));

        // Wait for the cooldown to expire
        thread::sleep(ERR_SPAM_COOLDOWN);

        // After the cooldown, errors should still be under cooldown since threshold is 0
        assert!(matches!(
            esm.handle_user_error(USER0),
            SpamStatus::OnCooldown(_)
        ));
    }

    #[test]
    fn test_cooldown_expiry_and_reset() {
        let mut csm = CommandSpamManager::new(SPAM_THRESHOLD, SPAM_COOLDOWN);

        match SPAM_THRESHOLD {
            0 => {
                // For a threshold of 0, any command should immediately trigger cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Subsequent commands should still be under cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            1 => {
                // For a threshold of 1, the first command should be allowed
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // The next command should trigger spam detection
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Wait for the cooldown to expire
                thread::sleep(SPAM_COOLDOWN);

                // After cooldown, commands should be allowed again
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // Ensure count is reset and spam detection starts fresh
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // Next command should trigger cooldown again
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            _ => {
                // For thresholds greater than 1
                // Trigger cooldown
                for _ in 0..SPAM_THRESHOLD {
                    assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                }

                // Wait for the cooldown to expire
                thread::sleep(SPAM_COOLDOWN);

                // Ensure the user is allowed to issue commands again
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // Ensure count is reset and spam detection starts fresh
                for _ in 0..SPAM_THRESHOLD {
                    assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                }

                // Next command should trigger cooldown again
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
        }
    }

    #[test]
    fn test_user_command_cooldown() {
        let mut cm = CooldownManager::new();

        // User 0's command hits cooldown
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD0, SPAM_COOLDOWN),
            SpamStatus::Allowed
        ));
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD0, SPAM_COOLDOWN),
            SpamStatus::OnCooldown(_)
        ));

        // User 1 should still be allowed to run the same command
        assert!(matches!(
            cm.check_and_update_cooldown(USER1, CMD0, SPAM_COOLDOWN),
            SpamStatus::Allowed
        ));

        // User 0 should still be able to run a different command
        assert!(matches!(
            cm.check_and_update_cooldown(USER0, CMD1, SPAM_COOLDOWN),
            SpamStatus::Allowed
        ));
    }

    #[test]
    fn test_multiple_users_command_spam() {
        let mut csm = CommandSpamManager::new(SPAM_THRESHOLD, SPAM_COOLDOWN);

        // User 0 hits threshold
        for _ in 0..SPAM_THRESHOLD {
            assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
        }
        assert!(matches!(
            csm.check_command_spam(USER0),
            SpamStatus::OnCooldown(_)
        ));

        // User 1 should still be allowed
        for _ in 0..SPAM_THRESHOLD {
            assert!(matches!(csm.check_command_spam(USER1), SpamStatus::Allowed));
        }
        // User 1 hits their threshold as well
        assert!(matches!(
            csm.check_command_spam(USER1),
            SpamStatus::OnCooldown(_)
        ));
    }

    #[test]
    fn test_rapid_fire_commands_across_time_window() {
        let mut csm = CommandSpamManager::new(SPAM_THRESHOLD, SPAM_COOLDOWN);

        match SPAM_THRESHOLD {
            0 => {
                // For a threshold of 0, any command should immediately trigger cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));

                // Subsequent commands should still be under cooldown
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            1 => {
                // For a threshold of 1, the first command should trigger cooldown
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
            _ => {
                // For thresholds greater than 1
                // Issue some commands below the threshold
                for _ in 0..(SPAM_THRESHOLD - 1) {
                    assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                }

                // Wait for the cooldown window to pass
                thread::sleep(SPAM_COOLDOWN);

                // Commands should reset and start fresh
                assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));

                // Fill the spam threshold again
                for _ in 0..SPAM_THRESHOLD {
                    assert!(matches!(csm.check_command_spam(USER0), SpamStatus::Allowed));
                }

                // The next command should trigger spam detection
                assert!(matches!(
                    csm.check_command_spam(USER0),
                    SpamStatus::OnCooldown(_)
                ));
            }
        }
    }
}
