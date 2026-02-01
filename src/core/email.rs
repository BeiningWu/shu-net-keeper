use crate::config::SmtpConfigValidated;
use crate::error::{EmailError, EmailResult};
use gethostname::gethostname;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use tracing::{debug, error, info};

fn send_email_with_config(
    smtp: &SmtpConfigValidated,
    subject: &str,
    body: &str,
) -> EmailResult<()> {
    debug!("准备发送邮件...");

    // 获取配置（确保已验证）
    let server = &smtp.server;
    let username = &smtp.sender;
    let password = &smtp.password;
    let receiver = &smtp.receiver;

    info!("发送邮件到: {}", receiver);
    debug!("SMTP 服务器: {}:{}", server, smtp.port);

    // 创建邮件
    debug!("构建邮件消息...");
    let email = Message::builder()
        .from(username.parse().map_err(|e| {
            error!("发件人地址无效 {}: {}", username, e);
            EmailError::InvalidSender(username.to_string())
        })?)
        .to(receiver.parse().map_err(|e| {
            error!("收件人地址无效 {}: {}", receiver, e);
            EmailError::InvalidReceiver(receiver.to_string())
        })?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| {
            error!("创建邮件失败: {}", e);
            EmailError::MessageCreationFailed(e.to_string())
        })?;

    debug!("邮件消息构建成功");

    // 配置 SMTP
    debug!("配置 SMTP 传输...");
    let creds = Credentials::new(username.clone(), password.clone());

    let mailer = SmtpTransport::relay(&server)
        .map_err(|e| {
            error!("连接 SMTP 服务器失败 {}: {}", server, e);
            EmailError::SmtpConnectionFailed(format!("{}:{}", server, smtp.port))
        })?
        .port(smtp.port)
        .credentials(creds)
        .build();

    debug!("SMTP 传输配置完成");

    // 发送邮件
    debug!("正在发送邮件...");
    mailer.send(&email).map_err(|e| {
        error!("发送邮件失败: {}", e);
        EmailError::SendFailed(e.to_string())
    })?;

    info!("✓ 邮件发送成功");
    Ok(())
}

/// 发送登录通知邮件
pub fn send_login_notification(
    smtp: &SmtpConfigValidated,
    username: &str,
    ip: &str,
    ip_changed: bool,
) -> EmailResult<()> {
    info!("准备发送登录通知邮件，用户: {}", username);

    // 获取本地主机名
    let hostname = gethostname()
        .to_string_lossy()
        .to_string();

    let subject = if ip_changed {
        "校园网登录通知 - IP地址变更"
    } else {
        "校园网登录通知"
    };

    let body = if ip_changed {
        format!(
            "您的账号已成功登录校园网\n\n            主机名: {}\n            IP 地址: {}\n            登录时间: {}\n\n            ⚠️  注意: IP地址已变更，如非本人操作，请及时修改密码。",
            hostname,
            ip,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    } else {
        format!(
            "您的账号已成功登录校园网\n\n            主机名: {}\n            IP 地址: {}\n            登录时间: {}\n\n            如非本人操作，请及时修改密码。",
            hostname,
            ip,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    };

    debug!("邮件内容已构建，主题: {}", subject);

    send_email_with_config(smtp, subject, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_login_notification() {
        // TODO: 请填写 SMTP 配置信息
        let smtp_config = SmtpConfigValidated {
            server: "smtp.example.com".to_string(),
            port: 587,
            sender: "your-email@example.com".to_string(),
            password: "your-smtp-password".to_string(),
            receiver: "recipient@example.com".to_string(),
        };

        // 测试 IP 地址未变化的情况
        println!("测试 IP 地址未变化的情况...");
        match send_login_notification(&smtp_config, "testuser", "192.168.1.1", false) {
            Ok(()) => println!("✓ IP 未变化: 邮件发送成功"),
            Err(e) => println!("✗ IP 未变化: 邮件发送失败: {}", e),
        }

        // 测试 IP 地址变化的情况
        println!("测试 IP 地址变化的情况...");
        match send_login_notification(&smtp_config, "testuser", "192.168.1.2", true) {
            Ok(()) => println!("✓ IP 已变化: 邮件发送成功"),
            Err(e) => println!("✗ IP 已变化: 邮件发送失败: {}", e),
        }
    }
}
