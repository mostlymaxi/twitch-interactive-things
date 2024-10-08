use anyhow::Result;
use tracing::instrument;
use twitcheventsub::{EventSubError, MessageData, TwitchEventSubApi};

// ----------------------------------------------------------------------------
// add your command module here:
pub mod mostlyhelp;
pub mod mostlypasta;
pub mod ping;

// ----------------------------------------------------------------------------

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()>;

    fn help(&self) -> String;
}

pub struct MockTwitchEventSubApi {}

impl MockTwitchEventSubApi {
    pub fn init_twitch_api() -> MockTwitchEventSubApi {
        MockTwitchEventSubApi {}
    }
}

pub enum TwitchApiWrapper {
    Live(TwitchEventSubApi),
    Test(MockTwitchEventSubApi),
}

impl TwitchApiWrapper {
    fn send_chat_message<S: Into<String>>(&mut self, message: S) -> Result<String, EventSubError> {
        match self {
            Self::Live(api) => api.send_chat_message(message),
            Self::Test(_mock) => todo!(),
        }
    }

    fn send_chat_message_with_reply<S: Into<String>>(
        &mut self,
        message: S,
        reply_message_parent_id: Option<S>,
    ) -> Result<String, EventSubError> {
        match self {
            Self::Live(api) => {
                api.send_chat_message_with_reply(message, reply_message_parent_id.map(S::into))
            }
            Self::Test(_mock) => todo!(),
        }
    }
}

type CommandCell = Rc<RefCell<dyn ChatCommand>>;
#[derive(Clone)]
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
    pub fn handle_cmd(&mut self, api: &mut TwitchApiWrapper, cmd: &str, ctx: &MessageData) {
        match self.get_mut(cmd).map(|c| c.borrow_mut().handle(api, ctx)) {
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
    help.init(map.clone());
    map.insert(help);

    map
}
