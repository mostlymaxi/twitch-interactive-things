use serde_json::Value;

pub mod mostlydebug;
pub mod mostlygnu;

pub trait ChatCommand {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<&'static str>
    where
        Self: Sized;

    fn handle(&mut self, args: String, ctx: Value) -> String;
}
