//! TODO: < short message explaining command >
//!
//! TODO: usage: ```!<name> <args>```
//!
//! TODO: author: <twitch name>

use tracing::instrument;

use super::ChatCommand;

// TODO: rename struct
pub struct Count(usize);

// TODO: implement the ChatCommand trait for your CommandStruct
impl ChatCommand for Count {
    fn new() -> Self {
        Self(0)
    }

    // TODO: add command "names"
    fn names() -> Vec<String> {
        vec!["count".to_string()]
    }

    // TODO: create useful help message
    fn help(&self) -> String {
        "usage: !<name> <args>".to_string()
    }

    // TODO: do stuff
    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let Self(count) = self;
        if api.send_chat_message(format!("current count: {count}")).is_ok() {
            *self = Self(*count + 1);
        }
        Ok(())
    }
}
