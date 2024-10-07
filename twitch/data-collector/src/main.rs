use franz_client::FranzProducer;
use std::{collections::HashMap, time::Duration};
use twitcheventsub::*;

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

    twitch.build().expect("twitch api client")
}

async fn init_franz_producers(topics: Vec<&str>) -> HashMap<String, FranzProducer> {
    let mut franz_producers = HashMap::new();

    for topic in topics {
        let p = franz_client::FranzProducer::new("tits.franz.mostlymaxi.com:8085", topic)
            .await
            .expect("franz client connection");

        franz_producers.insert(topic.to_string(), p);
    }

    franz_producers
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut api = init_twitch_client();
    let mut producers = init_franz_producers(vec!["redeem", "chat", "follow", "raid"]).await;

    'outer: loop {
        let responses = api.receive_all_messages(Some(Duration::from_millis(10)));
        for response in responses {
            let event = match response {
                ResponseType::Close => break 'outer,
                ResponseType::Ready => {
                    log::debug!("ready");
                    continue;
                }
                ResponseType::Error(e) => {
                    panic!("{:?}", e);
                }
                ResponseType::Event(e) => e,
                ResponseType::RawResponse(_) => {
                    continue;
                }
            };

            match event {
                Event::ChatMessage(m) => {
                    let msg = serde_json::to_string(&m).unwrap();
                    log::debug!("{msg}");

                    producers
                        .get_mut("chat")
                        .expect("chat topic exists")
                        .send(msg)
                        .await
                        .unwrap();
                }
                Event::Follow(m) => {
                    let msg = serde_json::to_string(&m).unwrap();
                    log::debug!("{msg}");

                    franz_topics
                        .get_mut("follow")
                        .expect("follow topic exists")
                        .send(msg)
                        .await
                        .unwrap();
                }
                Event::Raid(m) => {
                    let msg = serde_json::to_string(&m).unwrap();
                    log::debug!("{msg}");

                    franz_topics
                        .get_mut("raid")
                        .expect("raid topic exists")
                        .send(msg)
                        .await
                        .unwrap();
                }
                Event::PointsCustomRewardRedeem(m) => {
                    let msg = serde_json::to_string(&m).unwrap();
                    log::debug!("{msg}");

                    franz_topics
                        .get_mut("redeem")
                        .expect("redeem topic exists")
                        .send(msg)
                        .await
                        .unwrap();
                }

                Event::ChannelPointsAutoRewardRedeem(m) => {
                    let msg = serde_json::to_string(&m).unwrap();
                    log::debug!("{msg}");

                    franz_topics
                        .get_mut("redeem")
                        .expect("redeem topic exists")
                        .send(msg)
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}
