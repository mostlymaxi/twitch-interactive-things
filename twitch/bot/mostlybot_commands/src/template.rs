//! TODO: < short message explaining command >
//!
//! TODO: usage: ```!<name> <args>```
//!
//! TODO: author: <twitch name>

use anyhow::{anyhow, Result};
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use tracing::{debug, error, instrument};
use twitcheventsub::MessageData;

// TODO: rename struct
struct CommandStruct {}

// TODO: implement the ChatCommand trait for your CommandStruct
impl ChatCommand for CommandStruct {
    fn new() -> Self {
        Self {}
    }

    // TODO: add command "names"
    fn names() -> Vec<String> {
        vec!["<name>".to_string()]
    }

    // TODO: create useful help message
    fn help(&self) -> String {
        "usage: !<name> <args>".to_string()
    }

    // TODO: do stuff
    #[instrument(skip(self, api))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()> {
        todo!()
    }
}
