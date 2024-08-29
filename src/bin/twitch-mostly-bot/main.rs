use mmap_wrapper::MmapMutWrapper;
use std::time::Duration;
use twitch_eventsub::*;
use twitch_interactive_core::*;

struct TwitchMostlyBot {
    api: TwitchEventSubApi,
    latest_info_wrapper: MmapMutWrapper<LatestStreamInfo>,
}

impl TwitchMostlyBot {
    pub fn new() -> Self {
        // TODO: arbitrary file name????
        let latest_info_wrapper = LatestStreamInfo::new_mut("/tmp/strim-mmap-test.bin");

        let api = init_twitch_client();

        Self {
            api,
            latest_info_wrapper,
        }
    }

    fn run(mut self) {
        log::info!("started mostlybot");
        let _latest_info = unsafe { self.latest_info_wrapper.get_inner() };

        loop {
            // Set duration to ZERO for non blocking for loop of messages
            // Recommended for most setups
            // If you are not running this inside a game and just byitself
            // Such as a chat bot, setting this to 1 millis seems to be good
            let responses = self.api.receive_messages(Duration::from_millis(1));
            for response in responses {
                if let ResponseType::Close = response {
                    log::warn!("twitch asked to politely close your socks");
                    break;
                }
                if let ResponseType::Error(e) = response {
                    log::error!("{:#?}", e);
                    break;
                }

                let ResponseType::Event(event) = response else {
                    continue;
                };

                match event {
                    Event::ChatMessage(m) => {}
                    _ => {
                        // Events that you don't care about or are not subscribed to, can be ignored.
                    }
                }
            }
        }
    }
}

fn main() {
    let mostly_bot = TwitchMostlyBot::new();
    mostly_bot.run();
}
