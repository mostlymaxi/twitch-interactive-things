//! returns list of commands
//!
//! usage: ```!commands```
//!
//! author: mostlymaxi

use anyhow::anyhow;
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use tracing::{debug, error, instrument};
use twitcheventsub::MessageData;

pub struct MostlyCommands {}

impl ChatCommand for MostlyCommands {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["commands".to_string(), "cmds".to_string()]
    }

    fn help(&self) -> String {
        "usage: !commands".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        match api.send_chat_message_with_reply(
            "https://docs.rs/mostlybot/latest/mostlybot/commands/index.html",
            Some(&ctx.message_id),
        ) {
            Ok(s) => {
                debug!(reply = %s);
                Ok(())
            }
            Err(e) => {
                error!(error = ?e);
                Err(anyhow!("{:?}", e))
            }
        }
    }
}
