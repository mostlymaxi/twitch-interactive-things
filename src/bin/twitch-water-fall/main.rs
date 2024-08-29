use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use twitch_interactive_core::*;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let latest_info_wrapper = LatestStreamInfo::new("/tmp/strim-mmap-test.bin");
    let latest_info = unsafe { latest_info_wrapper.get_inner() };

    let mut c = Command::new("cvlc")
        .arg("./waterfall-176958.mp3")
        .arg("-R")
        .arg("-I")
        .arg("rc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut c_in = c.stdin.take().unwrap();
    c_in.write_all("volume\n".as_bytes()).unwrap();
    c_in.write_all("volume 100\n".as_bytes()).unwrap();
    let mut volume: u8;

    let mut waters: f64;
    let max_waters: f64 = 100.0;
    let water_scaling = 2.55;

    while running.load(Ordering::Relaxed) {
        waters = max_waters.min(latest_info.waters_per_10m as f64);
        volume = (waters * water_scaling).floor() as u8;

        c_in.write_all(format!("volume {}\n", volume).as_bytes())
            .unwrap();

        thread::sleep(Duration::from_secs(1));
    }
}
