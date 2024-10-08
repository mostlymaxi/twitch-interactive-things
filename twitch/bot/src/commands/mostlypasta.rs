use std::{thread, time::Duration};

use anyhow::{anyhow, Result};
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::commands::ChatCommand;

pub struct MostlyPasta {}

impl MostlyPasta {}

impl ChatCommand for MostlyPasta {
    fn new() -> Self {
        Self {}
    }

    fn names() -> Vec<String> {
        vec!["mostlypasta".to_owned()]
    }

    fn handle(&mut self, api: &mut TwitchEventSubApi, ctx: &MessageData) -> Result<()> {
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
        api.send_chat_message(pasta).unwrap();
        thread::sleep(Duration::from_millis(250));

        let pasta = format!(
            r"
Many computer users run a modified version of the {gnu} system every day, without realizing it. Through a peculiar turn of events, the version of {gnu} which is widely used today is often called {linux}, and many of its users are not aware that it is basically the {gnu} system, developed by the {gnu} Project."
        );
        api.send_chat_message(pasta).unwrap();
        thread::sleep(Duration::from_millis(250));

        let pasta = format!(
            r"
There really is a {linux}, and these people are using it, but it is just a part of the system they use. {linux} is the kernel: the program in the system that allocates the machine's resources to the other programs that you run. The kernel is an essential part of an operating system, but useless by itself; it can only function in the context of a complete operating system."
        );
        api.send_chat_message(pasta).unwrap();
        thread::sleep(Duration::from_millis(250));

        let pasta = format!("{linux} is normally used in combination with the {gnu} operating system: the whole system is basically {gnu} with {linux} added, or {gnu}/{linux}. All the so-called {linux} distributions are really distributions of {gnu}/{linux}!");
        api.send_chat_message(pasta).unwrap();

        Ok(())
    }

    fn help(&self) -> String {
        "!mostlypasta <gnu> <linux>".to_owned()
    }
}
