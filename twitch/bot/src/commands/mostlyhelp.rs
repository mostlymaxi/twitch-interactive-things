use crate::commands::{ChatCommand, CommandMap};
use anyhow::{anyhow, Result};
use serde_json::Value;

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

    fn handle(&mut self, args: String, _ctx: Value) -> Result<String> {
        let mut args = args.split_whitespace();
        let Some(cmd_name) = args.next() else {
            return Err(anyhow!("no arguments passed"));
        };

        if args.next().is_some() {
            return Err(anyhow!("too many arguments"));
        }

        Ok(self
            .cmds
            .get(cmd_name)
            .map(|c| c.borrow().help())
            .unwrap_or(format!("{cmd_name} does not exist")))
    }

    fn help(&self) -> String {
        r"!help <command name>".to_owned()
    }
}
