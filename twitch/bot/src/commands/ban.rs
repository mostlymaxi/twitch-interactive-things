//! ban command
//!
//! usage: ```!ban <arguments>```
//!
//! author: Cathyprime

use super::ChatCommand;
use anyhow::anyhow;
use tracing::{debug, error};

pub struct MostlyBan {}

impl ChatCommand for MostlyBan {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["ban".to_string()]
    }

    fn help(&self) -> String {
        "usage: !ban <arguments>".to_string()
    }

    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let arg: Vec<_> = ctx.message.text.split_whitespace().skip(1).collect();
        let arg: String = arg.join(" ");

        if arg.is_empty() {
            Err(anyhow!("No argument provided"))?
        }
        let msg = format!("{} has been banned", arg.trim());

        match api.send_chat_message_with_reply(&msg, Some(&ctx.message_id)) {
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

#[cfg(test)]
mod test {

    macro_rules! message {
        ($message:tt) => {
            concat!(r###"{"broadcaster_user_id":"938429017","broadcaster_user_name":"mostlymaxi","broadcaster_user_login":"mostlymaxi","chatter_user_id":"938429017","chatter_user_name":"mostlymaxi","chatter_user_login":"mostlymaxi","message_id":"3104f083-2bdb-4d6a-bb5d-30b407876ea4","message":{"text":""###, $message, r###"","fragments":[{"type":"text","text":""###, $message, r###"","cheermote":null,"emote":null,"mention":null}]},"color":"#FF0000","badges":[{"set_id":"broadcaster","id":"1","info":""},{"set_id":"subscriber","id":"0","info":"3"}],"message_type":"text","cheer":null,"reply":null,"channel_points_custom_reward_id":null,"channel_points_animation_id":null}"###)
        }
    }

    use super::*;
    use crate::commands::{MockTwitchEventSubApi, TwitchApiWrapper};

    #[test]
    fn handle() {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyBan::new();

        let test_msg = message!("!ban @mostlymaxi");

        let ctx = serde_json::from_str(test_msg).unwrap();
        cmd.handle(&mut api, &ctx).unwrap();
    }

    #[test]
    fn handle_many() {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyBan::new();

        let test_msg = message!("!ban rust users Kappa");

        let ctx = serde_json::from_str(test_msg).unwrap();
        cmd.handle(&mut api, &ctx).unwrap();
    }

    #[test]
    fn missing_arg() {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyBan::new();

        let test_msg = message!("!ban ");

        let ctx = serde_json::from_str(test_msg).unwrap();
        assert_eq!(
            cmd.handle(&mut api, &ctx).unwrap_err().to_string(),
            "No argument provided"
        )
    }
}
