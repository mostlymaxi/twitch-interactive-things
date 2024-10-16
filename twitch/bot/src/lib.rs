pub mod api;
pub mod command;
pub mod commands;
pub mod spam;

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::{
        api::{MockTwitchEventSubApi, TwitchApiWrapper},
        command::{handle_command_if_applicable, CommandMap},
        commands::{ping, ChatCommand},
        spam::SpamManager,
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
        let mut spam_check = SpamManager::new(1, Duration::from_secs(3), 0, Duration::ZERO);

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
}
