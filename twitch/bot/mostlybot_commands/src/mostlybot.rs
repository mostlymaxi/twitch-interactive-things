//! get link to mostlybot git
//!
//! usage: ```!mostlybot```
//!
//! author: mostlymaxi

use anyhow::anyhow;
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use tracing::{debug, error, instrument};
use twitcheventsub::MessageData;

pub struct MostlyBot {}

impl ChatCommand for MostlyBot {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["mostlybot".to_string(), "bot".to_string()]
    }

    fn help(&self) -> String {
        "usage: !mostlybot".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        match api.send_chat_message_with_reply(
            "contribute to the mostlybot here!: https://github.com/mostlymaxi/twitch-interactive-things/tree/main/twitch/bot",
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
