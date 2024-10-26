//! js command
//!
//! usage: ```!js```
//!
//! author: Cathyprime

use super::ChatCommand;
use anyhow::anyhow;
use rand::seq::SliceRandom;
use tracing::{debug, error};

pub struct MostlyJs {}

impl ChatCommand for MostlyJs {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["js".to_string()]
    }

    fn help(&self) -> String {
        "usage: !js".to_string()
    }

    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let msgs = vec!["Undefined", "[object Object]", "x === y"];
        let js = msgs.choose(&mut rand::thread_rng());
        let msg = format!("\"{}\" does not exist", js.unwrap());

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
