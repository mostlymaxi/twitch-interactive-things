use anyhow::Result;
use tracing::instrument;
use twitcheventsub::{EventSubError, MessageData, TwitchEventSubApi};

pub mod mostlyhelp;
pub mod mostlypasta;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn handle<T: TwitchApiWrapper>(&mut self, api: &mut T, ctx: &MessageData) -> Result<()>;

    fn help(&self) -> String;
}

pub trait TwitchApiWrapper: 'static {
    fn send_chat_message<S: Into<String>>(&mut self, message: S) -> Result<String, EventSubError>
    where
        Self: Sized;
}

impl TwitchApiWrapper for TwitchEventSubApi {
    fn send_chat_message<S: Into<String>>(&mut self, message: S) -> Result<String, EventSubError> {
        self.send_chat_message(message.into())
    }
}

pub struct TestTwitchEventSubApi {}

impl TwitchApiWrapper for TestTwitchEventSubApi {
    fn send_chat_message<S: Into<String>>(&mut self, message: S) -> Result<String, EventSubError> {
        // check for message length
        // check for how soon the message got resent
        // etc...
        Ok(message.into())
    }
}

type CommandCell = Rc<RefCell<dyn ChatCommand>>;
pub struct CommandMap(HashMap<String, CommandCell>);

impl CommandMap {
    fn new() -> Self {
        CommandMap(HashMap::new())
    }

    fn insert<C: ChatCommand>(&mut self, cmd: C) {
        let cmd = Rc::new(RefCell::new(cmd));
        for name in C::names() {
            self.0.insert(name, Rc::clone(&cmd) as _);
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut CommandCell> {
        self.0.get_mut(key)
    }

    fn get(&self, key: &str) -> Option<&CommandCell> {
        self.0.get(key)
    }

    #[instrument(skip(self, api, ctx))]
    pub fn handle_cmd(&mut self, api: &mut TwitchEventSubApi, cmd: &str, ctx: &MessageData) {
        match self
            .get_mut(cmd)
            .map(|c| c.borrow_mut().handle::<TwitchEventSubApi>(api, ctx))
        {
            None => {
                let _ = api.send_chat_message_with_reply(
                    format!("{cmd} does not exist"),
                    Some(ctx.message_id.clone()),
                );
            }
            Some(Err(e)) => {
                let _ = api.send_chat_message_with_reply(
                    format!("err: {e}"),
                    Some(ctx.message_id.clone()),
                );
            }
            Some(Ok(())) => {}
        }
    }
}

pub fn init() -> CommandMap {
    let mut map = CommandMap::new();
    // most commands will just be inserted
    map.insert(mostlypasta::MostlyPasta::new());

    // help is special
    let mut help = mostlyhelp::MostlyHelp::new();
    // help.init(map.clone());
    map.insert(help);

    map
}
