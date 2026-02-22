use crate::constants::ONLINE_INFO_URL;
use crate::error::{NetworkError, NetworkResult};
use serde::Deserialize;
use tracing::{debug, error, info};

/// 在线用户信息
#[derive(Debug, Deserialize)]
pub(crate) struct OnlineUserInfo {
    #[serde(rename = "userIp")]
    user_ip: Option<String>,
}

/// 查询在线用户信息（网络请求 + 解析）
/// 返回值：
/// - Ok(Some(info)): 成功获取到响应（无论是否已登录）
/// - Err: 网络错误
fn query_online_info() -> NetworkResult<OnlineUserInfo> {
    debug!("请求在线用户信息: {}", ONLINE_INFO_URL);

    let agent = ureq::agent();
    let response = agent.get(ONLINE_INFO_URL).call().map_err(|e| {
        error!("请求在线用户信息失败: {}", e);
        if is_connection_error(&e) {
            NetworkError::NotConnected("未连接到校园网".to_string())
        } else {
            NetworkError::RequestFailed(e.to_string())
        }
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

    let info = serde_json::from_str::<OnlineUserInfo>(&body).map_err(|e| {
        error!("解析 JSON 响应失败: {}", e);
        NetworkError::ParseFailed(e.to_string())
    })?;

    Ok(info)
}

/// 获取主机 IP 地址
/// 返回值：
/// - Ok(Some(ip)): 已登录，返回用户 IP
/// - Ok(None): 未登录（userIp 为 null）
/// - Err: 网络错误
pub fn get_host_ip() -> NetworkResult<Option<String>> {
    debug!("开始获取主机 IP 地址...");

    let info = query_online_info()?;

    match info.user_ip {
        Some(ip) => {
            info!("成功获取主机 IP: {}", ip);
            Ok(Some(ip))
        }
        None => {
            debug!("用户未登录校园网");
            Ok(None)
        }
    }
}

/// 检查是否是连接错误（未联网）
fn is_connection_error(err: &ureq::Error) -> bool {
    match err {
        ureq::Error::Transport(transport) => {
            matches!(
                transport.kind(),
                ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Dns
            )
        }
        ureq::Error::Status(code, _) => *code == 0 || *code == 502 || *code == 504,
    }
}

/// 检查网络连接状态
/// 返回值：
/// - Ok(true): 已连接且已登录
/// - Ok(false): 未登录（网络可达但用户未登录）
/// - Err: 网络错误（无法连接到校园网）
///
/// 若已登录且 `ip_status` 为 None，则将当前 IP 写入 `ip_status`，
/// 用于在程序首次启动时就记录基准 IP，以便后续正确检测 IP 变化。
pub fn check_network_connection(ip_status: &mut Option<String>) -> Result<bool, NetworkError> {
    match get_host_ip() {
        Ok(Some(ip)) => {
            if ip_status.is_none() {
                *ip_status = Some(ip);
            }
            Ok(true)
        }
        Ok(None) => Ok(false), // 未登录
        Err(e) => Err(e),      // 网络错误
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
                        let json: Value =
                            serde_json::from_str(&content).unwrap_or(Value::String(content));
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                    Err(e) => println!("解析响应体失败: {:?}", e),
                }
            }
            Err(e) => println!("请求失败: {:?}", e),
        }
    }
}
