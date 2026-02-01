use crate::constants::{HEALTH_CHECK_URL, NETWORK_CHECK_RETRIES, NETWORK_CHECK_TIMEOUT_SECS};
use crate::error::{NetworkError, NetworkResult};
use crate::http_client::HttpClientFactory;
use std::time::Duration;
use tracing::{debug, warn};

pub fn check_network_connection(
    url: Option<&str>,
    timeout_sec: Option<u64>,
) -> NetworkResult<()> {
    let url = url.unwrap_or(HEALTH_CHECK_URL);
    let timeout_sec = timeout_sec.unwrap_or(NETWORK_CHECK_TIMEOUT_SECS);
    let retries = NETWORK_CHECK_RETRIES;

    debug!("检查网络连接，目标: {}, 超时: {}秒", url, timeout_sec);

    // 使用工厂创建网络检查客户端
    let client = HttpClientFactory::new_network_check(timeout_sec)
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

    for attempt in 1..=retries {
        debug!("网络连接检查尝试 {}/{}", attempt, retries);

        match client.get(url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("网络连接正常，状态码: {}", response.status());
                    return Ok(());
                } else {
                    debug!("收到响应但状态码异常: {}", response.status());
                    if attempt == retries {
                        return Err(NetworkError::ResponseError {
                            status: response.status().as_u16(),
                            message: format!("状态码: {}", response.status()),
                        });
                    }
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    debug!("第 {} 次连接尝试超时", attempt);
                    if attempt == retries {
                        return Err(NetworkError::Timeout(format!(
                            "连接超时，已重试 {} 次",
                            retries
                        )));
                    }
                } else if e.is_connect() {
                    debug!("第 {} 次连接尝试失败: 连接错误", attempt);
                    if attempt == retries {
                        return Err(NetworkError::ConnectionFailed(e.to_string()));
                    }
                } else {
                    debug!("第 {} 次连接尝试失败", attempt);
                    if attempt == retries {
                        return Err(NetworkError::RequestFailed(e.to_string()));
                    }
                }
            }
        }

        if attempt < retries {
            debug!("等待 {} 秒后重试...", timeout_sec);
            std::thread::sleep(Duration::from_secs(timeout_sec));
        }
    }

    warn!("网络连接检查失败，已重试 {} 次", retries);
    Err(NetworkError::Unreachable(format!(
        "无法连接到 {}，已重试 {} 次",
        url, retries
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[test]
    fn test_network_connection_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("ok")
            .create();
        let result_mock = check_network_connection(Some(&server.url()), None);
        assert!(result_mock.is_ok());
        mock.assert();

        let result_baidu = check_network_connection(Some("https://www.baidu.com"), None);
        assert!(result_baidu.is_ok());
    }

    #[test]
    fn test_network_connection_failure() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/").with_status(400).create();

        let result = check_network_connection(Some(&server.url()), None);

        assert!(result.is_err());
    }

    #[test]
    fn test_network_connection_timeout() {
        let result = check_network_connection(Some("http://192.0.2.1:9999"), Some(1));

        assert!(result.is_err());
    }
}
