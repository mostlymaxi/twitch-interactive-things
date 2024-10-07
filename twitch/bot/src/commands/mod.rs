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

//; pub type CommandMap = HashMap<String, Rc<RefCell<dyn ChatCommand>>>;
pub struct CommandMap(HashMap<String, Rc<RefCell<dyn ChatCommand>>>);

impl CommandMap {
    fn new() -> Self {
        CommandMap(HashMap::new())
    }

    fn insert<C: ChatCommand>(&mut self, cmd: C) {
        let cmd = Rc::new(RefCell::new(cmd));
        for name in C::names() {
            self.0.insert(name.into(), Rc::clone(&cmd) as _);
        }
    }
}

pub fn init() -> CommandMap {
    let mut map = CommandMap::new();
    map.insert(mostlypasta::MostlyPasta::new());
    map.insert(mostlyhelp::MostlyHelp::new());
    map
}
