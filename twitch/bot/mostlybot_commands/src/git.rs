//! returns link to maxi's git
//!
//! usage: ```!git```
//!
//! author: mostlymaxi
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use twitcheventsub::MessageData;

pub struct MostlyGit {}

impl ChatCommand for MostlyGit {
    fn new() -> Self {
        MostlyGit {}
    }

    fn help(&self) -> String {
        "usage: !git".to_string()
    }

    fn names() -> Vec<String> {
        vec!["git".to_string(), "github".to_string()]
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "check out maxi's git: https://github.com/mostlymaxi",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
