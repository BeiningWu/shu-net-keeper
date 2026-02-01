use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

use crate::config::SmtpConfigValidated;

pub fn send_email_with_config(
    smtp: &SmtpConfigValidated,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    // 获取配置（确保已验证）
    let server = &smtp.server;

    let username = &smtp.sender;

    let password = &smtp.password;

    let receiver = &smtp.receiver;

    // info!("准备发送邮件到: {}", receiver);

    // 创建邮件
    let email = Message::builder()
        .from(
            username
                .parse()
                .map_err(|e| format!("发件人地址无效: {}", e))?,
        )
        .to(receiver
            .parse()
            .map_err(|e| format!("收件人地址无效: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("创建邮件失败: {}", e))?;

    // 配置 SMTP
    let creds = Credentials::new(username.clone(), password.clone());

    let mailer = SmtpTransport::relay(&server)
        .map_err(|e| format!("连接 SMTP 服务器失败: {}", e))?
        .port(smtp.port)
        .credentials(creds)
        .build();

    // 发送邮件
    mailer.send(&email).map_err(|e| {
        // error!("发送邮件失败: {}", e);
        format!("发送邮件失败: {}", e)
    })?;

    // info!("✓ 邮件发送成功");
    Ok(())
}

/// 发送登录通知邮件
pub fn send_login_notification(
    smtp: &SmtpConfigValidated,
    username: &str,
    ip: &str,
) -> Result<(), String> {
    let subject = "校园网登录通知";
    let body = format!(
        "您的账号已成功登录校园网\n\n\
        用户名: {}\n\
        IP 地址: {}\n\
        登录时间: {}\n\n\
        如非本人操作，请及时修改密码。",
        username,
        ip,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    send_email_with_config(smtp, subject, &body)
}
