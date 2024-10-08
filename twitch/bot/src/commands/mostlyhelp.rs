use crate::commands::{ChatCommand, CommandMap};
use anyhow::{anyhow, Result};
use tracing::instrument;
use twitcheventsub::MessageData;

use super::TwitchApiContext;

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

impl<T: TwitchApiContext> ChatCommand<T> for MostlyHelp {
    fn new() -> Self {
        Self {
            cmds: CommandMap::new(),
        }
    }

    fn names() -> Vec<String> {
        vec!["help".to_owned()]
    }

    #[instrument(skip(self, api, ctx))]
    fn handle(&mut self, api: &mut T, ctx: &MessageData) -> Result<()> {
        let mut args = ctx.message.text.split_whitespace();
        let _ = args.next();

        let Some(cmd_name) = args.next() else {
            return Err(anyhow!("no arguments passed"));
        };

        if args.next().is_some() {
            return Err(anyhow!("too many arguments"));
        }

        let help_msg = self
            .cmds
            .get(cmd_name)
            .map(|c| c.borrow().help())
            .unwrap_or(format!("{cmd_name} does not exist"));

        tracing::debug!(help_msg = %help_msg);

        api.send_chat_message_with_reply(help_msg, None).unwrap();

        Ok(())
    }

    fn help(&self) -> String {
        r"!help <command name>".to_owned()
    }
}
