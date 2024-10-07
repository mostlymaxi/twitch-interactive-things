mod commands;

use serde_json::Value;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use twitcheventsub::{Subscription, TwitchEventSubApi, TwitchKeys};

fn init_twitch_api() -> TwitchEventSubApi {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let twitch = TwitchEventSubApi::builder(keys)
        // sockets are used to read data from the request so a port
        // must be specified
        .set_redirect_url("https://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .enable_irc("mostlymaxi", "mostlymaxi")
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
        .add_subscriptions(vec![Subscription::PermissionIRCWrite]);

    twitch.build().expect("twitch api build")
}

#[tokio::main]
async fn main() {
    let mut api = init_twitch_api();

    let mut hs = commands::init();
    let twitter = CancellationToken::new();

    let twitter_clone = twitter.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }

        twitter_clone.cancel();
    });

    // pulling all the redeems from twitch
    let mut c = franz_client::FranzConsumer::new("tits.franz.mostlymaxi.com:8085", "chat")
        .await
        .unwrap();

    while let Some(msg) = select! {
        _ = twitter.cancelled() => None,
        m = c.recv() => m
    } {
        let Ok(msg) = msg else { continue };
        let Ok(msg) = serde_json::from_str::<Value>(&msg) else {
            continue;
        };

        let args = msg["message"]["text"].to_string();
        let mut args = args.split_whitespace();

        let Some(cmd) = args
            .next()
            .filter(|cmd| cmd.starts_with('!'))
            .and_then(|cmd| cmd.strip_prefix('!'))
        else {
            continue;
        };

        hs.handle_cmd(cmd, args.collect(), msg);
    }
}
