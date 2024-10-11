use crate::{api::TwitchApiWrapper, commands::ChatCommand, spamcheck::SpamCheck};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
    time::Duration,
};
use tracing::instrument;
use twitcheventsub::MessageData;

pub enum CommandParseResult {
    NotACommand,
    InvalidCommand,
    ValidCommand(String, Vec<String>),
}

// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Command {
    inner: Rc<RefCell<dyn ChatCommand>>,
}

impl std::panic::RefUnwindSafe for Command {}

impl Command {
    const PREFIX: char = '!';

    fn new(cmd: Rc<RefCell<dyn ChatCommand>>) -> Self {
        Self { inner: cmd }
    }

    pub fn borrow(&self) -> Ref<dyn ChatCommand> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<dyn ChatCommand> {
        self.inner.borrow_mut()
    }

    // pub fn borrow(&self) -> &dyn ChatCommand {
    //     // SAFETY: Single-threaded unique access
    //     unsafe { &*self.inner.as_ref() }
    // }

    // pub fn borrow_mut(&self) -> &mut dyn ChatCommand {
    //     // SAFETY: Single-threaded unique access
    //     unsafe { &mut *self.inner.as_ptr() }
    // }

    /// Parses the message to check if it's a command
    pub fn parse(message: &str) -> CommandParseResult {
        let trimmed_message = message.trim();

        if trimmed_message.is_empty() || !trimmed_message.starts_with(Self::PREFIX) {
            return CommandParseResult::NotACommand;
        }

        let mut words = trimmed_message.split_whitespace();

        if let Some(first_word) = words.next() {
            // Strip the prefix
            let command_name = first_word.trim_start_matches(Self::PREFIX);

            if command_name.is_empty() {
                return CommandParseResult::InvalidCommand;
            }

            // Check for invalid characters
            if !command_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            {
                return CommandParseResult::InvalidCommand;
            }

            // remaining words as arguments
            let args: Vec<String> = words.map(String::from).collect();

            return CommandParseResult::ValidCommand(command_name.into(), args);
        }

        CommandParseResult::InvalidCommand
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct CommandMap {
    inner: HashMap<String, Command>,
}

impl Default for CommandMap {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
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
            self.inner.insert(name, Command::new(cmd.clone() as _));
        }
    }

    pub fn get(&self, key: &str) -> Option<&Command> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Command> {
        self.inner.get_mut(key)
    }
}

// ----------------------------------------------------------------------------

enum ChatNotification {
    NotACommand,
    InvalidCommand,
    SpamDetected,
    CommandCooldown(String, Duration),
    CommandSentByBot(String),
    CommandDoesNotExist(String),
    HandleError(String, String),
}

/// Sends a message to the chat based on the provided context
fn notify_chat(api: &TwitchApiWrapper, ctx: &MessageData, notification: ChatNotification) {
    let msg = match notification {
        ChatNotification::NotACommand => format!("\"{}\" is not a command", ctx.message.text),
        ChatNotification::InvalidCommand => {
            format!("\"{}\", invalid command format", ctx.message.text)
        }
        ChatNotification::SpamDetected => {
            format!(
                "\"{}\", you are sending commands too quickly",
                ctx.message.text
            )
        }
        ChatNotification::CommandCooldown(cmd_name, duration) => format!(
            "\"{}\" is on cooldown, wait {:.1} seconds",
            cmd_name,
            duration.as_secs_f32()
        ),
        ChatNotification::CommandSentByBot(cmd_name) => format!("\"{}\" sent by bot", cmd_name),
        ChatNotification::CommandDoesNotExist(cmd_name) => {
            format!("\"{}\" does not exist", cmd_name)
        }
        ChatNotification::HandleError(cmd_name, err) => {
            format!("\"{}\" command handle error: {}", cmd_name, err)
        }
    };

    // additional logging for test mode
    let msg = if let TwitchApiWrapper::Test(_) = api {
        format!(
            "@{}, id: {}, msg: {msg}, raw: \"{}\"",
            ctx.chatter.name, ctx.chatter.id, ctx.message.text
        )
    } else {
        msg
    };

    let _ = api.send_chat_message_with_reply(msg, Some(ctx.message_id.clone()));
}

/// Handles incoming chat commands if applicable (validity checks, etc...)
#[instrument(skip(api, ctx, cmds, spam_check))]
pub fn handle_command_if_applicable(
    ctx: &MessageData,
    api: &TwitchApiWrapper,
    cmds: &mut CommandMap,
    bot_id: &str,
    spam_check: &mut SpamCheck,
) {
    // Ignore commands sent by the bot itself
    if ctx.chatter.id == bot_id {
        return;
    }

    // Parse the command from the message
    let (cmd_name, _args) = match Command::parse(&ctx.message.text) {
        CommandParseResult::NotACommand => {
            if let TwitchApiWrapper::Test(_) = api {
                notify_chat(api, ctx, ChatNotification::NotACommand);
            }
            return;
        }
        CommandParseResult::InvalidCommand => {
            notify_chat(api, ctx, ChatNotification::InvalidCommand);
            return;
        }
        CommandParseResult::ValidCommand(cmd_name, args) => (cmd_name, args),
    };

    // Check if the user is sending commands too quickly
    if spam_check.check_spam(&ctx.chatter.id) {
        notify_chat(api, ctx, ChatNotification::SpamDetected);
        return;
    }

    // Check if the command exists and handle it
    let Some(cmd_cell) = cmds.get_mut(&cmd_name) else {
        notify_chat(
            api,
            ctx,
            ChatNotification::CommandDoesNotExist(cmd_name.clone()),
        );
        return;
    };

    // let mut cmd = cmd_cell.borrow_mut();

    // Check if the command is under cooldown
    if let Some(duration) =
        spam_check.check_command_cooldown(&cmd_name, cmd_cell.borrow().cooldown())
    {
        notify_chat(
            api,
            ctx,
            ChatNotification::CommandCooldown(cmd_name, duration),
        );
        return;
    }

    if std::panic::catch_unwind(|| {
        if let Err(err) = cmd_cell.borrow_mut().handle(api, ctx) {
            notify_chat(
                api,
                ctx,
                ChatNotification::HandleError(cmd_name.clone(), err.to_string()),
            );
        }
    })
    .is_err()
    {
        notify_chat(
            api,
            ctx,
            ChatNotification::HandleError(cmd_name.clone(), "Caught panic".to_string()),
        );
    }
}
