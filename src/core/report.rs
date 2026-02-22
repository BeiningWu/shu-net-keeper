use crate::constants::ONLINE_INFO_URL;
use crate::error::{NetworkError, NetworkResult};
use serde::Deserialize;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
struct OnlineUserInfo {
    #[serde(rename = "userIp")]
    user_ip: Option<String>,
}

pub fn get_host_ip() -> NetworkResult<Option<String>> {
    debug!("开始获取主机 IP 地址...");

    // 创建 agent
    let agent = ureq::agent();

    debug!("请求在线用户信息: {}", ONLINE_INFO_URL);
    let response = agent.get(ONLINE_INFO_URL).call().map_err(|e| {
        error!("请求在线用户信息失败: {}", e);
        NetworkError::RequestFailed(e.to_string())
    })?;

    let status = response.status();
    debug!("收到响应，状态码: {}", status);

    if !(status >= 200 && status < 300) {
        error!("获取在线用户信息失败，状态码: {}", status);
        return Err(NetworkError::ResponseError {
            status,
            message: format!("状态码: {}", status),
        });
    }

    let body = response.into_string().map_err(|e| {
        error!("读取响应内容失败: {}", e);
        NetworkError::ParseFailed(e.to_string())
    })?;

    debug!("响应内容: {}", body);

    match serde_json::from_str::<OnlineUserInfo>(&body) {
        Ok(info) => {
            if let Some(ip) = &info.user_ip {
                info!("成功获取主机 IP: {}", ip);
                Ok(Some(ip.clone()))
            } else {
                debug!("响应中没有 userIp 字段");
                Ok(None)
            }
        }
        Err(e) => {
            error!("解析 JSON 响应失败: {}", e);
            Err(NetworkError::ParseFailed(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_host_ip() {
        let result = get_host_ip();
        match result {
            Ok(Some(ip)) => println!("获取到 IP: {}", ip),
            Ok(None) => println!("未获取到 IP"),
            Err(e) => println!("获取 IP 失败: {:?}", e),
        }
    }

    #[test]
    fn get_online_user_info() {
        use serde_json::Value;

        let agent = ureq::agent();

        let response = agent.get(ONLINE_INFO_URL).call();

        match response {
            Ok(response) => {
                let content = response.into_string();
                match content {
                    Ok(content) => {
                        let json: Value = serde_json::from_str(&content).unwrap_or(Value::String(content));
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                    Err(e) => println!("解析响应体失败: {:?}", e),
                }
            }
            Err(e) => println!("请求失败: {:?}", e),
        }
    }
}
