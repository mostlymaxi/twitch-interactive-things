//! returns link to maxi's vod channel
//!
//! usage: ```!vods```
//!
//! author: mostlymaxi
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use twitcheventsub::MessageData;

pub struct MostlyVods {}

impl ChatCommand for MostlyVods {
    fn new() -> Self {
        MostlyVods {}
    }

    fn help(&self) -> String {
        "usage: !vods".to_string()
    }

    fn names() -> Vec<String> {
        vec!["vods".to_string(), "vod".to_string()]
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "check out maxi's vods on youtube!: https://www.youtube.com/@mostlyvods",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
