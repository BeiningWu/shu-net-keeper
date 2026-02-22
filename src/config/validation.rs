use crate::config::types::*;
use crate::constants::REQUIRED_USERNAME_LENGTH;
use crate::error::{ConfigError, ConfigResult, ValidationError};
use tracing::{debug, error, info};
use validator::validate_email;

/// 统一的 Option 字段验证器
struct FieldValidator;

impl FieldValidator {
    fn require_string(value: &Option<String>, field_name: &str) -> ConfigResult<String> {
        let s = value
            .as_ref()
            .ok_or_else(|| ValidationError::MissingField(field_name.to_string()))?;

        if s.is_empty() {
            return Err(ValidationError::EmptyField(field_name.to_string()).into());
        }

        Ok(s.clone())
    }

    fn require_email(value: &Option<String>, field_name: &str) -> ConfigResult<String> {
        let email = Self::require_string(value, field_name)?;

        if !validate_email(&email) {
            return Err(ValidationError::InvalidEmail(email).into());
        }

        Ok(email)
    }

    fn require_port(value: Option<u16>, field_name: &str) -> ConfigResult<u16> {
        let port = value.ok_or_else(|| ValidationError::MissingField(field_name.to_string()))?;

        if port == 0 {
            return Err(ValidationError::InvalidPort(port).into());
        }

        Ok(port)
    }
}

pub fn validate_config(config: &APPConfig) -> ConfigResult<APPConfigValidated> {
    debug!("开始验证配置...");

    validate_username(&config.username)?;
    validate_password(&config.password)?;

    let validated_smtp = if config.smtp_enabled {
        info!("SMTP 已启用，验证 SMTP 配置...");
        Some(validate_smtp_config(config.smtp.as_ref())?)
    } else {
        info!("SMTP 未启用，跳过邮件通知");
        None
    };

    info!("配置验证完成");

    Ok(APPConfigValidated {
        username: config.username.clone(),
        password: config.password.clone(),
        interval: config.interval,
        smtp: validated_smtp,
    })
}

fn validate_username(username: &str) -> ConfigResult<()> {
    if username.len() != REQUIRED_USERNAME_LENGTH {
        error!(
            "用户名长度验证失败: 期望{}位，实际{}位",
            REQUIRED_USERNAME_LENGTH,
            username.len()
        );
        return Err(ValidationError::InvalidUsername(format!(
            "用户名必须是{}位，当前为{}位",
            REQUIRED_USERNAME_LENGTH,
            username.len()
        ))
        .into());
    }

    if !username.chars().all(|c| c.is_ascii_digit()) {
        error!("用户名格式验证失败: 包含非数字字符");
        return Err(ValidationError::InvalidUsername("用户名必须全部是数字".to_string()).into());
    }

    debug!("用户名验证通过: {}", username);
    Ok(())
}

fn validate_password(password: &str) -> ConfigResult<()> {
    if password.is_empty() {
        error!("密码验证失败: 密码为空");
        return Err(ValidationError::EmptyField("密码".to_string()).into());
    }
    debug!("密码验证通过");
    Ok(())
}

fn validate_smtp_config(smtp: Option<&SmtpConfig>) -> ConfigResult<SmtpConfigValidated> {
    let smtp =
        smtp.ok_or_else(|| ConfigError::SmtpConfig("SMTP 已启用但未配置 [smtp] 部分".to_string()))?;

    Ok(SmtpConfigValidated {
        server: FieldValidator::require_string(&smtp.server, "SMTP 服务器")?,
        port: FieldValidator::require_port(smtp.port, "SMTP 端口")?,
        sender: FieldValidator::require_email(&smtp.sender, "发件人邮箱")?,
        password: FieldValidator::require_string(&smtp.password, "SMTP 密码")?,
        receiver: FieldValidator::require_email(&smtp.receiver, "接收邮箱")?,
    })
}
