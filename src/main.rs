mod config;
mod email;
mod error;
mod logger;
mod login;
mod network;
mod report;

use error::Result;
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

fn main() {
    if let Err(e) = run() {
        error!("程序运行失败: {}", e);
        eprintln!("✗ {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // 初始化日志系统
    if let Err(e) = logger::init() {
        eprintln!("✗ 日志系统初始化失败: {}", e);
        return Err(error::AppError::Other(format!("日志系统初始化失败: {}", e)));
    }

    info!("========== SHU 网络守护程序启动 ==========");

    let config = config::load_config()?;

    info!("配置加载成功，用户: {}", config.username);
    info!("检查间隔: {} 秒", config.interval);

    run_daemon(config)
}

fn run_daemon(appconfig: config::APPConfigValidated) -> Result<()> {
    let mut last_ip_address: Option<String> = None;

    loop {
        info!("正在检查网络连接状态...");

        // 检查网络连接
        match network::check_network_connection(None, None) {
            Ok(()) => {
                info!("✓ 网络连接正常");
            }
            Err(_) => {
                warn!("网络未连接，尝试登录...");

                match login::network_login(&appconfig.username, &appconfig.password) {
                    Ok(()) => {
                        info!("✓ 登录成功");

                        // 获取主机 IP 地址
                        let current_ip = report::get_host_ip().ok().flatten().unwrap_or_else(|| {
                            warn!("获取主机 IP 失败，使用默认值");
                            "未知".to_string()
                        });

                        // 检查 IP 地址是否有变化
                        let ip_changed = last_ip_address.as_ref() != Some(&current_ip);

                        // 发送邮件通知
                        if let Some(smtp) = &appconfig.smtp {
                            info!("准备发送登录通知邮件...");
                            if let Err(e) = email::send_login_notification(
                                smtp,
                                &appconfig.username,
                                &current_ip,
                                ip_changed,
                            ) {
                                error!("✗ 邮件发送失败: {}", e);
                            } else {
                                info!("✓ 邮件通知发送成功");
                            }
                        }

                        // 更新上一次的 IP 地址
                        last_ip_address = Some(current_ip);
                    }
                    Err(e) => {
                        error!("✗ 登录失败: {}", e);
                    }
                }
            }
        }

        info!("等待 {} 秒后再次检查...\n", appconfig.interval);
        thread::sleep(Duration::from_secs(appconfig.interval));
    }
}
