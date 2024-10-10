//! test command that replies pong
//!
//! usage: ```!ping```
//!
//! author: mostlymaxi

use anyhow::anyhow;
use tracing::{debug, error, instrument};

use super::ChatCommand;

/// See ```this``` struct for more details on implementing your own command.
///
/// Commands are backed by structs, this allows for them to have internal,
/// mutable state (think counting the number of times the command has been called).
/// The name of the struct is not terribly important but should be intuitively tied
/// to the command name.
///
/// The ping command doesn't take any state so it can be defined as so:
///
/// ```ignore
/// struct MostlyPing {}
/// ```
pub struct MostlyPing {}

/// All commands must implement the ChatCommand trait
/// (traits are interfaces in rust).
///
/// ```ignore
/// impl ChatCommand for MostlyPing { ... }
/// ```
impl ChatCommand for MostlyPing {
    /// Constructor for your command struct. Initialize
    /// any internal state that your command will need.
    fn new() -> Self {
        Self {}
    }

    /// A function to get the list "names" of your command.
    /// This is what will eventually be matched on in twitch chat.
    ///
    /// ### example
    /// in order for the command to be run whenever a user types ```!ping``` we
    /// must do the following:
    ///
    /// ```rs
    /// fn names() -> Vec<String> {
    ///     vec!["ping".to_string()]
    /// }
    /// ```
    ///
    /// Note: the lack of exclamation marks in the name. They are assumed to already be there.
    fn names() -> Vec<String> {
        vec!["ping".to_string()]
    }

    /// function that returns a helpful message when a chatter either does:
    /// ```!help <command name>``` or the command returns an error
    fn help(&self) -> String {
        "usage: !ping".to_string()
    }

    /// Where the magic happens. Use the api and context (bunch of data around the chat message
    /// that matches your command) to do whatever it is you want your command to do.
    ///
    /// Some basic rules involve:
    /// - no panics, return an error instead
    /// - limit processing time as much as possible (< 2 seconds)
    /// - dont spawn background threads / fork
    /// - make your code readable and well documented
    /// - be reasonable
    ///
    /// Obviously every rule has it's exceptions and will be checked on a case by case basis.
    ///
    /// Your goal is to CONVINCE ME that this command is a good idea so it's an exercise in
    /// clear communication - NOT JUST CODING SKILL
    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &mut super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        match api.send_chat_message_with_reply("pong", Some(&ctx.message_id)) {
            Ok(s) => {
                debug!(reply = %s);
                Ok(())
            }
            Err(e) => {
                error!(error = ?e);
                Err(anyhow!("{:?}", e))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::{MockTwitchEventSubApi, TwitchApiWrapper};

    #[test]
    fn handle() {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyPing::new();

        let test_msg = r###"{"broadcaster_user_id":"938429017","broadcaster_user_name":"mostlymaxi","broadcaster_user_login":"mostlymaxi","chatter_user_id":"938429017","chatter_user_name":"mostlymaxi","chatter_user_login":"mostlymaxi","message_id":"3104f083-2bdb-4d6a-bb5d-30b407876ea4","message":{"text":"!ping","fragments":[{"type":"text","text":"!ping","cheermote":null,"emote":null,"mention":null}]},"color":"#FF0000","badges":[{"set_id":"broadcaster","id":"1","info":""},{"set_id":"subscriber","id":"0","info":"3"}],"message_type":"text","cheer":null,"reply":null,"channel_points_custom_reward_id":null,"channel_points_animation_id":null}"###;

        let ctx = serde_json::from_str(test_msg).unwrap();
        cmd.handle(&mut api, &ctx).unwrap();
    }
}
