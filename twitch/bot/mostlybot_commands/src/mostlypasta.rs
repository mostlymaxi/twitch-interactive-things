//! makes custom gnu/linux copy pasta
//!
//! usage: ```!mostlypasta <gnu> <linux>```
//!
//! author: mostlymaxi

use anyhow::{anyhow, Result};
use mostlybot_api::{ChatCommand, TwitchApiWrapper};
use twitcheventsub::MessageData;

pub struct MostlyPasta {}

impl MostlyPasta {}

impl ChatCommand for MostlyPasta {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["mostlypasta".to_owned()]
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()> {
        let mut args = ctx.message.text.split_whitespace();
        let _ = args.next();
        let gnu = args.next().ok_or(anyhow!("not enough arguments"))?;
        let linux = args.next().ok_or(anyhow!("not enough arguments"))?;

        if args.next().is_some() {
            return Err(anyhow!("too many arguments"));
        }

        let pasta = format!(
            r"
I'd just like to interject for a moment. What you're refering to as {linux}, is in fact, {gnu}/{linux}, or as I've recently taken to calling it, {gnu} plus {linux}. {linux} is not an operating system unto itself, but rather another free component of a fully functioning {gnu} system made useful by the {gnu} corelibs, shell utilities and vital system components comprising a full OS as defined by POSIX."
        );

        let _ = api.send_chat_message(pasta);
        Ok(())
    }

    fn help(&self) -> String {
        "!mostlypasta <gnu> <linux>".to_owned()
    }
}
