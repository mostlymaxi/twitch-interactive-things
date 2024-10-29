use std::{collections::HashMap, time::Duration};
use tokio::{select, signal, time};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use twitcheventsub::*;

struct DataCollector {
    api: TwitchEventSubApi,
    producers: HashMap<String, franz_client::Producer>,
    twitter: CancellationToken,
}

impl DataCollector {
    pub async fn new(twitter: CancellationToken) -> Self {
        let api = Self::init_twitch_client();
        let producers = Self::init_franz_producers(vec!["redeem", "chat", "follow", "raid"]).await;

        DataCollector {
            api,
            producers,
            twitter,
        }
    }

    #[instrument(skip(self, msg))]
    async fn send(&mut self, topic: &str, msg: &str) {
        match self
            .producers
            .get_mut(topic)
            .expect("chat topic exists")
            .send(msg)
        {
            Err(e) => {
                tracing::error!(error = %e, msg = %msg);
                self.twitter.cancel();
            }
            Ok(()) => tracing::debug!(msg = %msg),
        }
    }

    #[instrument(skip(self))]
    pub async fn run(mut self) {
        let mut interval = time::interval(Duration::from_millis(50));

        loop {
            select! {
                _ = self.twitter.cancelled() => break,
                _ = interval.tick() => {},
            }

            for response in self.api.receive_all_messages(Some(Duration::ZERO)) {
                let event = match response {
                    ResponseType::Close => {
                        self.twitter.cancel();
                        break;
                    }
                    ResponseType::Ready => {
                        tracing::debug!("ready");
                        continue;
                    }
                    ResponseType::Error(e) => {
                        tracing::error!("{:#?}", e);
                        self.twitter.cancel();
                        break;
                    }
                    ResponseType::Event(e) => e,
                    ResponseType::RawResponse(r) => {
                        tracing::trace!(response = %r);
                        continue;
                    }
                };

                match event {
                    Event::ChatMessage(m) => {
                        let m = serde_json::to_string(&m).unwrap();
                        self.send("chat", &m).await;
                    }
                    Event::Follow(m) => {
                        let m = serde_json::to_string(&m).unwrap();
                        self.send("follow", &m).await;
                    }
                    Event::Raid(m) => {
                        let m = serde_json::to_string(&m).unwrap();
                        self.send("raid", &m).await;
                    }
                    Event::PointsCustomRewardRedeem(m) => {
                        let m = serde_json::to_string(&m).unwrap();
                        self.send("redeem", &m).await;
                    }
                    Event::ChannelPointsAutoRewardRedeem(m) => {
                        let m = serde_json::to_string(&m).unwrap();
                        self.send("redeem", &m).await;
                    }
                    _ => {}
                }
            }
        }
    }

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

    async fn init_franz_producers(topics: Vec<&str>) -> HashMap<String, franz_client::Producer> {
        let mut franz_producers = HashMap::new();
        let broker = std::env::var("FRANZ_BROKER").expect("FRANZ_BROKER environment variable set");

        for topic in topics {
            let p = franz_client::Producer::new(&broker, &topic.to_owned())
                .expect("franz client connection");

            franz_producers.insert(topic.to_string(), p);
        }

        franz_producers
    }
}

async fn cancel_on_signal(twitter: CancellationToken) {
    match signal::ctrl_c().await {
        Ok(()) => tracing::info!("caught signal. shutting down..."),
        Err(err) => tracing::error!("unable to listen for shutdown signal: {}", err),
    }

    twitter.cancel();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let twitter = CancellationToken::new();
    let twitter_c = twitter.clone();
    let dc = DataCollector::new(twitter_c).await;

    select! {
        _ = dc.run() => {},
        _ = cancel_on_signal(twitter) => {},
    }
}
