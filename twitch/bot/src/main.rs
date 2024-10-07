mod commands;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use commands::*;

use serde_json::Value;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

fn init_commands() -> HashMap<String, Rc<RefCell<dyn ChatCommand>>> {
    let mut h: HashMap<String, Rc<RefCell<dyn ChatCommand>>> = HashMap::new();

    // this gonna get macro-ified!!! :D
    let cmd = Rc::new(RefCell::new(mostlygnu::MostlyGnu::new()));
    for name in mostlygnu::MostlyGnu::names() {
        h.insert(name.to_owned(), cmd.clone());
    }

    let cmd = Rc::new(RefCell::new(mostlydebug::MostlyHelp::new_with_args(
        h.clone(),
    )));
    for name in mostlydebug::MostlyHelp::names() {
        h.insert(name.to_owned(), cmd.clone());
    }

    h
}

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

    let mut hs = init_commands();
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
            .filter(|cmd| cmd.starts_with("!"))
            .and_then(|cmd| cmd.strip_prefix("!"))
        else {
            continue;
        };

        hs.get_mut(cmd)
            .map(|c| Some(c.borrow_mut().handle(args.collect(), msg)));
    }
}
