//! special help command that gets info on all other commands
//!
//! usage: ```!help <command>```
//!
//! author: mostlymaxi

use crate::commands::{ChatCommand, CommandMap};
use anyhow::{anyhow, Result};
use tracing::instrument;
use twitcheventsub::MessageData;

use super::TwitchApiWrapper;

pub const HELP_COOLDOWN_SECS: u64 = 3;

pub struct MostlyHelp {
    cmds: CommandMap,
}

impl MostlyHelp {
    /// the help command is special in that it needs access to the command
    /// map to call the help function on a specified command
    pub fn init(&mut self, cmds: CommandMap) {
        self.cmds = cmds;
    }
}

impl ChatCommand for MostlyHelp {
    fn new() -> Self {
        Self {
            cmds: CommandMap::new(),
        }
    }

    fn names() -> Vec<String> {
        vec!["help".to_owned()]
    }

    #[instrument(skip(self, api, ctx))]
    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()> {
        let mut args = ctx.message.text.split_whitespace();
        let _ = args.next();

        let Some(cmd_name) = args.next() else {
            let _ = api.send_chat_message_with_reply(
                format!("usage: {}", self.help()),
                Some(ctx.message_id.clone()),
            );
            let _ = api.send_chat_message_with_reply(
                "[WARN] if you're looking for the list of commands try: !commands",
                Some(&ctx.message_id),
            );

            return Ok(());
        };

        if args.next().is_some() {
            return Err(anyhow!("too many arguments"));
        }

        self.cmds.get_mut(cmd_name).map(|c| c.help(api, ctx));

        Ok(())
    }

    fn help(&self) -> String {
        r"!help <command name>".to_owned()
    }
}
