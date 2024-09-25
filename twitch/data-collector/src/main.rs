use std::{collections::HashMap, io::Write, net::TcpStream, thread::sleep, time::Duration};
use twitch_eventsub::*;

// 1. read data from twitch api
// 2. split data into respective topics
// 3. maybe continue the weird file that i do
// for legacy support
fn init_twitch_client() -> TwitchEventSubApi {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let twitch = TwitchEventSubApi::builder(keys)
        .set_redirect_url("https://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
        .add_subscriptions(vec![
            Subscription::ChannelFollow,
            Subscription::ChannelRaid,
            Subscription::ChannelNewSubscription,
            Subscription::ChannelGiftSubscription,
            Subscription::ChannelResubscription,
            Subscription::ChannelCheer,
            Subscription::ChannelPointsCustomRewardRedeem,
            Subscription::ChannelPointsAutoRewardRedeem,
            Subscription::ChatMessage,
            Subscription::AdBreakBegin,
        ]);

    match twitch.build() {
        Ok(api) => api,
        Err(EventSubError::TokenMissingScope) => {
            panic!("Reauthorisation of token is required for the token to have all the requested subscriptions.");
        }
        Err(EventSubError::NoSubscriptionsRequested) => {
            panic!("No subscriptions passed into builder!");
        }
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

const MAX_RETRIES: usize = 5;

fn main() {
    env_logger::init();

    let mut current_retries = 0;

    let mut api = init_twitch_client();
    let mut franz_topics = HashMap::new();

    for topic in ["chat", "follow", "raid", "redeem"] {
        let mut sock = TcpStream::connect("localhost:8085").unwrap();
        sock.write_all(b"0\n").unwrap();
        sock.write_all(topic.as_bytes()).unwrap();
        sock.write_all(b"\n").unwrap();
        franz_topics.insert(topic.to_string(), sock);
    }

    'outer: loop {
        let responses = api.receive_all_messages(Some(Duration::from_millis(10)));
        for response in responses {
            let event = match response {
                ResponseType::Close => break 'outer,
                ResponseType::Ready => {
                    log::debug!("ready");
                    current_retries = 0;
                    continue;
                }
                ResponseType::Error(e) => {
                    log::error!("{:?}", e);
                    if current_retries > MAX_RETRIES {
                        log::error!("max retries reached... exiting");
                        break 'outer;
                    }
                    current_retries += 1;
                    sleep(Duration::from_secs(10));
                    continue;
                }
                ResponseType::Event(e) => e,
                ResponseType::RawResponse(_) => {
                    current_retries = 0;
                    continue;
                }
            };

            current_retries = 0;

            match event {
                Event::ChatMessage(m) => {
                    let mut msg = serde_json::to_string(&m).unwrap();
                    log::trace!("{msg}");
                    msg.push('\n');
                    let msg = msg.as_bytes();

                    franz_topics
                        .get_mut("chat")
                        .unwrap()
                        .write_all(msg)
                        .unwrap();
                }
                Event::Follow(m) => {
                    let mut msg = serde_json::to_string(&m).unwrap();
                    log::trace!("{msg}");
                    msg.push('\n');
                    let msg = msg.as_bytes();

                    franz_topics
                        .get_mut("follow")
                        .unwrap()
                        .write_all(msg)
                        .unwrap();
                }
                Event::Raid(m) => {
                    let mut msg = serde_json::to_string(&m).unwrap();
                    log::trace!("{msg}");
                    msg.push('\n');
                    let msg = msg.as_bytes();

                    franz_topics
                        .get_mut("raid")
                        .unwrap()
                        .write_all(msg)
                        .unwrap();
                }
                Event::PointsCustomRewardRedeem(m) => {
                    let mut msg = serde_json::to_string(&m).unwrap();
                    log::trace!("{msg}");
                    msg.push('\n');
                    let msg = msg.as_bytes();

                    franz_topics
                        .get_mut("redeem")
                        .unwrap()
                        .write_all(msg)
                        .unwrap();
                }

                Event::ChannelPointsAutoRewardRedeem(m) => {
                    let mut msg = serde_json::to_string(&m).unwrap();
                    log::trace!("{msg}");
                    msg.push('\n');
                    let msg = msg.as_bytes();

                    franz_topics
                        .get_mut("redeem")
                        .unwrap()
                        .write_all(msg)
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}
