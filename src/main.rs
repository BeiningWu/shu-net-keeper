mod config;
mod constants;
mod core;
mod daemon;
mod error;
mod logger;

use error::Result;
use tracing::{error, info};

fn main() {
    if let Err(e) = run() {
        error!("程序运行失败: {}", e);
        eprintln!("✗ {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // 初始化日志系统
    logger::init().map_err(|e| {
        eprintln!("✗ 日志系统初始化失败: {}", e);
        error::AppError::Other(format!("日志系统初始化失败: {}", e))
    })?;

    info!("========== SHU 网络守护程序启动 ==========");

    // 加载配置
    let config = config::load_config()?;
    info!("配置加载成功，用户: {}", config.username);
    info!("检查间隔: {} 秒", config.interval);

    // 运行守护进程
    daemon::run(config)
}
