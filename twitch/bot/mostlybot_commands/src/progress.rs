//! progress command that shows progress
//!
//! usage: ```!progress```
//!
//! author: FreeFull

use anyhow::anyhow;
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use rand::{thread_rng, Rng};
use tracing::{debug, error, instrument};
use twitcheventsub::MessageData;

pub struct Progress {}

impl ChatCommand for Progress {
    fn new() -> Self {
        Progress {}
    }

    fn names() -> Vec<String> {
        vec!["progress".to_string()]
    }

    #[instrument(skip(self, api))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        let progress = thread_rng().gen_range(0.0..100.0);
        let progress = format!("Progress: {progress:.6}% done!");
        match api.send_chat_message_with_reply(&progress, Some(&ctx.message_id)) {
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

    fn help(&self) -> String {
        "usage: !progress".to_string()
    }
}
