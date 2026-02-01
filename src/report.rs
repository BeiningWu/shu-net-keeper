use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct OnlineUserInfo {
    #[serde(rename = "userIp")]
    user_ip: Option<String>,
}

pub fn get_host_ip() -> Result<Option<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("无法创建 HTTP 客户端");

    let url = "http://10.10.9.9/eportal/InterFace.do?method=getOnlineUserInfo";

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;
    let data: OnlineUserInfo = response
        .json()
        .map_err(|e| format!("解析 JSON 请求失败: {}", e))?;

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
