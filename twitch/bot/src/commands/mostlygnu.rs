use crate::ChatCommand;

pub struct MostlyGnu {}

impl ChatCommand for MostlyGnu {
    fn new() -> Self {
        MostlyGnu {}
    }

    fn names() -> Vec<&'static str> {
        vec!["gnu"]
    }

    fn handle(&mut self, args: String, ctx: serde_json::Value) -> String {
        "asdf".to_owned()
    }
}
