use mmap_wrapper::MmapMutWrapper;
use std::time::{Duration, SystemTime};
use twitch_eventsub::*;
use twitch_interactive_core::*;

struct TwitchDataCollector {
    api: TwitchEventSubApi,
    now: SystemTime,
    tail_msgs: Vec<Duration>,
    latest_info_wrapper: MmapMutWrapper<LatestStreamInfo>,
}

impl TwitchDataCollector {
    pub fn new() -> Self {
        // TODO: arbitrary file name????
        let latest_info_wrapper = LatestStreamInfo::new_mut("/tmp/strim-mmap-test.bin");

        let api = init_twitch_client();

        Self {
            api,
            now: SystemTime::now(),
            tail_msgs: Vec::new(),
            latest_info_wrapper,
        }
    }

    fn run(mut self) {
        log::info!("started data collector...");
        let latest_info = unsafe { self.latest_info_wrapper.get_inner() };

        loop {
            self.clean();

            latest_info.msgs_per_15s = self.gabagoo(15);
            latest_info.msgs_per_30s = self.gabagoo(30);
            latest_info.msgs_per_60s = self.gabagoo(60);

            // could be its own function
            if latest_info.follow < self.now.elapsed().unwrap().as_secs().saturating_sub(60) {
                latest_info.follow = 0;
            }

            if latest_info.raid < self.now.elapsed().unwrap().as_secs().saturating_sub(60 * 5) {
                latest_info.raid = 0;
            }

            // Set duration to ZERO for non blocking for loop of messages
            // Recommended for most setups
            // If you are not running this inside a game and just byitself
            // Such as a chat bot, setting this to 1 millis seems to be good
            let responses = self.api.receive_messages(Duration::from_millis(300));
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
                    Event::ChatMessage(_) => {
                        self.tail_msgs.push(self.now.elapsed().unwrap());
                    }
                    Event::Follow(_) => {
                        latest_info.follow = self.now.elapsed().unwrap().as_secs();
                    }
                    Event::Raid(_) => {
                        latest_info.raid = self.now.elapsed().unwrap().as_secs();
                    }
                    Event::PointsCustomRewardRedeem(redeem) => {
                        match redeem.reward.title.as_ref() {
                            "mostlytrain" => latest_info.redeem = 1,
                            "mostlypride" => latest_info.redeem = 2,
                            "mostlymusic" => latest_info.redeem = 3,
                            "mostlypackets" => latest_info.redeem = 4,
                            _ => {}
                        }

                        log::debug!("latest_redeem: {}", latest_info.redeem);
                    }
                    Event::ChannelPointsAutoRewardRedeem(redeem) => {
                        log::debug!("{:#?}", redeem);
                    }
                    _ => {
                        // Events that you don't care about or are not subscribed to, can be ignored.
                    }
                }
            }
        }
    }

    fn gabagoo(&self, n: u64) -> u64 {
        self.tail_msgs
            .iter()
            .filter(|t| t.as_secs() > self.now.elapsed().unwrap().as_secs().saturating_sub(n))
            .count() as u64
    }

    fn clean(&mut self) {
        self.tail_msgs.retain(|t| {
            t.as_secs()
                > self
                    .now
                    .elapsed()
                    .unwrap()
                    .as_secs()
                    .saturating_sub(60 * 30)
        });
    }
}

fn main() {
    let data_collector = TwitchDataCollector::new();
    data_collector.run();
}
