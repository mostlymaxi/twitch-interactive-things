//! returns link to the SPARCL discord
//!
//! usage: ```!discord```
//!
//! author: mostlymaxi
use super::ChatCommand;

pub struct MostlyDiscord {}

impl ChatCommand for MostlyDiscord {
    fn new() -> Self {
        MostlyDiscord {}
    }

    fn help(&self) -> String {
        "usage: !discord".to_string()
    }

    fn names() -> Vec<String> {
        vec!["discord".to_string(), "disc".to_string()]
    }

    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "join the SPARCL discord: https://discord.gg/aMAAbZy4QD",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
