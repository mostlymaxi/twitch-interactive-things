use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use tracing::instrument;
use twitcheventsub::MessageData;

use crate::{
    api::TwitchApiWrapper,
    commands::{
        help::HELP_COOLDOWN_SECS, ChatCommand, DNE_ERROR_COOLDOWN_SECS, GEN_ERROR_COOLDOWN_SECS,
    },
};

type CommandCell = Rc<RefCell<dyn ChatCommand>>;

#[derive(Clone)]
pub struct CommandData {
    cooldown: Duration,
    help_cooldown: Duration,
    last_called: Instant,
    last_help: Instant,
    cmd: CommandCell,
}

impl CommandData {
    fn new(cmd: CommandCell, cooldown: Duration) -> Self {
        Self {
            cmd,
            last_called: Instant::now(),
            last_help: Instant::now(),
            cooldown,
            help_cooldown: Duration::from_secs(HELP_COOLDOWN_SECS),
        }
    }

    pub fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()> {
        if self.last_called.elapsed() < self.cooldown {
            return Err(anyhow!(
                "command is on cooldown, slow down there partner..."
            ));
        }

        let res = self.cmd.borrow_mut().handle(api, ctx);
        // we make last_called after handle in case the command
        // runs for a long time
        self.last_called = Instant::now();

        res
    }

    pub fn help(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) {
        if self.last_help.elapsed() < self.help_cooldown {
            return;
        }

        let _ = api.send_chat_message_with_reply(
            format!("usage: {}", self.cmd.borrow().help()),
            Some(ctx.message_id.clone()),
        );

        self.last_help = Instant::now();
    }
}

#[derive(Clone)]
pub struct CommandMap {
    inner: HashMap<String, CommandData>,
    last_cmd: Instant,
}

impl Default for CommandMap {
    fn default() -> Self {
        CommandMap {
            inner: HashMap::new(),
            last_cmd: Instant::now(),
        }
    }
}

impl CommandMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<C: ChatCommand>(&mut self, cmd: C) {
        let cmd = Rc::new(RefCell::new(cmd));
        for name in C::names() {
            self.inner
                .insert(name, CommandData::new(Rc::clone(&cmd) as _, C::cooldown()));
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut CommandData> {
        self.inner.get_mut(key)
    }

    pub fn get(&self, key: &str) -> Option<&CommandData> {
        self.inner.get(key)
    }

    #[instrument(skip(self, api, ctx))]
    pub fn handle_cmd(&mut self, api: &mut TwitchApiWrapper, cmd: &str, ctx: &MessageData) {
        // we're going to swallow debug messages if commands are being sent
        // more often than every x seconds... hopefully this is a good
        // balance of spam and debugging
        match self.get_mut(cmd).map(|c| c.handle(api, ctx)) {
            None => {
                if self.last_cmd.elapsed() > Duration::from_secs(DNE_ERROR_COOLDOWN_SECS) {
                    let _ = api.send_chat_message_with_reply(
                        format!("{cmd} does not exist, find the list of existing commands with !commands"),
                        Some(ctx.message_id.clone()),
                    );
                }
            }
            Some(Err(e)) => {
                if self.last_cmd.elapsed() > Duration::from_secs(GEN_ERROR_COOLDOWN_SECS) {
                    let _ = api.send_chat_message_with_reply(
                        format!("[ERROR] {e}, try !help {cmd} for more details"),
                        Some(ctx.message_id.clone()),
                    );
                }
            }
            Some(Ok(())) => {}
        }

        self.last_cmd = Instant::now();
    }
}
