#![doc = include_str!("../README.md")]

use mostlybot::*;

use api::TwitchApiWrapper;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use twitcheventsub::{MessageData, Subscription, TwitchEventSubApi, TwitchKeys};

use command::handle_command_if_applicable;
use spam::Spam;

// ----------------------------------------------------------------------------

#[instrument]
fn init_twitch_api() -> TwitchEventSubApi {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let twitch = TwitchEventSubApi::builder(keys)
        // sockets are used to read data from the request so a port
        // must be specified
        .set_redirect_url("https://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        // .enable_irc("mostlymaxi", "mostlymaxi")
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
        .add_subscriptions(vec![
            Subscription::PermissionSendAnnouncements,
            Subscription::PermissionDeleteMessage,
            Subscription::ChatMessage,
        ]);

    twitch.build().expect("twitch api build")
}

#[instrument]
async fn init_franz_consumer(topic: &str) -> franz_client::Consumer {
    let broker = std::env::var("FRANZ_BROKER").expect("FRANZ_BROKER environment variable set");
    franz_client::Consumer::new(&broker, topic, Some(0)).unwrap()
}

async fn cancel_on_signal(token: CancellationToken) {
    match signal::ctrl_c().await {
        Ok(()) => info!("caught signal. shutting down..."),
        Err(err) => error!("unable to listen for shutdown signal: {}", err),
    }

    token.cancel();
}

// ----------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut api = TwitchApiWrapper::Live(init_twitch_api());
    let mut consumer = init_franz_consumer("chat").await;

    let cancel_token = CancellationToken::new();
    tokio::spawn(cancel_on_signal(cancel_token.clone()));

    let mut commands = commands::init();
    let bot_id = std::env::var("TWITCH_BOT_ID").expect("TWITCH_BOT_ID environment variable set");

    let mut spam = Spam::default();

    // handle chat commands
    while let Ok(msg) = consumer.recv() {
        let msg = String::from_utf8(msg).unwrap();
        let Ok(chat_msg) = serde_json::from_str::<MessageData>(&msg) else {
            continue;
        };

        handle_command_if_applicable(&chat_msg, &mut api, &mut commands, &bot_id, &mut spam);
    }
}
