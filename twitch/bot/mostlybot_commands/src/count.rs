//! be lazy and have the computer count for you!
//!
//! usage: ```!count```
//!
//! author: lunispang

use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use tracing::instrument;
use twitcheventsub::MessageData;

pub struct Count(usize);

impl ChatCommand for Count {
    fn new() -> Self {
        Self(0)
    }

    fn names() -> Vec<String> {
        vec!["count".to_string()]
    }

    fn help(&self) -> String {
        "usage: !count".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> anyhow::Result<()> {
        let Self(count) = self;
        if api
            .send_chat_message(format!("current count: {count}"))
            .is_ok()
        {
            *self = Self(*count + 1);
        }
        Ok(())
    }
}
