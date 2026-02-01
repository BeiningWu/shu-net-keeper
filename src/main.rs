mod config;
mod email;
mod login;
mod network;
mod report;

use std::thread;
use std::time::Duration;

fn main() {
    let config = config::load_config().unwrap_or_else(|e| {
        eprintln!("✗ {}", e);
        std::process::exit(1);
    });

    run_daemon(config);
}

fn run_daemon(appconfig: config::APPConfigValidated) {
    loop {
        // let now = Local::now();

        if !network::check_network_connection(None, None) {
            let login_result = login::network_login(&appconfig.username, &appconfig.password);
            match login_result {
                Ok(()) => {
                    if let Some(smtp) = &appconfig.smtp {
                        match email::send_login_notification(
                            smtp,
                            &appconfig.username,
                            "192.168.1.1",
                        ) {
                            Ok(()) => {}
                            Err(e) => {
                                print!("{}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    print!("{}", e);
                }
            }
        }

        thread::sleep(Duration::from_secs(300));
    }
}
