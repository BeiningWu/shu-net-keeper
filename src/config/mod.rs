mod loader;
mod types;
mod validation;

pub use loader::load_config;
pub use types::{APPConfigValidated, SmtpConfigValidated};

#[cfg(test)]
mod tests {
    use super::types::*;
    use super::validation::validate_config;

    // ============ SmtpConfig 验证测试 ============

    #[test]
    fn test_smtp_valid() {
        let smtp = SmtpConfig {
            server: Some("smtp.qq.com".to_string()),
            port: Some(465),
            sender: Some("test@qq.com".to_string()),
            password: Some("auth_code".to_string()),
            receiver: Some("notify@example.com".to_string()),
        };

        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true,
            smtp: Some(smtp),
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_smtp_invalid_email() {
        let smtp = SmtpConfig {
            server: Some("smtp.qq.com".to_string()),
            port: Some(465),
            sender: Some("not_an_email".to_string()), // ❌ 无效
            password: Some("auth".to_string()),
            receiver: Some("notify@example.com".to_string()),
        };

        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true,
            smtp: Some(smtp),
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_smtp_missing_field() {
        let smtp = SmtpConfig {
            server: None, // ❌ 缺失
            port: Some(465),
            sender: Some("test@qq.com".to_string()),
            password: Some("auth".to_string()),
            receiver: Some("test@example.com".to_string()),
        };

        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true,
            smtp: Some(smtp),
        };

        assert!(validate_config(&config).is_err());
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

        let config = APPConfig {
            username: "12345678".to_string(),
            password: "testpass".to_string(),
            interval: 600,
            smtp_enabled: true,
            smtp: Some(smtp),
        };

        assert!(validate_config(&config).is_err());
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
                port: Some(465),
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
        assert_eq!(smtp.port, 465);
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
