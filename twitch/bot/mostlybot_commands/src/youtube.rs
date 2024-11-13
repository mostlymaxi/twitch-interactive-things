//! returns link to maxi's youtube channel
//!
//! usage: ```!youtube```
//!
//! author: mostlymaxi
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use twitcheventsub::MessageData;

pub struct MostlyYoutube {}

impl ChatCommand for MostlyYoutube {
    fn new() -> Self {
        MostlyYoutube {}
    }

    fn help(&self) -> String {
        "usage: !youtube".to_string()
    }

    fn names() -> Vec<String> {
        vec!["youtube".to_string(), "yt".to_string()]
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "check out maxi's youtube!: https://www.youtube.com/@mostlymaxi",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
