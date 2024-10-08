mod commands;

use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use twitcheventsub::{MessageData, Subscription, TwitchEventSubApi, TwitchKeys};

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
            Subscription::PermissionIRCWrite,
        ]);

    twitch.build().expect("twitch api build")
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
    let mut api = init_twitch_api();

    let mut hs = commands::init();
    let twitter = CancellationToken::new();

    let twitter_clone = twitter.clone();
    tokio::spawn(async move {
        cancel_on_signal(twitter_clone).await;
    });

    // pulling all the redeems from twitch
    let broker = std::env::var("FRANZ_BROKER").expect("FRANZ_BROKER environment variable set");
    let bot_id = std::env::var("TITS_BOT_ID").expect("TITS_BOT_ID environment variable set");
    let mut c = franz_client::FranzConsumer::new(&broker, &"chat".to_owned())
        .await
        .unwrap();

    while let Some(msg) = select! {
        _ = twitter.cancelled() => None,
        m = c.recv() => m
    } {
        let Ok(msg) = msg else { continue };
        let Ok(msg) = serde_json::from_str::<MessageData>(&msg) else {
            continue;
        };

        if msg.chatter.id == bot_id {
            continue;
        }

        let args = msg.message.text.clone();
        let mut args = args.split_whitespace();

        let Some(cmd) = args
            .next()
            .filter(|cmd| cmd.starts_with('!'))
            .and_then(|cmd| cmd.strip_prefix('!'))
        else {
            continue;
        };

        hs.handle_cmd(&mut api, cmd, &msg);
    }
}
