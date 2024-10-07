use crate::commands::{ChatCommand, CommandMap};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

pub struct MostlyHelp {
    cmds: CommandMap,
}

impl MostlyHelp {
    pub fn new() -> Self {
        Self {
            cmds: HashMap::new(),
        }
    }

    // pub fn new_with_args(cmds: CommandMap) -> Self {
    //     MostlyHelp { cmds }
    // }
}

impl ChatCommand for MostlyHelp {
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
