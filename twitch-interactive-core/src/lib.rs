use std::fs::File;
use std::mem;

use mmap_wrapper::{MmapMutWrapper, MmapWrapper};
use twitch_eventsub::{EventSubError, Subscription, TwitchEventSubApi, TwitchKeys};

#[repr(C)]
pub struct LatestStreamInfo {
    pub msgs_per_15s: u64,
    pub msgs_per_30s: u64,
    pub msgs_per_60s: u64,
    pub raid: u64,   // set to true for first 3 minutes of raid or something?
    pub follow: u64, // see above^ but for 15 seconds?
    pub redeem: u64,
    pub waters_per_10m: u64,
}

impl LatestStreamInfo {
    pub fn new_mut<P: AsRef<str>>(path: P) -> MmapMutWrapper<Self> {
        let f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.as_ref())
            .unwrap();

        let _ = f.set_len(mem::size_of::<LatestStreamInfo>() as u64);

        let m = unsafe { memmap2::MmapMut::map_mut(&f).unwrap() };
        MmapMutWrapper::new(m)
    }

    pub fn new<P: AsRef<str>>(path: P) -> MmapWrapper<Self> {
        let f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.as_ref())
            .unwrap();

        let _ = f.set_len(mem::size_of::<LatestStreamInfo>() as u64);

        let m = unsafe { memmap2::Mmap::map(&f).unwrap() };
        MmapWrapper::new(m)
    }
}

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
            Subscription::DeleteMessage,
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
