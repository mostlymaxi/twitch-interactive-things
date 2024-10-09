//! module containing all commands
use anyhow::Result;
use tracing::instrument;
use twitcheventsub::{EventSubError, MessageData, TwitchEventSubApi};

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
pub mod count;
pub mod help;
pub mod kofi;
pub mod mostlybot;
pub mod mostlypasta;
pub mod ping;
pub mod pong;
pub mod progress;

// ----------------------------------------------------------------------------

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()>;

    fn help(&self) -> String;
}

pub struct MockTwitchEventSubApi {}

impl MockTwitchEventSubApi {
    pub fn init_twitch_api() -> MockTwitchEventSubApi {
        MockTwitchEventSubApi {}
    }
}

pub enum TwitchApiWrapper {
    Live(TwitchEventSubApi),
    Test(MockTwitchEventSubApi),
}

impl TwitchApiWrapper {
    fn send_chat_message<S: Into<String>>(&mut self, message: S) -> Result<String, EventSubError> {
        match self {
            Self::Live(api) => api.send_chat_message(message),
            Self::Test(_mock) => todo!(),
        }
    }

    fn send_chat_message_with_reply<S: Into<String>>(
        &mut self,
        message: S,
        reply_message_parent_id: Option<S>,
    ) -> Result<String, EventSubError> {
        match self {
            Self::Live(api) => {
                api.send_chat_message_with_reply(message, reply_message_parent_id.map(S::into))
            }
            Self::Test(_mock) => {
                match reply_message_parent_id {
                    Some(id) => println!("@{} {}", id.into(), message.into()),
                    None => println!("MockApi: {}", message.into()),
                }
                Ok(String::new())
            }
        }
    }
}

type CommandCell = Rc<RefCell<dyn ChatCommand>>;
#[derive(Clone)]
pub struct CommandMap(HashMap<String, CommandCell>);

impl CommandMap {
    fn new() -> Self {
        CommandMap(HashMap::new())
    }

    fn insert<C: ChatCommand>(&mut self, cmd: C) {
        let cmd = Rc::new(RefCell::new(cmd));
        for name in C::names() {
            self.0.insert(name, Rc::clone(&cmd) as _);
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut CommandCell> {
        self.0.get_mut(key)
    }

    fn get(&self, key: &str) -> Option<&CommandCell> {
        self.0.get(key)
    }

    #[instrument(skip(self, api, ctx))]
    pub fn handle_cmd(&mut self, api: &mut TwitchApiWrapper, cmd: &str, ctx: &MessageData) {
        match self.get_mut(cmd).map(|c| c.borrow_mut().handle(api, ctx)) {
            None => {
                let _ = api.send_chat_message_with_reply(
                    format!("{cmd} does not exist"),
                    Some(ctx.message_id.clone()),
                );
            }
            Some(Err(e)) => {
                let _ = api.send_chat_message_with_reply(
                    format!("err: {e}"),
                    Some(ctx.message_id.clone()),
                );
            }
            Some(Ok(())) => {}
        }
    }
}

pub fn init() -> CommandMap {
    let mut map = CommandMap::new();
    // most commands will just be inserted
    map.insert(mostlypasta::MostlyPasta::new());
    map.insert(ping::MostlyPing::new());
    map.insert(pong::MostlyPong::new());
    map.insert(commands::MostlyCommands::new());
    map.insert(mostlybot::MostlyBot::new());
    map.insert(count::Count::new());
    map.insert(kofi::MostlyKofi::new());
    map.insert(progress::Progress::new());

    // help is special
    let mut help = help::MostlyHelp::new();
    help.init(map.clone());
    map.insert(help);

    map
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::{ping, ChatCommand, CommandMap, MockTwitchEventSubApi, TwitchApiWrapper};
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
