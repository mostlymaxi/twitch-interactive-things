//! module containing all commands

#[allow(clippy::module_inception)]
pub mod commands;

// ----------------------------------------------------------------------------
// NOTE: modules must match the command you expect people to use in chat (or at least one of them).
// For example: mostlypasta -> !mostlypasta <gnu> <linux>
//
// but this does not apply to the internal struct.
//
//
// add your command module to this list:
pub mod ban;
pub mod bot_time;
pub mod count;
pub mod discord;
pub mod git;
pub mod help;
pub mod kofi;
pub mod lurk;
pub mod mostlybot;
pub mod mostlypasta;
pub mod ping;
pub mod pong;
pub mod progress;
pub mod vods;
pub mod youtube;

// ----------------------------------------------------------------------------

use crate::api::TwitchApiWrapper;
use crate::command::CommandMap;
use anyhow::Result;
use std::time::Duration;
use twitcheventsub::MessageData;

pub const DEFAULT_CMD_COOLDOWN_MS: u64 = 250;
pub const DNE_ERROR_COOLDOWN_SECS: u64 = 30;
pub const GEN_ERROR_COOLDOWN_SECS: u64 = 3;

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn cooldown(&self) -> Duration {
        Duration::from_millis(DEFAULT_CMD_COOLDOWN_MS)
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()>;

    fn help(&self) -> String;
}

pub fn init() -> CommandMap {
    let mut map = CommandMap::new();
    // most commands will just be inserted
    map.insert(mostlypasta::MostlyPasta::new());
    map.insert(ping::MostlyPing::new());
    map.insert(pong::MostlyPong::new());
    map.insert(ban::MostlyBan::new());
    map.insert(commands::MostlyCommands::new());
    map.insert(mostlybot::MostlyBot::new());
    map.insert(count::Count::new());
    map.insert(kofi::MostlyKofi::new());
    map.insert(progress::Progress::new());
    map.insert(youtube::MostlyYoutube::new());
    map.insert(git::MostlyGit::new());
    map.insert(discord::MostlyDiscord::new());
    map.insert(vods::MostlyVods::new());
    map.insert(lurk::Lurk::new());
    map.insert(bot_time::BotTime::new());

    // help is special
    let mut help = help::MostlyHelp::new();
    help.init(map.clone());
    map.insert(help);

    map
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        api::MockTwitchEventSubApi,
        command::handle_command_if_applicable,
        commands::{ping, ChatCommand, CommandMap, TwitchApiWrapper},
        spamcheck::SpamCheck,
    };
    use serde_json::json;
    use std::time::Duration;
    use twitcheventsub::MessageData;

    /// Simulates a twitch chat message
    fn create_chat_msg(cmd: &str, chatter_id: &str) -> serde_json::Value {
        json!({
            "broadcaster_user_id": "938429017",
            "broadcaster_user_name": "mostlymaxi",
            "broadcaster_user_login": "mostlymaxi",
            "chatter_user_id": format!("{chatter_id}"),
            "chatter_user_name": "mostlymaxi",
            "chatter_user_login": "mostlymaxi",
            "message_id": "3104f083-2bdb-4d6a-bb5d-30b407876ea4",
            "message": {
                "text": format!("{cmd}"),
                "fragments": [
                    {
                        "type": "text",
                        "text": "!ping",
                        "cheermote": null,
                        "emote": null,
                        "mention": null
                    }
                ]
            },
            "color": "#FF0000",
            "badges": [
                {
                    "set_id": "broadcaster",
                    "id": "1",
                    "info": ""
                },
                {
                    "set_id": "subscriber",
                    "id": "0",
                    "info": "3"
                }
            ],
            "message_type": "text",
            "cheer": null,
            "reply": null,
            "channel_points_custom_reward_id": null,
            "channel_points_animation_id": null
        })
    }

    /// Test handling of various command scenarios, including spam detection and invalid commands
    #[test]
    fn test_chat_command_handling() {
        let mut commands = CommandMap::new();
        commands.insert(ping::MostlyPing::new());

        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());

        // Allow 1 command every 3 seconds
        let mut spam_check = SpamCheck::new(1, Duration::from_secs(3));

        const BOT_ID: &str = "id_bot";

        // Create a variety of test cases to cover different command scenarios
        let chat_messages = vec![
            // Basic valid command
            create_chat_msg("!ping", "id_ping0"),
            // Valid command with arguments
            create_chat_msg("!ping with args", "id_ping1"),
            // Non-existent command
            create_chat_msg("!nonexistent", "id_phantom"),
            // Message without the command prefix
            create_chat_msg("ping", "id_invalid_no_prefix"),
            // Valid command, but sent by the bot itself
            create_chat_msg("!ping", BOT_ID),
            // Empty command (just the prefix)
            create_chat_msg("!", "id_empty_command"),
            // Command with extra spaces
            create_chat_msg("!ping    ", "id_ping_extra_spaces"),
            // Command with special characters
            create_chat_msg("!ping$@", "id_ping_special_chars"),
            // Mixed case command
            create_chat_msg("!Ping", "id_mixed_case"),
            // Spam check (with same user/chatter id)
            create_chat_msg("!ping", "id_spam"),
            create_chat_msg("!ping2", "id_spam"),
            create_chat_msg("!ping$@", "id_spam"),
            create_chat_msg("!Ping", "id_spam"),
        ];

        for (_i, chat_msg) in chat_messages.into_iter().enumerate() {
            // // Simulating time delay between commands
            // if i > 0 {
            //     std::thread::sleep(Duration::from_secs(1));
            // }
            let chat_msg: MessageData = serde_json::from_value(chat_msg).unwrap();
            handle_command_if_applicable(
                &chat_msg,
                &mut api,
                &mut commands,
                BOT_ID,
                &mut spam_check,
            );
        }
    }

    #[test]
    fn test_per_command_cooldown() {
        let cooldown = Duration::from_millis(100);
        let mut spam_check = SpamCheck::new(3, Duration::from_millis(0));

        // First time the command is executed, no cooldown should be active
        assert!(spam_check
            .check_command_cooldown("ping", cooldown)
            .is_none());

        // Immediately after, the command should be under cooldown
        assert!(spam_check
            .check_command_cooldown("ping", cooldown)
            .is_some());

        // Simulate waiting
        std::thread::sleep(cooldown);

        // Cooldown should have expired, allowing the command to be executed again
        assert!(spam_check
            .check_command_cooldown("ping", cooldown)
            .is_none());
    }

    #[test]
    fn test_spam_detection_multiple_users() {
        // Allow 1 command every 10 milliseconds
        let mut spam_check = SpamCheck::new(1, Duration::from_millis(10));

        // User 1: First command should go through (not spam)
        assert!(!spam_check.check_spam("user1"));

        // User 1: Immediate second command should be flagged as spam
        assert!(spam_check.check_spam("user1"));

        // User 2: New user, first command should go through (not spam)
        assert!(!spam_check.check_spam("user2"));

        // Simulate time passing to clear the cooldown
        std::thread::sleep(Duration::from_millis(15));

        // User 1: After waiting, command should go through again (not spam)
        assert!(!spam_check.check_spam("user1"));
    }
}
