use anyhow::Result;
use serde_json::Value;

pub mod mostlygnu;
pub mod mostlyhelp;

pub trait ChatCommand {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn handle(&mut self, args: String, ctx: Value) -> Result<String>;

    fn help(&self) -> String;
}
