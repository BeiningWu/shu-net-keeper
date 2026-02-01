mod config;
mod email;
mod login;
mod network;
mod report;

use chrono::Local;
use std::thread;
use std::time::Duration;

fn main() {
    let config = config::load_config().unwrap_or_else(|e| {
        eprintln!("✗ {}", e);
        std::process::exit(1);
    });

    let uid = config.username;
    let pwd = config.password;

    // println!("Hello, world!");
    // network::check_nerwork_connection();
    run_daemon();
}

fn run_daemon() {
    loop {
        let now = Local::now();

        if !network::check_network_connection(None, None) {}

        thread::sleep(Duration::from_secs(300));
    }
}
