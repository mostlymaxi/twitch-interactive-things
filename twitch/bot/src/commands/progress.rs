//! progress command that shows progress
//!
//! usage: ```!progress```
//!
//! author: FreeFull

use anyhow::anyhow;
use rand::{thread_rng, Rng};
use tracing::{debug, error, instrument};

use super::ChatCommand;

pub struct Progress {}

impl ChatCommand for Progress {
    fn new() -> Self {
        Progress {}
    }

    fn names() -> Vec<String> {
        vec!["progress".to_string()]
    }

    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
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
