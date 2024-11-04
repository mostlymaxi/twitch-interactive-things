//! js command
//!
//! usage: ```!js```
//!
//! author: Cathyprime

use anyhow::anyhow;
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use rand::seq::SliceRandom;
use tracing::{debug, error};
use twitcheventsub::MessageData;

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

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
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
