use anyhow::Result;
use serde_json::Value;

pub mod mostlyhelp;
pub mod mostlypasta;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait ChatCommand: 'static {
    fn names() -> Vec<String>
    where
        Self: Sized;

    fn handle(&mut self, args: String, ctx: Value) -> Result<String>;

    fn help(&self) -> String;
}

pub type CommandMap = HashMap<String, Rc<RefCell<dyn ChatCommand>>>;

pub fn init() -> CommandMap {
    fn cmd_insert<C: ChatCommand>(map: &mut CommandMap, cmd: C) {
        let cmd = Rc::new(RefCell::new(cmd));
        for name in C::names() {
            map.insert(name.into(), Rc::clone(&cmd) as _);
        }
    }

    let mut map = HashMap::new();
    cmd_insert(&mut map, mostlypasta::MostlyPasta::new());
    cmd_insert(&mut map, mostlyhelp::MostlyHelp::new());
    map
}
