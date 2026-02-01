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
fn check_and_handle_network(
    config: &APPConfigValidated,
    state: &mut DaemonState,
) -> Result<()> {
    // 1. 检查网络连接
    if core::network::check_network_connection(None, None).is_ok() {
        return Ok(());
    }

    // 2. 网络未连接，尝试登录
    warn!("网络未连接，尝试登录...");
    core::login::network_login(&config.username, &config.password)?;
    info!("✓ 登录成功");

    // 3. 获取当前 IP
    let current_ip = get_current_ip();

    // 4. 检测 IP 变化
    let ip_changed = state.last_ip_address.as_ref() != Some(&current_ip);

    // 5. 发送邮件通知
    if let Some(smtp) = &config.smtp {
        send_notification(smtp, &config.username, &current_ip, ip_changed)?;
    }

    // 6. 更新状态
    state.last_ip_address = Some(current_ip);

    Ok(())
}

fn get_current_ip() -> String {
    core::report::get_host_ip()
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
    use super::*;

    #[test]
    fn test_daemon_state_creation() {
        let state = DaemonState::new();
        assert!(state.last_ip_address.is_none());
    }

    #[test]
    fn test_get_current_ip_fallback() {
        // 测试 IP 获取（可能成功或失败）
        let ip = get_current_ip();
        assert!(!ip.is_empty());
    }
}
