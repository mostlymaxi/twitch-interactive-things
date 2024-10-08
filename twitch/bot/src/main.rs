mod commands;

use commands::TwitchApiWrapper;
use franz_client::FranzConsumer;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use twitcheventsub::{MessageData, Subscription, TwitchEventSubApi, TwitchKeys};

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

async fn init_franz_consumer(topic: &str) -> FranzConsumer {
    let broker = std::env::var("FRANZ_BROKER").expect("FRANZ_BROKER environment variable set");
    FranzConsumer::new(&broker, &topic.to_owned())
        .await
        .unwrap()
}

async fn cancel_on_signal(token: CancellationToken) {
    match signal::ctrl_c().await {
        Ok(()) => info!("caught signal. shutting down..."),
        Err(err) => error!("unable to listen for shutdown signal: {}", err),
    }

    token.cancel();
}

async fn handle_chat_messages(
    mut consumer: FranzConsumer,
    cancel_token: CancellationToken,
    mut handler: impl FnMut(MessageData, &str),
) {
    while let Some(msg) = select! {
        _ = cancel_token.cancelled() => None,
        m = consumer.recv() => m
    } {
        let Ok(msg) = msg else { continue };
        let Ok(msg) = serde_json::from_str::<MessageData>(&msg) else {
            continue;
        };

        let args = msg.message.text.clone();
        let mut args = args.split_whitespace();

        let Some(cmd) = args
            .next()
            .filter(|cmd| cmd.starts_with('!'))
            .and_then(|cmd| cmd.strip_prefix('!'))
        else {
            continue;
        };

        handler(msg, cmd);
    }
}

// ----------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut api = TwitchApiWrapper::Live(init_twitch_api());
    let consumer = init_franz_consumer("chat").await;

    let cancel_token = CancellationToken::new();
    tokio::spawn(cancel_on_signal(cancel_token.clone()));

    let mut hs = commands::init();
    let bot_id = std::env::var("TITS_BOT_ID").expect("TITS_BOT_ID environment variable set");

    handle_chat_messages(consumer, cancel_token, |msg, cmd| {
        if msg.chatter.id == bot_id {
            return;
        }
        hs.handle_cmd(&mut api, cmd, &msg);
    })
    .await;
}
