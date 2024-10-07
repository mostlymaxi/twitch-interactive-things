use anyhow::Result;
use serde_json::Value;

pub mod mostlyhelp;
pub mod mostlypasta;

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
