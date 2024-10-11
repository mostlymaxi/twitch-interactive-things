//! returns maxi's kofi link
//!
//! usage: ```!kofi```
//!
//! author: mostlymaxi
use super::ChatCommand;

pub struct MostlyKofi {}

impl ChatCommand for MostlyKofi {
    fn new() -> Self {
        MostlyKofi {}
    }

    fn help(&self) -> String {
        "usage: !kofi".to_string()
    }

    fn names() -> Vec<String> {
        vec![
            "kofi".to_string(),
            "ko-fi".to_string(),
            "donate".to_string(),
        ]
    }

    fn handle(
        &mut self,
        api: &super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let _ = api.send_chat_message_with_reply(
            "buy maxi a coffee: https://ko-fi.com/mostlymaxi",
            Some(&ctx.message_id),
        );

        Ok(())
    }
}
