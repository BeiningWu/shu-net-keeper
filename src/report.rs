use crate::error::{NetworkError, NetworkResult};
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
struct OnlineUserInfo {
    #[serde(rename = "userIp")]
    user_ip: Option<String>,
}

pub fn get_host_ip() -> NetworkResult<Option<String>> {
    debug!("开始获取主机 IP 地址...");

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;

    let url = "http://10.10.9.9/eportal/InterFace.do?method=getOnlineUserInfo";

    debug!("请求在线用户信息: {}", url);
    let response = client.get(url).send().map_err(|e| {
        error!("HTTP 请求失败: {}", e);
        NetworkError::RequestFailed(e.to_string())
    })?;

    debug!("解析 JSON 响应...");
    let data: OnlineUserInfo = response.json().map_err(|e| {
        error!("解析 JSON 失败: {}", e);
        NetworkError::ParseFailed(e.to_string())
    })?;

    match &data.user_ip {
        Some(ip) => {
            info!("成功获取主机 IP: {}", ip);
        }
        None => {
            debug!("未获取到 IP 地址（可能未登录）");
        }
    }

    Ok(data.user_ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_get_success() {
        let result = get_host_ip();

        assert!(result.is_ok());

        let host_ip = result.unwrap();

        match host_ip {
            Some(ip) => {
                print!("本机地址: {}", ip);
                assert!(!ip.is_empty(), "IP 地址不应该为空");
            }
            None => {
                print!("未获取到IP地址");
            }
        }
    }
}
