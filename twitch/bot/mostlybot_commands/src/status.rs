//! sends current bot status
//!
//! usage: !status
//!
//! author: lunispang

use mostlybot_api::{ChatCommand, RateLimit, TwitchApiWrapper};
use twitcheventsub::MessageData;

pub struct MostlyStatus;

impl ChatCommand for MostlyStatus {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["status".to_string(), "mostlystatus".to_string()]
    }

    fn help(&self) -> String {
        "usage: !status".to_string()
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, _: &MessageData) -> anyhow::Result<()> {
        let _ = api.send_chat_message("Bot is offline");
        Ok(())
    }

    fn rate_limit(&self) -> RateLimit {
        RateLimit::new(5, std::time::Duration::from_secs(1))
    }
}
