use crate::error::{ConfigError, ConfigResult, ValidationError};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info};
use validator::validate_email;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct APPConfig {
    /// 用户名（学号）
    pub username: String,

    /// 密码
    pub password: String,

    /// 检查间隔（秒）
    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default)]
    pub smtp_enabled: bool,

    #[serde(default)]
    pub smtp: Option<SmtpConfig>,
}

fn default_interval() -> u64 {
    600
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmtpConfig {
    pub server: Option<String>,
    pub port: Option<u16>,
    pub sender: Option<String>,
    pub password: Option<String>,
    pub receiver: Option<String>,
}

// ✅ 验证后的配置（所有字段都不是 Option）
pub struct APPConfigValidated {
    pub username: String,
    pub password: String,
    pub interval: u64,
    pub smtp: Option<SmtpConfigValidated>, // ✅ 如果 enabled = false，这里是 None
}

pub struct SmtpConfigValidated {
    pub server: String,
    pub port: u16,
    pub sender: String,
    pub password: String,
    pub receiver: String,
}

impl SmtpConfig {
    pub fn validate_if_enabled(&self) -> ConfigResult<SmtpConfigValidated> {
        // 验证服务器
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| ValidationError::MissingField("SMTP 服务器".to_string()))?;

        if server.is_empty() {
            return Err(ValidationError::EmptyField("SMTP 服务器".to_string()).into());
        }

        // 验证端口
        let port = self
            .port
            .ok_or_else(|| ValidationError::MissingField("SMTP 端口".to_string()))?;

        if port == 0 {
            return Err(ValidationError::InvalidPort(port).into());
        }

        // 验证发件人邮箱
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| ValidationError::MissingField("发件人邮箱".to_string()))?;

        if !validate_email(sender) {
            return Err(ValidationError::InvalidEmail(sender.clone()).into());
        }

        // 验证密码
        let password = self
            .password
            .as_ref()
            .ok_or_else(|| ValidationError::MissingField("SMTP 密码".to_string()))?;

        if password.is_empty() {
            return Err(ValidationError::EmptyField("SMTP 密码".to_string()).into());
        }

        // 验证接收邮箱
        let receiver = self
            .receiver
            .as_ref()
            .ok_or_else(|| ValidationError::MissingField("接收邮箱".to_string()))?;

        if !validate_email(receiver) {
            return Err(ValidationError::InvalidEmail(receiver.clone()).into());
        }

        Ok(SmtpConfigValidated {
            server: server.clone(),
            port,
            sender: sender.clone(),
            password: password.clone(),
            receiver: receiver.clone(),
        })
    }
}

fn get_config_path() -> PathBuf {
    match env::current_exe() {
        Ok(exe_path) => match exe_path.parent() {
            Some(exe_dir) => exe_dir.join("config.toml"),
            None => PathBuf::from("config.toml"),
        },
        Err(_) => PathBuf::from("config.toml"),
    }
}

pub fn load_config() -> ConfigResult<APPConfigValidated> {
    let config_path = get_config_path();

    debug!("配置文件路径: {}", config_path.display());

    if !config_path.exists() {
        error!("配置文件不存在: {}", config_path.display());
        return Err(ConfigError::FileNotFound {
            path: config_path.display().to_string(),
        });
    }

    debug!("正在读取配置文件...");
    let content = fs::read_to_string(&config_path).map_err(|e| {
        error!("读取配置文件失败: {}", e);
        ConfigError::ReadFailed(e.to_string())
    })?;

    debug!("正在解析配置文件...");
    let config: APPConfig = toml::from_str(&content).map_err(|e| {
        error!("解析配置文件失败: {}", e);
        ConfigError::ParseFailed(e.to_string())
    })?;

    info!("配置文件解析成功，正在验证...");
    validate_config(&config)
}

fn validate_config(config: &APPConfig) -> ConfigResult<APPConfigValidated> {
    debug!("开始验证配置...");

    // 验证用户名
    if config.username.len() != 8 {
        error!(
            "用户名长度验证失败: 期望8位，实际{}位",
            config.username.len()
        );
        return Err(ValidationError::InvalidUsername(format!(
            "用户名必须是8位，当前为{}位",
            config.username.len()
        ))
        .into());
    }

    if !config.username.chars().all(|c| c.is_ascii_digit()) {
        error!("用户名格式验证失败: 包含非数字字符");
        return Err(ValidationError::InvalidUsername("用户名必须全部是数字".to_string()).into());
    }

    debug!("用户名验证通过: {}", config.username);

    // 验证密码
    if config.password.is_empty() {
        error!("密码验证失败: 密码为空");
        return Err(ValidationError::EmptyField("密码".to_string()).into());
    }

    debug!("密码验证通过");

    // ✅ 关键逻辑：根据 smtp_enabled 决定是否验证 SMTP
    let validated_smtp = if config.smtp_enabled {
        info!("SMTP 已启用，验证 SMTP 配置...");
        // ✅ 启用了 SMTP，必须验证
        match &config.smtp {
            Some(smtp) => {
                // 验证 SMTP 配置
                let validated = smtp.validate_if_enabled()?;
                info!("SMTP 配置验证通过");
                Some(validated)
            }
            None => {
                // ❌ 启用了但没有配置 SMTP
                error!("SMTP 已启用但未配置 [smtp] 部分");
                return Err(ConfigError::SmtpConfig(
                    "SMTP 已启用但未配置 [smtp] 部分".to_string(),
                ));
            }
        }
    } else {
        info!("SMTP 未启用，跳过邮件通知");
        // ✅ 未启用 SMTP，返回 None
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============ SmtpConfig 验证测试 ============

    #[test]
    fn test_smtp_valid() {
        let smtp = SmtpConfig {
            server: Some("smtp.qq.com".to_string()),
            port: Some(587),
            sender: Some("test@qq.com".to_string()),
            password: Some("auth_code".to_string()),
            receiver: Some("notify@example.com".to_string()),
        };

        assert!(smtp.validate_if_enabled().is_ok());
    }

    #[test]
    fn test_smtp_invalid_email() {
        let smtp = SmtpConfig {
            server: Some("smtp.qq.com".to_string()),
            port: Some(587),
            sender: Some("not_an_email".to_string()), // ❌ 无效
            password: Some("auth".to_string()),
            receiver: Some("notify@example.com".to_string()),
        };

        assert!(smtp.validate_if_enabled().is_err());
    }

    #[test]
    fn test_smtp_missing_field() {
        let smtp = SmtpConfig {
            server: None, // ❌ 缺失
            port: Some(587),
            sender: Some("test@qq.com".to_string()),
            password: Some("auth".to_string()),
            receiver: Some("test@example.com".to_string()),
        };

        assert!(smtp.validate_if_enabled().is_err());
    }

    #[test]
    fn test_smtp_invalid_port() {
        let smtp = SmtpConfig {
            server: Some("smtp.qq.com".to_string()),
            port: Some(0), // ❌ 无效端口
            sender: Some("test@qq.com".to_string()),
            password: Some("auth".to_string()),
            receiver: Some("notify@example.com".to_string()),
        };

        assert!(smtp.validate_if_enabled().is_err());
    }

    // ============ APPConfig 验证测试 ============

    #[test]
    fn test_config_valid_no_smtp() {
        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: false,
            smtp: None,
        };

        let validated = validate_config(&config).unwrap();
        assert_eq!(validated.username, "12345678");
        assert!(validated.smtp.is_none());
    }

    #[test]
    fn test_config_valid_with_smtp() {
        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true,
            smtp: Some(SmtpConfig {
                server: Some("smtp.qq.com".to_string()),
                port: Some(587),
                sender: Some("test@qq.com".to_string()),
                password: Some("auth".to_string()),
                receiver: Some("notify@example.com".to_string()),
            }),
        };

        let validated = validate_config(&config).unwrap();
        assert_eq!(validated.username, "12345678");
        assert!(validated.smtp.is_some());

        let smtp = validated.smtp.unwrap();
        assert_eq!(smtp.server, "smtp.qq.com");
        assert_eq!(smtp.port, 587);
    }

    #[test]
    fn test_config_smtp_enabled_but_missing() {
        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true, // ❌ 启用了
            smtp: None,         // ❌ 但没配置
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_config_invalid_username_length() {
        let config = APPConfig {
            username: "123".to_string(), // ❌ 不是8位
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: false,
            smtp: None,
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_config_invalid_username_format() {
        let config = APPConfig {
            username: "1234567a".to_string(), // ❌ 包含字母
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: false,
            smtp: None,
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_config_empty_password() {
        let config = APPConfig {
            username: "12345678".to_string(),
            password: "".to_string(), // ❌ 空密码
            interval: 600,
            smtp_enabled: false,
            smtp: None,
        };

        assert!(validate_config(&config).is_err());
    }
}
