//! module containing all commands
use anyhow::Result;
use twitcheventsub::MessageData;

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
pub mod tictactoe;

// ----------------------------------------------------------------------------

use std::time::Duration;

pub const DEFAULT_CMD_COOLDOWN_MS: u64 = 250;
pub const DNE_ERROR_COOLDOWN_SECS: u64 = 30;
pub const GEN_ERROR_COOLDOWN_SECS: u64 = 3;

use crate::{api::TwitchApiWrapper, commandmap::CommandMap};

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn cooldown() -> Duration
    where
        Self: Sized,
    {
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
    map.insert(tictactoe::TicTacToe::new());

    // help is special
    let mut help = help::MostlyHelp::new();
    help.init(map.clone());
    map.insert(help);

    map
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::api::MockTwitchEventSubApi;

    use super::{ping, ChatCommand, CommandMap, TwitchApiWrapper};
    use serde_json::json;
    use twitcheventsub::MessageData;

    /// Helper function to create a mock chat message JSON object
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

    /// Simulates a series of chat messages to test the command handling functionality
    /// Can be run with `cargo test main -- --show-output` to display the output
    #[test]
    fn main() {
        let mut commands = CommandMap::new();
        // Add available chat commands
        commands.insert(ping::MostlyPing::new());

        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());

        // Simulate chat messages
        const BOT_ID: &str = "id_bot";

        let chat_messages = vec![
            create_chat_msg("!ping", "id_ping0"),
            create_chat_msg("!ping with args", "id_ping1"),
            create_chat_msg("!mostlypasta", "id_pasta"),
            create_chat_msg("!non_existent_command", "id_phantom"),
            create_chat_msg("ping", "id_invalid_command"),
            create_chat_msg("!ping", BOT_ID),
        ];

        for chat_msg in chat_messages {
            let chat_msg_data: MessageData = serde_json::from_value(chat_msg).unwrap();

            // same parsing and handling as `crate::handle_chat_messages`
            let text = chat_msg_data.message.text.clone();
            let mut args = text.split_whitespace();

            let Some(cmd) = args
                .next()
                .filter(|cmd| cmd.starts_with('!'))
                .and_then(|cmd| cmd.strip_prefix('!'))
                .map(|cmd| cmd.to_lowercase())
            else {
                println!("Invalid message format: {}", text);
                continue;
            };

            if chat_msg_data.chatter.id == BOT_ID {
                println!("Message sent by bot: {}", text);
                continue;
            }
            commands.handle_cmd(&mut api, &cmd, &chat_msg_data);
        }
    }
}
