use crate::config::APPConfigValidated;
use crate::{core, error::Result};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

/// 守护进程状态
pub struct DaemonState {
    last_ip_address: Option<String>,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            last_ip_address: None,
        }
    }
}

/// 守护进程主循环
pub fn run(config: APPConfigValidated) -> Result<()> {
    let mut state = DaemonState::new();

    loop {
        info!("正在检查网络连接状态...");

        match check_and_handle_network(&config, &mut state) {
            Ok(_) => info!("✓ 网络连接正常"),
            Err(e) => error!("处理网络状态失败: {}", e),
        }

        info!("等待 {} 秒后再次检查...\n", config.interval);
        thread::sleep(Duration::from_secs(config.interval));
    }
}

/// 检查网络并处理登录
fn check_and_handle_network(config: &APPConfigValidated, state: &mut DaemonState) -> Result<()> {
    // 1. 检查网络连接（已连接时顺带初始化 ip_status）
    if core::network::check_network_connection(&mut state.last_ip_address)? {
        return Ok(());
    }

    // 2. 网络未连接，尝试登录
    warn!("网络未连接，尝试登录...");
    core::login::network_login(&config.username, &config.password)?;
    info!("✓ 登录成功");

    // 3. 获取当前 IP
    let current_ip = get_current_ip();

    // 4. 检测 IP 变化（last_ip_address 为 None 表示首次登录，不视为 IP 变化）
    let ip_changed = matches!(&state.last_ip_address, Some(old) if old != &current_ip);

    // 5. 发送邮件通知
    if let Some(smtp) = &config.smtp {
        send_notification(smtp, &config.username, &current_ip, ip_changed)?;
    }

    // 6. 更新状态
    state.last_ip_address = Some(current_ip);

    Ok(())
}

fn get_current_ip() -> String {
    core::network::get_host_ip()
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            warn!("获取主机 IP 失败，使用默认值");
            "未知".to_string()
        })
}

fn send_notification(
    smtp: &crate::config::SmtpConfigValidated,
    username: &str,
    ip: &str,
    ip_changed: bool,
) -> Result<()> {
    info!("准备发送登录通知邮件...");
    core::email::send_login_notification(smtp, username, ip, ip_changed).map_err(|e| {
        error!("✗ 邮件发送失败: {}", e);
        e
    })?;
    info!("✓ 邮件通知发送成功");
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ip_changed_logic() {
        let current_ip = "192.168.1.1".to_string();

        // 首次登录（None）：不应视为 IP 变化
        let last: Option<String> = None;
        assert!(!matches!(&last, Some(old) if old != &current_ip));

        // 相同 IP：不应视为 IP 变化
        let last = Some("192.168.1.1".to_string());
        assert!(!matches!(&last, Some(old) if old != &current_ip));

        // IP 变化：应视为 IP 变化
        let last = Some("10.0.0.1".to_string());
        assert!(matches!(&last, Some(old) if old != &current_ip));
    }
}
