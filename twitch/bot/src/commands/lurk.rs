//! command that lets mostlymaxi know that you are lurking and will not be paying attention to the
//! stream during the term of your lurk
//!
//! usage: ```!lurk``` or ```!unlurk```
//!
//! author: bhavyakukkar

use anyhow::anyhow;
use std::collections::HashSet;
use twitcheventsub::EventSubError;

use super::ChatCommand;

const LURK_SUCCESSFUL: &str = "have a nice lurk!";
const LURK_FAILED: &str = "you are already lurking silly! to unlurk do !unlurk";
const UNLURK_SUCCESSFUL: &str = "welcome back! hope you were productive during your lurk!";
const UNLURK_FAILED: &str = "you weren't lurking but welcome back anyway!";

/// A struct holding the users that are currently lurking
pub struct Lurk {
    /// A hash-set of the ids of the users that are currently lurking
    pub users_lurking: HashSet<String>,
}

impl Lurk {
    /// Create the Lurk command starting with no users currently lurking
    pub fn new() -> Self {
        Lurk {
            users_lurking: HashSet::new(),
        }
    }
}

// added Default impl so clippy will stop complaining
impl Default for Lurk {
    fn default() -> Self {
        Self::new()
    }
}

// what to do when an api reply succeeds
fn reply_ok(s: String) {
    tracing::debug!(reply = %s);
}

// what to do when an api reply fails
fn reply_err(e: EventSubError) -> anyhow::Error {
    tracing::error!(error = ?e);
    anyhow!("{:?}", e)
}

impl ChatCommand for Lurk {
    fn new() -> Self {
        Lurk::new()
    }

    fn names() -> Vec<String> {
        vec!["lurk".to_string(), "unlurk".to_string()]
    }

    fn help(&self) -> String {
        "usage: !lurk or !unlurk".to_string()
    }

    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        // split the words in the messages
        let mut args = ctx.message.text.split_whitespace();
        // get the first word of the message which should be the !lurk or !unlurk
        let command_invoked = args.next().ok_or(anyhow!(
            "invoked lurk command without any content in the message text"
        ))?;

        // match the command: lurk or unlurk
        match command_invoked {
            "!lurk" => {
                if self.users_lurking.insert(ctx.chatter.id.clone()) {
                    // user has called !lurk while not previously lurking and will now start lurking
                    api.send_chat_message_with_reply(LURK_SUCCESSFUL, Some(&ctx.message_id))
                        .map(reply_ok)
                        .map_err(reply_err)
                } else {
                    // user called !lurk while already lurking and will just continue lurking
                    api.send_chat_message_with_reply(LURK_FAILED, Some(&ctx.message_id))
                        .map(reply_ok)
                        .map_err(reply_err)
                }
            }
            "!unlurk" => {
                if self.users_lurking.remove(&ctx.chatter.id) {
                    // user has called !unlurk while previously lurking and will now stop lurking
                    api.send_chat_message_with_reply(UNLURK_SUCCESSFUL, Some(&ctx.message_id))
                        .map(reply_ok)
                        .map_err(reply_err)
                } else {
                    // user has called !unlurk while not previously lurking and will continue not
                    // lurking
                    api.send_chat_message_with_reply(UNLURK_FAILED, Some(&ctx.message_id))
                        .map(reply_ok)
                        .map_err(reply_err)
                }
            }
            _ => Err(anyhow!(
                "invoked lurk command without first word being neither lurk nor unlurk"
            )),
        }
    }
}
