//! Shows how long the bot has been running
//!
//! usage: ```!bot-time```
//!
//! author: Nilando
use anyhow::anyhow;
use tracing::instrument;

use super::ChatCommand;
use std::time::SystemTime;

pub struct BotTime {
    start_time: SystemTime
}

impl ChatCommand for BotTime {
    fn new() -> Self {
        Self {
            start_time: SystemTime::now()
        }
    }

    fn names() -> Vec<String> {
        vec!["bottime".to_string(), "bot_time".to_string(), "bot-time".to_string()]
    }

    fn help(&self) -> String {
        "usage: !bottime".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let now = SystemTime::now();
        let seconds = now.duration_since(self.start_time)?.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;

        let msg =
            if minutes == 0 {
                format!("Bot has been running for {seconds} seconds.")
            } else if hours == 0 {
                format!("Bot has been running for {minutes} minutes.")
            } else {
                format!("Bot has been running for {hours} hours.")
            };

        match api.send_chat_message_with_reply(&msg, Some(&ctx.message_id)) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("{:?}", e))
        }
    }
}
