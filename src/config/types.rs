use crate::constants::DEFAULT_CHECK_INTERVAL;
use serde::{Deserialize, Serialize};

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
    DEFAULT_CHECK_INTERVAL
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
