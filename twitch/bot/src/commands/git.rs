//! returns link to maxi's git
//!
//! usage: ```!git```
//!
//! author: mostlymaxi
use super::ChatCommand;

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

    fn handle(
        &mut self,
        api: &super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "check out maxi's git: https://github.com/mostlymaxi",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
