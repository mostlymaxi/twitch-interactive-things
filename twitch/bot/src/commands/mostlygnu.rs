use crate::ChatCommand;

pub struct MostlyGnu {}

impl ChatCommand for MostlyGnu {
    fn new() -> Self {
        MostlyGnu {}
    }

    fn names() -> Vec<String> {
        vec!["gnu".to_owned()]
    }

    fn handle(&mut self, args: String, ctx: serde_json::Value) -> String {
        "asdf".to_owned()
    }
}
