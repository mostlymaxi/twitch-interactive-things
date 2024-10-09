//! returns link to maxi's vod channel
//!
//! usage: ```!vods```
//!
//! author: mostlymaxi
use super::ChatCommand;

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

    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "check out maxi's vods on youtube!: https://www.youtube.com/@mostlyvods",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
