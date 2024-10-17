//! module containing all commands

#[allow(clippy::module_inception)]
pub mod commands;

// ----------------------------------------------------------------------------
// NOTE: modules must match the command you expect people to use in chat (or at least one of them).
// For example: mostlypasta -> !mostlypasta <gnu> <linux>
//
// but this does not apply to the internal struct.
//
//
// add your command module to this list:
pub mod ban;
pub mod bot_time;
pub mod count;
pub mod discord;
pub mod git;
pub mod help;
pub mod kofi;
pub mod lurk;
pub mod mostlybot;
pub mod mostlypasta;
pub mod ping;
pub mod pong;
pub mod progress;
pub mod rewrite;
pub mod tictactoe;
pub mod uwu;
pub mod vods;
pub mod youtube;

// ----------------------------------------------------------------------------

use crate::api::TwitchApiWrapper;
use crate::command::CommandMap;
use crate::spam::RateLimit;
use anyhow::Result;
use std::time::Duration;
use twitcheventsub::MessageData;

pub const DEFAULT_CMD_COOLDOWN_MS: u64 = 250;
pub const DNE_ERROR_COOLDOWN_SECS: u64 = 30;
pub const GEN_ERROR_COOLDOWN_SECS: u64 = 3;

pub trait ChatCommand: 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn names() -> Vec<String>
    where
        Self: Sized;

    fn rate_limit(&self) -> RateLimit {
        RateLimit::new(1, Duration::from_millis(DEFAULT_CMD_COOLDOWN_MS))
    }

    fn handle(&mut self, api: &mut TwitchApiWrapper, ctx: &MessageData) -> Result<()>;

    fn help(&self) -> String;
}

pub fn init() -> CommandMap {
    let mut map = CommandMap::new();
    // most commands will just be inserted
    map.insert(mostlypasta::MostlyPasta::new());
    map.insert(ping::MostlyPing::new());
    map.insert(pong::MostlyPong::new());
    map.insert(ban::MostlyBan::new());
    map.insert(commands::MostlyCommands::new());
    map.insert(mostlybot::MostlyBot::new());
    map.insert(count::Count::new());
    map.insert(kofi::MostlyKofi::new());
    map.insert(progress::Progress::new());
    map.insert(youtube::MostlyYoutube::new());
    map.insert(git::MostlyGit::new());
    map.insert(discord::MostlyDiscord::new());
    map.insert(vods::MostlyVods::new());
    map.insert(lurk::Lurk::new());
    map.insert(bot_time::BotTime::new());
    map.insert(uwu::MostlyUwU::new());
    map.insert(rewrite::MostlyRewrite::new());
    map.insert(tictactoe::TicTacToe::new());

    // help is special
    let mut help = help::MostlyHelp::new();
    help.init(map.clone());
    map.insert(help);

    map
}
