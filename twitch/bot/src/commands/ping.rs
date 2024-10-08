use anyhow::anyhow;
use tracing::{debug, error, instrument};

use super::ChatCommand;

struct MostlyPing {}

impl ChatCommand for MostlyPing {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["ping".to_string()]
    }

    fn help(&self) -> String {
        "bot health check".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        match api.send_chat_message_with_reply("pong", Some(&ctx.message_id)) {
            Ok(s) => {
                debug!(ping_api_reply = %s);
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
    use crate::commands::{MockTwitchEventSubApi, TwitchApiWrapper};

    #[test]
    fn handle() {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyPing::new();

        let test_msg = r###"{"broadcaster_user_id":"938429017","broadcaster_user_name":"mostlymaxi","broadcaster_user_login":"mostlymaxi","chatter_user_id":"938429017","chatter_user_name":"mostlymaxi","chatter_user_login":"mostlymaxi","message_id":"3104f083-2bdb-4d6a-bb5d-30b407876ea4","message":{"text":"!ping","fragments":[{"type":"text","text":"!ping","cheermote":null,"emote":null,"mention":null}]},"color":"#FF0000","badges":[{"set_id":"broadcaster","id":"1","info":""},{"set_id":"subscriber","id":"0","info":"3"}],"message_type":"text","cheer":null,"reply":null,"channel_points_custom_reward_id":null,"channel_points_animation_id":null}"###;

        let ctx = serde_json::from_str(test_msg).unwrap();
        cmd.handle(&mut api, &ctx).unwrap();
    }
}
