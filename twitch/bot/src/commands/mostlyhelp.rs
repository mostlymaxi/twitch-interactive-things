use anyhow::{anyhow, Result};
use serde_json::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ChatCommand;

pub struct MostlyHelp {
    cmds: HashMap<String, Rc<RefCell<dyn ChatCommand>>>,
}

impl MostlyHelp {
    pub fn new_with_args(cmds: HashMap<String, Rc<RefCell<dyn ChatCommand>>>) -> Self {
        MostlyHelp { cmds }
    }
}

impl ChatCommand for MostlyHelp {
    fn new() -> MostlyHelp {
        MostlyHelp {
            cmds: HashMap::new(),
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
