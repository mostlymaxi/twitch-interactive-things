use std::str::FromStr;
use std::time::Duration;

use strum::EnumString;
use tokio::process::{Child, Command};
use tokio::{select, signal, sync};
use tokio_util::sync::CancellationToken;

#[derive(Debug, EnumString)]
#[strum(ascii_case_insensitive)]
enum CommandRedeem {
    MostlyTrain,
    MostlyPackets,
    MostlyPride,
    MostlyCPU,
    MostlyMusic,
    MostlyWater,
    MostlyStretch,
    MostlyKeyboard,
    MostlyStfu,
    MostlyMario,
    MostlyNeo,
    MostlyRedpill,
}

impl From<CommandRedeem> for Command {
    fn from(value: CommandRedeem) -> Self {
        match value {
            CommandRedeem::MostlyTrain => Command::new("sl-loop"),
            CommandRedeem::MostlyCPU => {
                let mut c = Command::new("btm");
                c.arg("-e");
                c.args(["--default_widget_type", "cpu"]);
                c
            }
            CommandRedeem::MostlyPackets => {
                let mut c = Command::new("btm");
                c.arg("-e");
                c.args(["--default_widget_type", "net"]);
                c
            }
            CommandRedeem::MostlyPride => {
                let mut c = Command::new("hyfetch");
                c.arg("--june");
                c
            }
            CommandRedeem::MostlyMusic => Command::new("cava"),
            CommandRedeem::MostlyWater => Command::new("asciifish"),
            CommandRedeem::MostlyStretch => {
                let mut c = Command::new("cbonsai");
                c.arg("-li");
                c
            }
            CommandRedeem::MostlyStfu => {
                let mut c = Command::new("cbonsai");
                c.arg("-li");
                c
            }
            CommandRedeem::MostlyKeyboard => Command::new("genact"),
            CommandRedeem::MostlyMario => Command::new("pipes.sh"),
            CommandRedeem::MostlyNeo => Command::new("cmatrix"),
            CommandRedeem::MostlyRedpill => Command::new("cmatrix"),
        }
    }
}

struct PointsTerm {
    current_running_process: Child,
    current_redeem: Command,
}

impl PointsTerm {
    pub fn new() -> Self {
        let mut current_redeem = Command::new("cava");
        current_redeem.kill_on_drop(true);
        let current_running_process = current_redeem.spawn().expect("spawned process");

        Self {
            current_redeem,
            current_running_process,
        }
    }

    pub async fn update(&mut self, redeem: CommandRedeem) {
        self.kill_current().await;
        self.current_redeem = redeem.into();
        self.current_running_process = self.current_redeem.spawn().expect("spawned process");
    }

    async fn kill_current(&mut self) {
        let child_id = match self.current_running_process.id() {
            Some(id) => id,
            None => return,
        };

        let _ = Command::new("kill")
            .args(["--timeout", "1000", "TERM"])
            .args(["--timeout", "1000", "KILL"])
            .args(["--signal", "INT"])
            .arg(child_id.to_string())
            .spawn()
            .unwrap()
            .wait()
            .await;

        let _ = self.current_running_process.wait().await;
    }
}

impl Drop for PointsTerm {
    fn drop(&mut self) {
        let child_id = match self.current_running_process.id() {
            Some(id) => id,
            None => return,
        };

        let _ = std::process::Command::new("kill")
            .args(["--timeout", "1000", "TERM"])
            .args(["--timeout", "1000", "KILL"])
            .args(["--signal", "INT"])
            .arg(child_id.to_string())
            .spawn()
            .unwrap()
            .wait();

        let _ = self.current_running_process.try_wait();
    }
}

#[tokio::main]
async fn main() {
    let twitter = CancellationToken::new();

    let (msg_tx, mut msg_rx) = sync::watch::channel(None);

    let twitter_clone = twitter.clone();
    tokio::spawn(async move {
        let mut pt = PointsTerm::new();

        while !twitter_clone.is_cancelled() {
            // get the latest redeem every 300 ms and update the points term
            tokio::time::sleep(Duration::from_millis(300)).await;
            if msg_rx.changed().await.is_err() {
                break;
            }

            let latest_msg: Option<String> = msg_rx.borrow_and_update().clone();

            if let Some(latest_msg) = latest_msg {
                let Ok(latest_msg) = serde_json::from_str::<serde_json::Value>(&latest_msg) else {
                    continue;
                };

                let Ok(cmd) = CommandRedeem::from_str(
                    latest_msg["reward"]["title"].to_string().trim_matches('"'),
                ) else {
                    continue;
                };

                pt.update(cmd).await;
            }
        }

        pt.kill_current().await;
    });

    let twitter_clone = twitter.clone();
    tokio::spawn(async move {
        // pulling all the redeems from twitch
        let mut c = franz_client::FranzConsumer::new("tits.franz.mostlymaxi.com:8085", "redeem")
            .await
            .unwrap();

        while let Some(msg) = select! {
            _ = twitter_clone.cancelled() => None,
            m = c.recv() => m

        } {
            // update latest redeem
            msg_tx.send(msg.ok()).unwrap();
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    twitter.cancel();
}
