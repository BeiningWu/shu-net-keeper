use crate::config::types::*;
use crate::config::validation::validate_config;
use crate::error::{ConfigError, ConfigResult};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error};

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

    validate_config(&config)
}
