use serde_json::Value;

use crate::ChatCommand;

pub struct MostlyDebug {}

impl ChatCommand for MostlyDebug {
    fn new() -> Self {
        MostlyDebug {}
    }

    fn names() -> Vec<&'static str> {
        vec!["debug", "dbg"]
    }

    fn handle(&mut self, args: String, ctx: Value) -> String {
        "asdf".to_owned()
    }
}
