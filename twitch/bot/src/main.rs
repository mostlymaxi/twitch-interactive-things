mod commands;

use serde_json::Value;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
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

    let mut hs = commands::init();

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
