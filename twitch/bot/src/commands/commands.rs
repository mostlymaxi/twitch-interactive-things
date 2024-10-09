//! returns list of commands
//!
//! usage: ```!commands```
//!
//! author: mostlymaxi

use anyhow::anyhow;
use tracing::{debug, error, instrument};

use super::ChatCommand;

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
    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
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
