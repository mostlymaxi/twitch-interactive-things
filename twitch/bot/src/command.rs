use crate::{api::TwitchApiWrapper, commands::ChatCommand, spam::Spam};
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
            self.inner.insert(name, Command::new(Rc::clone(&cmd) as _));
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

enum ChatErrorKind {
    NotACommand,
    InvalidCommand,
    SpamDetected,
    CommandCooldown(String, Duration),
    // CommandSentByBot(String),
    CommandDoesNotExist(String),
    HandleError(String, String),
}

/// Sends a message to the chat on command error
fn send_chat_err_msg(
    api: &mut TwitchApiWrapper,
    spam: &mut Spam,
    ctx: &MessageData,
    error: ChatErrorKind,
) {
    // Comment this to disable failed command spam handling
    if let Some(cooldown) = spam.check_failed_command_cooldown(&ctx.chatter.id) {
        tracing::warn!(
            "User {} is on error cooldown for another {:.1} seconds",
            ctx.chatter.id,
            cooldown.as_secs_f32()
        );
        return;
    }

    let msg = match error {
        ChatErrorKind::NotACommand => format!("\"{}\" is not a command", ctx.message.text),
        ChatErrorKind::InvalidCommand => {
            format!("\"{}\", invalid command format", ctx.message.text)
        }
        ChatErrorKind::SpamDetected => {
            format!(
                "\"{}\", you are sending commands too quickly",
                ctx.message.text
            )
        }
        ChatErrorKind::CommandCooldown(cmd_name, duration) => format!(
            "\"{}\" is on cooldown, wait {:.1} seconds",
            cmd_name,
            duration.as_secs_f32()
        ),
        // ChatErrorKind::CommandSentByBot(cmd_name) => format!("\"{}\" sent by bot", cmd_name),
        ChatErrorKind::CommandDoesNotExist(cmd_name) => {
            format!("\"{}\" does not exist", cmd_name)
        }
        ChatErrorKind::HandleError(cmd_name, err) => {
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
#[instrument(skip(api, ctx, cmds, spam))]
pub fn handle_command_if_applicable(
    ctx: &MessageData,
    api: &mut TwitchApiWrapper,
    cmds: &mut CommandMap,
    bot_id: &str,
    spam: &mut Spam,
) {
    // Ignore commands sent by the bot itself
    if ctx.chatter.id == bot_id {
        return;
    }

    // Parse the command from the message
    let (cmd_name, _args) = match Command::parse(&ctx.message.text) {
        CommandParseResult::NotACommand => {
            if let TwitchApiWrapper::Test(_) = api {
                send_chat_err_msg(api, spam, ctx, ChatErrorKind::NotACommand);
            }
            return;
        }
        CommandParseResult::InvalidCommand => {
            send_chat_err_msg(api, spam, ctx, ChatErrorKind::InvalidCommand);
            return;
        }
        CommandParseResult::ValidCommand(cmd_name, args) => (cmd_name, args),
    };

    // Check if the user is sending commands too quickly
    if let Some(_) = spam.check_user_command_cooldown(&ctx.chatter.id) {
        send_chat_err_msg(api, spam, ctx, ChatErrorKind::SpamDetected);
        return;
    }

    // Check if the command exists and handle it
    let Some(cmd) = cmds.get_mut(&cmd_name) else {
        send_chat_err_msg(
            api,
            spam,
            ctx,
            ChatErrorKind::CommandDoesNotExist(cmd_name.clone()),
        );
        return;
    };

    let mut cmd = cmd.borrow_mut();

    // Check if the command is under cooldown
    if let Some(duration) = spam.check_global_command_cooldown(&cmd_name, Some(&cmd.rate_limit())) {
        send_chat_err_msg(
            api,
            spam,
            ctx,
            ChatErrorKind::CommandCooldown(cmd_name, duration),
        );
        return;
    }

    if let Err(err) = cmd.handle(api, ctx) {
        send_chat_err_msg(
            api,
            spam,
            ctx,
            ChatErrorKind::HandleError(cmd_name.clone(), err.to_string()),
        );
    }
}
