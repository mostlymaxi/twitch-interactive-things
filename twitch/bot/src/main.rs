#![doc = include_str!("../README.md")]
use std::time::Duration;

use mostlybot::*;

use api::TwitchApiWrapper;
use franz_client::FranzConsumer;
use tokio::{select, signal, time};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};
use twitcheventsub::{MessageData, Subscription, TwitchEventSubApi, TwitchKeys};

use command::handle_command_if_applicable;
use spam::SpamManager;

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
async fn init_franz_consumer(topic: &str) -> FranzConsumer {
    let broker = std::env::var("FRANZ_BROKER").expect("FRANZ_BROKER environment variable set");
    let mut consumer = FranzConsumer::new(&broker, &topic.to_owned())
        .await
        .unwrap();

    loop {
        if time::timeout(Duration::from_millis(500), consumer.recv())
            .await
            .is_err()
        {
            debug!("franz consumer caught up on messages");
            break;
        }
    }

    consumer
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

    let mut spam_manager = SpamManager::default();

    // handle chat commands
    while let Some(msg) = select! {
        _ = cancel_token.cancelled() => None,
        msg = consumer.recv() => msg
    } {
        let Ok(msg) = msg else { continue };

        let Ok(chat_msg) = serde_json::from_str::<MessageData>(&msg) else {
            continue;
        };

        handle_command_if_applicable(
            &chat_msg,
            &mut api,
            &mut commands,
            &bot_id,
            &mut spam_manager,
        );
    }
}
