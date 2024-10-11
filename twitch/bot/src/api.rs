use std::cell::RefCell;
use std::time::Duration;

use twitcheventsub::{EventSubError, TwitchEventSubApi};

pub struct MockTwitchEventSubApi {}

impl MockTwitchEventSubApi {
    pub fn init_twitch_api() -> MockTwitchEventSubApi {
        MockTwitchEventSubApi {}
    }
}

pub enum TwitchApiWrapper {
    Live(RefCell<TwitchEventSubApi>),
    Test(MockTwitchEventSubApi),
}

impl std::panic::RefUnwindSafe for TwitchApiWrapper {}

impl TwitchApiWrapper {
    pub fn send_chat_message<S: Into<String>>(&self, message: S) -> Result<String, EventSubError> {
        let res = match self {
            Self::Live(api) => api.borrow_mut().send_chat_message(message),
            Self::Test(_mock) => todo!(),
        };

        // apparently this is more than enough
        std::thread::sleep(Duration::from_millis(100));
        res
    }

    pub fn send_chat_message_with_reply<S: Into<String>>(
        &self,
        message: S,
        reply_message_parent_id: Option<S>,
    ) -> Result<String, EventSubError> {
        let res = match self {
            Self::Live(api) => api
                .borrow_mut()
                .send_chat_message_with_reply(message, reply_message_parent_id.map(S::into)),
            Self::Test(_mock) => {
                println!("{}", message.into());
                Ok(String::new())
            }
        };

        // apparently this is more than enough
        std::thread::sleep(Duration::from_millis(100));

        res
    }
}
