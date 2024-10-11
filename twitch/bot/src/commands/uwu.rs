//! Responds with random UwU kaomoji
//!
//! Broadcaster, moderators, & VIPs automatically have privilege to add new kaomoji
//!
//! I have no clue if the thing this is running on has persistent storage,
//! so there is a somewhat big list of default kaomoji.
//!
//! usage: ```!uwu ?[index]``` ```!uwu [kaomoji]``` ```!uwu remove ?[kaomoji / index]```
//!
//! author: vulae

use std::path::PathBuf;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::ChatCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Kaomoji {
    string: String,
    user_id: String,
}

impl Kaomoji {
    pub fn new<S: Into<String>>(string: S, user_id: S) -> Self {
        Self {
            string: string.into(),
            user_id: user_id.into(),
        }
    }

    pub fn new_dummy<S: Into<String>>(string: S) -> Self {
        Self {
            string: string.into(),
            user_id: "938429017".to_string(), // mostlymaxi user_id
        }
    }
}

impl Default for Kaomoji {
    fn default() -> Self {
        Self::new_dummy("UwU".to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct KaomojiList(Vec<Kaomoji>);

impl KaomojiList {
    fn count(&self) -> usize {
        self.0.len()
    }

    fn random(&self) -> Kaomoji {
        self.0
            .choose(&mut rand::thread_rng())
            .map_or(Kaomoji::default(), |v| v.clone())
    }

    fn get_index(&self, index: isize) -> Option<Kaomoji> {
        let index = if index >= 0 {
            index as usize
        } else {
            (self.0.len() as isize - isize::abs(index)) as usize
        };
        self.0.get(index).cloned()
    }

    fn contains(&self, kaomoji: &str) -> bool {
        self.0.iter().any(|k| k.string == kaomoji)
    }

    fn add(&mut self, kaomoji: Kaomoji) {
        if !self.contains(&kaomoji.string) {
            self.0.push(kaomoji);
        }
    }

    fn remove_kaomoji(&mut self, kaomoji: &str) -> Option<Kaomoji> {
        if let Some(index) = self.0.iter().position(|k| k.string == kaomoji) {
            self.remove_index(index as isize)
        } else {
            None
        }
    }

    /// Remove by index with negative indexing
    fn remove_index(&mut self, index: isize) -> Option<Kaomoji> {
        let index = if index >= 0 {
            index as usize
        } else {
            (self.0.len() as isize - isize::abs(index)) as usize
        };
        if index < self.0.len() {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    fn remove_last(&mut self) -> Option<Kaomoji> {
        self.0.pop()
    }
}

impl Default for KaomojiList {
    fn default() -> Self {
        Self(vec![
            Kaomoji::new_dummy("UwU"),
            Kaomoji::new_dummy("OwO"),
            Kaomoji::new_dummy("0-0"),
            Kaomoji::new_dummy("-_-"),
            Kaomoji::new_dummy(":3"),
            Kaomoji::new_dummy("(˶˃ ᵕ ˂˶) .ᐟ.ᐟ"),
            Kaomoji::new_dummy("⸜(｡˃ ᵕ ˂ )⸝♡"),
            Kaomoji::new_dummy("(^_^)"),
            Kaomoji::new_dummy("(^>-<^)"),
            Kaomoji::new_dummy("∩(·ω·)∩"),
            Kaomoji::new_dummy("^ω^"),
            Kaomoji::new_dummy("^_^"),
            Kaomoji::new_dummy("=^_^="),
            Kaomoji::new_dummy("(＾ｖ＾)"),
            Kaomoji::new_dummy("(*´▽｀*)"),
            Kaomoji::new_dummy("(´･ω･`)"),
            Kaomoji::new_dummy("(´；ω；`)"),
            Kaomoji::new_dummy("(˶◜ᵕ◝˶)"),
            Kaomoji::new_dummy("∘ ∘ ∘ ( °ヮ° ) ?"),
            Kaomoji::new_dummy("( ˶°ㅁ°) !!"),
            Kaomoji::new_dummy("(๑-﹏-๑)"),
            Kaomoji::new_dummy(" ˶ᵔ ᵕ ᵔ˶ "),
        ])
    }
}

/// This just parses the command arguments to make my life easier
#[derive(Debug, PartialEq)]
enum MostlyUwUArgs {
    DisplayRandom,
    DisplayIndex { index: isize },
    AddKaomoji { kaomoji: String },
    RemoveKaomoji { kaomoji: String },
    RemoveIndex { index: isize },
    RemoveLast,
}

impl TryFrom<&twitcheventsub::MessageData> for MostlyUwUArgs {
    type Error = anyhow::Error;

    fn try_from(ctx: &twitcheventsub::MessageData) -> Result<Self, Self::Error> {
        let content = ctx
            .message
            .text
            .split(' ')
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_owned();

        if content.is_empty() {
            Ok(MostlyUwUArgs::DisplayRandom)
        } else {
            let first = content.split(' ').nth(0).unwrap();
            if first.to_lowercase() == "remove" {
                let rest = content.split(' ').skip(1).collect::<Vec<_>>().join(" ");
                if rest.is_empty() {
                    Ok(MostlyUwUArgs::RemoveLast)
                } else if let Ok(index) = rest.parse::<isize>() {
                    Ok(MostlyUwUArgs::RemoveIndex { index })
                } else {
                    Ok(MostlyUwUArgs::RemoveKaomoji {
                        kaomoji: rest.trim().to_owned(),
                    })
                }
            } else if let Ok(index) = first.parse::<isize>() {
                Ok(MostlyUwUArgs::DisplayIndex { index })
            } else {
                Ok(MostlyUwUArgs::AddKaomoji {
                    kaomoji: content.trim().to_owned(),
                })
            }
        }
    }
}

pub struct MostlyUwU {
    kaomoji_file_path: PathBuf,
    /// If to allow any user to add a kaomoji
    kaomoji_add_permissions_user: bool,
}

impl MostlyUwU {
    fn load_kaomoji_list(&self) -> anyhow::Result<KaomojiList> {
        if !std::fs::exists(&self.kaomoji_file_path)? {
            return Ok(KaomojiList::default());
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(
            &self.kaomoji_file_path,
        )?)?)
    }

    fn save_kaomoji_list(&self, list: &KaomojiList) -> anyhow::Result<()> {
        std::fs::write(&self.kaomoji_file_path, serde_json::to_string(list)?)?;
        Ok(())
    }

    #[allow(unused)]
    fn delete_kaomoji_list(&self) -> anyhow::Result<()> {
        if std::fs::exists(&self.kaomoji_file_path)? {
            std::fs::remove_file(&self.kaomoji_file_path)?;
        }
        Ok(())
    }
}

impl ChatCommand for MostlyUwU {
    fn new() -> Self {
        Self {
            kaomoji_file_path: "kaomoji.json".into(),
            kaomoji_add_permissions_user: false,
        }
    }

    fn names() -> Vec<String> {
        vec![
            "uwu".to_string(),
            "UwU".to_string(),
            "owo".to_string(),
            "OwO".to_string(),
            "kaomoji".to_string(),
        ]
    }

    fn help(&self) -> String {
        "Responds UwU kaomoji\nusage: \"!uwu ?[index]\" - Random kaomoji\n\"!uwu [kaomoji]\" - Add kaomoji\n\"!uwu remove ?[kaomoji / index]\" - Remove kaomoji".to_string()
    }

    #[instrument(skip(self, api))]
    fn handle(
        &mut self,
        api: &super::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        // Collect Args -> Load kaomoji -> Validate permissions -> Execute command

        // Parse command arguments
        let Ok(args) = MostlyUwUArgs::try_from(ctx) else {
            let _ = api.send_chat_message_with_reply("Invalid arguments.", Some(&ctx.message_id));
            return Ok(());
        };

        let mut invalid_permissions = || -> anyhow::Result<()> {
            let _ = api
                .send_chat_message_with_reply("You can't do that. (•̀⤙•́ )", Some(&ctx.message_id));
            Ok(())
        };

        let mut kaomoji_list = self.load_kaomoji_list()?;

        match args {
            MostlyUwUArgs::DisplayRandom => {
                let kaomoji = kaomoji_list.random();
                let _ = api.send_chat_message_with_reply(&kaomoji.string, Some(&ctx.message_id));
            }
            MostlyUwUArgs::DisplayIndex { index } => {
                if let Some(kaomoji) = kaomoji_list.get_index(index) {
                    let _ =
                        api.send_chat_message_with_reply(&kaomoji.string, Some(&ctx.message_id));
                } else {
                    let _ = api.send_chat_message_with_reply(
                        &format!(
                            "Index out of bounds ({}), oh no (╥﹏╥)",
                            kaomoji_list.count()
                        ),
                        Some(&ctx.message_id),
                    );
                }
            }
            MostlyUwUArgs::AddKaomoji { kaomoji } => {
                if !(self.kaomoji_add_permissions_user
                    || ctx.moderator
                    || (ctx.chatter.id == ctx.broadcaster.id && !ctx.chatter.id.is_empty())
                    || ctx.badges.iter().any(|badge| badge.set_id == "vip"))
                {
                    return invalid_permissions();
                }

                let kaomoji = Kaomoji::new(&kaomoji, &ctx.chatter.id);
                if !kaomoji_list.contains(&kaomoji.string) {
                    let _ = api.send_chat_message_with_reply(
                        &format!("{} was added ( ˶ˆᗜˆ˵ )", &kaomoji.string),
                        Some(&ctx.message_id),
                    );
                    kaomoji_list.add(kaomoji);
                    self.save_kaomoji_list(&kaomoji_list)?;
                } else {
                    let _ = api.send_chat_message_with_reply(
                        &format!("{} Already exists (◔_◔)", &kaomoji.string),
                        Some(&ctx.message_id),
                    );
                }
            }
            MostlyUwUArgs::RemoveKaomoji { kaomoji } => {
                if !(ctx.moderator
                    || (ctx.chatter.id == ctx.broadcaster.id && !ctx.chatter.id.is_empty()))
                {
                    return invalid_permissions();
                }

                if let Some(kaomoji) = kaomoji_list.remove_kaomoji(&kaomoji) {
                    let _ = api.send_chat_message_with_reply(
                        &format!("{} was removed ꃋᴖꃋ", &kaomoji.string),
                        Some(&ctx.message_id),
                    );
                    self.save_kaomoji_list(&kaomoji_list)?;
                } else {
                    let _ = api.send_chat_message_with_reply(
                        "Couldn't find kaomoji to remove (ㅠ‸ㅠ)",
                        Some(&ctx.message_id),
                    );
                }
            }
            MostlyUwUArgs::RemoveIndex { index } => {
                if !(ctx.moderator
                    || (ctx.chatter.id == ctx.broadcaster.id && !ctx.chatter.id.is_empty()))
                {
                    return invalid_permissions();
                }

                if let Some(kaomoji) = kaomoji_list.remove_index(index) {
                    let _ = api.send_chat_message_with_reply(
                        &format!("{} was removed (ㅠ﹏ㅠ)", &kaomoji.string),
                        Some(&ctx.message_id),
                    );
                    self.save_kaomoji_list(&kaomoji_list)?;
                } else {
                    let _ = api.send_chat_message_with_reply(
                        &format!(
                            "Index out of bounds ({}), oh no (╥﹏╥)",
                            kaomoji_list.count()
                        ),
                        Some(&ctx.message_id),
                    );
                }
            }
            MostlyUwUArgs::RemoveLast => {
                if !(ctx.moderator
                    || (ctx.chatter.id == ctx.broadcaster.id && !ctx.chatter.id.is_empty()))
                {
                    return invalid_permissions();
                }

                if let Some(kaomoji) = kaomoji_list.remove_last() {
                    let _ = api.send_chat_message_with_reply(
                        &format!("{} was removed .‸.", &kaomoji.string),
                        Some(&ctx.message_id),
                    );
                    self.save_kaomoji_list(&kaomoji_list)?;
                } else {
                    let _ = api.send_chat_message_with_reply(
                        "There's no kaomoji, what did you do? (ó﹏ò｡)",
                        Some(&ctx.message_id),
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use twitcheventsub::MessageData;

    use super::*;
    use crate::api::{MockTwitchEventSubApi, TwitchApiWrapper};

    fn create_test_msg(content: &str) -> MessageData {
        serde_json::from_str(&format!(
            "{{\"broadcaster_user_id\":\"938429017\",\"broadcaster_user_name\":\"mostlymaxi\",\"broadcaster_user_login\":\"mostlymaxi\",\"chatter_user_id\":\"938429017\",\"chatter_user_name\":\"mostlymaxi\",\"chatter_user_login\":\"mostlymaxi\",\"message_id\":\"3104f083-2bdb-4d6a-bb5d-30b407876ea4\",\"message\":{{\"text\":\"{}\",\"fragments\":[{{\"type\":\"text\",\"text\":\"{}\",\"cheermote\":null,\"emote\":null,\"mention\":null}}]}},\"color\":\"#FF0000\",\"badges\":[{{\"set_id\":\"broadcaster\",\"id\":\"1\",\"info\":\"\"}},{{\"set_id\":\"subscriber\",\"id\":\"0\",\"info\":\"3\"}}],\"message_type\":\"text\",\"cheer\":null,\"reply\":null,\"channel_points_custom_reward_id\":null,\"channel_points_animation_id\":null}}",
            content, content
        )).unwrap()
    }

    fn create_test_user_msg(content: &str) -> MessageData {
        serde_json::from_str(&format!(
            "{{\"broadcaster_user_id\":\"938429017\",\"broadcaster_user_name\":\"mostlymaxi\",\"broadcaster_user_login\":\"mostlymaxi\",\"chatter_user_id\":\"\",\"chatter_user_name\":\"\",\"chatter_user_login\":\"\",\"message_id\":\"\",\"message\":{{\"text\":\"{}\",\"fragments\":[{{\"type\":\"text\",\"text\":\"{}\",\"cheermote\":null,\"emote\":null,\"mention\":null}}]}},\"color\":\"#000000\",\"badges\":[],\"message_type\":\"text\",\"cheer\":null,\"reply\":null,\"channel_points_custom_reward_id\":null,\"channel_points_animation_id\":null}}",
            content, content
        )).unwrap()
    }

    #[test]
    fn handle() -> anyhow::Result<()> {
        let mut api = TwitchApiWrapper::Test(MockTwitchEventSubApi::init_twitch_api());
        let mut cmd = MostlyUwU::new();
        cmd.kaomoji_file_path = "kaomoji_test.json".into();

        // Use `cargo test commands::uwu::test::handle -- --nocapture` to validate response output

        cmd.handle(&mut api, &create_test_msg("!uwu"))?; // {}
        cmd.handle(&mut api, &create_test_msg("!uwu remove UwU"))?; // UwU was removed
        cmd.handle(&mut api, &create_test_msg("!uwu remove UwU"))?; // Could not find
        cmd.handle(&mut api, &create_test_msg("!uwu UwU"))?; // UwU was added
        cmd.handle(&mut api, &create_test_msg("!uwu UwU"))?; // UwU already exists

        cmd.handle(&mut api, &create_test_msg("!uwu 0"))?; // OwO
        cmd.handle(&mut api, &create_test_msg("!uwu -1"))?; // UwU

        cmd.handle(&mut api, &create_test_user_msg("!uwu"))?; // {}
        cmd.handle(&mut api, &create_test_user_msg("!uwu remove UwU"))?; // You can't do that.
        cmd.handle(&mut api, &create_test_user_msg("!uwu UwU"))?; // You can't do that.

        cmd.delete_kaomoji_list()?;

        Ok(())
    }
}
