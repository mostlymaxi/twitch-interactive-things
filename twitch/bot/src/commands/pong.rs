//! command that replies to pong
//!
//! usage: ```!pong```
//!
//! author: Cathyprime

use anyhow::anyhow;
use tracing::{debug, error, instrument};

use super::ChatCommand;

pub struct MostlyPong {}

impl ChatCommand for MostlyPong {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["pong".to_string()]
    }

    fn help(&self) -> String {
        "usage: !pong".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        match api.send_chat_message_with_reply("FeelsWeirdMan", Some(&ctx.message_id)) {
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
    use super::*;
    use crate::api::{MockTwitchEventSubApi, TwitchApiWrapper};

    #[test]
    fn handle() {
        let api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyPong::new();

        let test_msg = r###"{"broadcaster_user_id":"938429017","broadcaster_user_name":"mostlymaxi","broadcaster_user_login":"mostlymaxi","chatter_user_id":"938429017","chatter_user_name":"mostlymaxi","chatter_user_login":"mostlymaxi","message_id":"3104f083-2bdb-4d6a-bb5d-30b407876ea4","message":{"text":"!pong","fragments":[{"type":"text","text":"!pong","cheermote":null,"emote":null,"mention":null}]},"color":"#FF0000","badges":[{"set_id":"broadcaster","id":"1","info":""},{"set_id":"subscriber","id":"0","info":"3"}],"message_type":"text","cheer":null,"reply":null,"channel_points_custom_reward_id":null,"channel_points_animation_id":null}"###;

        let ctx = serde_json::from_str(test_msg).unwrap();
        cmd.handle(&api, &ctx).unwrap();
    }
}
