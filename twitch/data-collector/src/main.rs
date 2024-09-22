use twitch_eventsub::{EventSubError, Subscription, TwitchEventSubApi, TwitchKeys};

// 1. read data from twitch api
// 2. split data into respective topics
// 3. maybe continue the weird file that i do
// for legacy support
pub fn init_twitch_client() -> TwitchEventSubApi {
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

fn main() {
    println!("Hello, world!");
}
