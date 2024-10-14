//! command that lets mostlymaxi know that you are lurking and will not be paying attention to the
//! stream during the term of your lurk
//!
//! usage: ```!lurk``` or ```!lurkwith <status>``` or ```!unlurk``` or ```!lurker [@username]``` or
//! ```!lurkers```
//!
//! author: bhavyakukkar

use anyhow::anyhow;
use std::collections::HashMap;
use twitcheventsub::EventSubError;

use super::ChatCommand;

mod replies {
    pub const LURK_SUCCESSFUL: &str = "have a nice lurk!";

    // make sure to keep the @ which will get replaced with chatter's name
    pub const LURK_FAILED: &str =
        "you are already lurking silly! to unlurk do `!unlurk` or view your lurk-status with \
        `!lurker @`";

    pub const LURK_STATUS_UPDATED: &str = "lurk status successfully updated!";

    // make sure to keep the % which will get replaced with what was chatter's lurking status
    pub const UNLURK_SUCCESSFUL: &str = "welcome back! hope you were productive %";

    pub const UNLURK_FAILED: &str = "you weren't lurking but welcome back anyway!";

    // make sure to keep the @ which will get replaced with chatter's name and % which will get
    // replaced with chatter's lurking status
    pub const LURKED_SUCCESSFUL: &str = "@ is lurking %";

    pub const LURKER_FAILED: &str = "you're not lurking";
}

type Username = String;
type LurkStatus = Option<String>;

/// A struct holding the users that are currently lurking
pub struct Lurk {
    /// A hash-map of the usernames of the users that are currently lurking mapped to their
    /// lurk-status
    pub users_lurking: HashMap<Username, LurkStatus>,
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
        Lurk {
            users_lurking: HashMap::new(),
        }
    }

    fn names() -> Vec<String> {
        vec![
            "lurk".to_string(),
            "lurkwith".to_string(),
            "unlurk".to_string(),
            "lurker".to_string(),
            "lurkers".to_string(),
        ]
    }

    fn help(&self) -> String {
        "usage: `!lurk` or `!lurkwith <status>` or `!unlurk` or `!lurker [@username]` or `!lurkers`"
            .to_string()
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
                match self.users_lurking.insert(ctx.chatter.name.clone(), None) {
                    // called !lurk while not previously lurking and will now start lurking
                    None => api
                        .send_chat_message_with_reply(
                            replies::LURK_SUCCESSFUL,
                            Some(&ctx.message_id),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                    // called !lurk while already lurking with or without a status, will continue
                    // lurking
                    Some(_) => api
                        .send_chat_message_with_reply(
                            &replies::LURK_FAILED.replace("@", &ctx.chatter.name),
                            Some(&ctx.message_id),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                }
            }
            "!lurkwith" => {
                // join the rest of the words in the message into the lurk-status
                let status: String = args.fold(String::new(), |a, b| a + " " + b);

                match self
                    .users_lurking
                    .insert(ctx.chatter.name.clone(), Some(status))
                {
                    // called !lurkwith while not previously lurking and will now start lurking
                    None => api
                        .send_chat_message_with_reply(
                            replies::LURK_SUCCESSFUL,
                            Some(&ctx.message_id),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                    // called !lurkwith while already lurking with or without a status, will
                    // continue lurking but with an updated status
                    Some(_) => api
                        .send_chat_message_with_reply(
                            replies::LURK_STATUS_UPDATED,
                            Some(&ctx.message_id),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                }
            }
            "!unlurk" => {
                match self.users_lurking.remove(&ctx.chatter.name) {
                    // called !unlurk while previously lurking and will now stop lurking
                    Some(previous_status) => api
                        .send_chat_message_with_reply(
                            &replies::UNLURK_SUCCESSFUL.replace(
                                "%",
                                &previous_status.unwrap_or("during your lurk!".to_string()),
                            ),
                            Some(&ctx.message_id),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                    // called !unlurk while not previously lurking and will continue not lurking
                    None => api
                        .send_chat_message_with_reply(replies::UNLURK_FAILED, Some(&ctx.message_id))
                        .map(reply_ok)
                        .map_err(reply_err),
                }
            }
            "!lurker" => {
                // join the rest of the words in the message into the username
                let mut username: String = args.fold(String::new(), |a, b| a + " " + b);

                // if no username provided, assume chatter's name as username
                if username.is_empty() {
                    username = ctx.chatter.name.clone();
                }

                // if username starts with @, remove it
                let mut chars = username.chars();
                if chars.next().ok_or(anyhow!("chatter.name received empty"))? == '@' {
                    username = chars.collect();
                }

                match self.users_lurking.get(&username) {
                    // called !lurker for username that is lurking with or without a status
                    Some(status) => api
                        .send_chat_message(
                            replies::LURKED_SUCCESSFUL
                                .replace("@", &("@".to_string() + &username))
                                .replace(
                                    "%",
                                    &status.clone().unwrap_or("with no status".to_string()),
                                ),
                        )
                        .map(reply_ok)
                        .map_err(reply_err),
                    // called !lurker for username that is not currently lurking
                    None => api
                        .send_chat_message(replies::LURKER_FAILED)
                        .map(reply_ok)
                        .map_err(reply_err),
                }
            }
            "!lurkers" => api
                .send_chat_message(
                    "Lurkers: ".to_string()
                        + &self
                            .users_lurking
                            .keys()
                            .map(|username| "@".to_string() + username)
                            .reduce(|a, b| a + ", " + &b)
                            .unwrap_or("<none>".to_string()),
                )
                .map(reply_ok)
                .map_err(reply_err),

            _ => Err(anyhow!(
                "invoked lurk command without first word being any of the commands {:?}",
                Self::names()
            )),
        }
    }
}
