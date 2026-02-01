use thiserror::Error;

/// 应用程序的统一错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),

    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),

    #[error("登录错误: {0}")]
    Login(#[from] LoginError),

    #[error("邮件错误: {0}")]
    Email(#[from] EmailError),

    #[error("验证错误: {0}")]
    Validation(#[from] ValidationError),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// 配置错误类型
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件不存在: {path}\n请先运行 --init 创建配置文件")]
    FileNotFound { path: String },

    #[error("读取配置文件失败: {0}")]
    ReadFailed(String),

    #[error("解析配置文件失败: {0}")]
    ParseFailed(String),

    #[error("配置验证失败: {0}")]
    ValidationFailed(String),

    #[error("SMTP 配置错误: {0}")]
    SmtpConfig(String),
}

/// 网络错误类型
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("请求超时: {0}")]
    Timeout(String),

    #[error("请求失败: {0}")]
    RequestFailed(String),

    #[error("响应错误 [{status}]: {message}")]
    ResponseError { status: u16, message: String },

    #[error("网络不可达: {0}")]
    Unreachable(String),

    #[error("解析响应失败: {0}")]
    ParseFailed(String),
}

/// 登录错误类型
#[derive(Error, Debug)]
pub enum LoginError {
    #[error("获取登录参数失败: {0}")]
    QueryStringFailed(String),

    #[error("创建 HTTP 客户端失败: {0}")]
    ClientCreationFailed(String),

    #[error("登录请求失败: {0}")]
    RequestFailed(String),

    #[error("登录响应解析失败: {0}")]
    ResponseParseFailed(String),

    #[error("登录失败 [{status}]: {message}")]
    AuthenticationFailed { status: u16, message: String },

    #[error("URL 解析失败: {0}")]
    UrlParseFailed(String),
}

/// 邮件错误类型
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("发件人地址无效: {0}")]
    InvalidSender(String),

    #[error("收件人地址无效: {0}")]
    InvalidReceiver(String),

    #[error("创建邮件失败: {0}")]
    MessageCreationFailed(String),

    #[error("连接 SMTP 服务器失败: {0}")]
    SmtpConnectionFailed(String),

    #[error("发送邮件失败: {0}")]
    SendFailed(String),
}

/// 数据验证错误类型
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("邮箱格式不正确: {0}")]
    InvalidEmail(String),

    #[error("用户名格式不正确: {0}")]
    InvalidUsername(String),

    #[error("端口号无效: {0}")]
    InvalidPort(u16),

    #[error("缺少必填字段: {0}")]
    MissingField(String),

    #[error("字段不能为空: {0}")]
    EmptyField(String),
}

// ==================== From 转换实现 ====================

// 为 reqwest::Error 提供转换
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Network(NetworkError::Timeout(err.to_string()))
        } else if err.is_connect() {
            AppError::Network(NetworkError::ConnectionFailed(err.to_string()))
        } else if err.is_request() {
            AppError::Network(NetworkError::RequestFailed(err.to_string()))
        } else {
            AppError::Network(NetworkError::RequestFailed(err.to_string()))
        }
    }
}

// 为 ValidationError 提供转换为 ConfigError
impl From<ValidationError> for ConfigError {
    fn from(err: ValidationError) -> Self {
        ConfigError::ValidationFailed(err.to_string())
    }
}

// 兼容现有的 String 错误类型
impl From<String> for AppError {
    fn from(err: String) -> Self {
        // 尝试根据错误信息判断类型
        if err.contains("配置") || err.contains("config") {
            AppError::Config(ConfigError::ValidationFailed(err))
        } else if err.contains("网络") || err.contains("连接") || err.contains("network") {
            AppError::Network(NetworkError::RequestFailed(err))
        } else if err.contains("登录") || err.contains("login") {
            AppError::Login(LoginError::RequestFailed(err))
        } else if err.contains("邮件") || err.contains("email") || err.contains("smtp") {
            AppError::Email(EmailError::SendFailed(err))
        } else {
            AppError::Other(err)
        }
    }
}

// ==================== Result 类型别名 ====================

/// 应用程序通用的 Result 类型
pub type Result<T> = std::result::Result<T, AppError>;

/// 配置相关的 Result 类型
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// 网络相关的 Result 类型
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// 登录相关的 Result 类型
pub type LoginResult<T> = std::result::Result<T, LoginError>;

/// 邮件相关的 Result 类型
pub type EmailResult<T> = std::result::Result<T, EmailError>;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::FileNotFound {
            path: "config.toml".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("配置文件不存在"));
        assert!(display.contains("config.toml"));
    }

    #[test]
    fn test_network_error_display() {
        let error = NetworkError::ResponseError {
            status: 404,
            message: "Not Found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("404"));
        assert!(display.contains("Not Found"));
    }

    #[test]
    fn test_app_error_from_config_error() {
        let config_err = ConfigError::ParseFailed("invalid toml".to_string());
        let app_err: AppError = config_err.into();

        match app_err {
            AppError::Config(_) => (),
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError::InvalidEmail("not-an-email".to_string());
        let display = format!("{}", error);
        assert!(display.contains("邮箱格式不正确"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_from_string() {
        let err: AppError = "配置错误".to_string().into();
        assert!(matches!(err, AppError::Config(_)));

        let err: AppError = "网络连接失败".to_string().into();
        assert!(matches!(err, AppError::Network(_)));
    }
}
