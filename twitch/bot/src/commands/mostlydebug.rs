use serde_json::Value;

use crate::ChatCommand;

pub struct MostlyDebug {}

impl ChatCommand for MostlyDebug {
    fn new() -> Self {
        MostlyDebug {}
    }

    fn names() -> Vec<String> {
        vec!["debug".to_owned(), "dbg".to_owned()]
    }

    fn handle(&mut self, args: String, ctx: Value) -> String {
        "asdf".to_owned()
    }
}
