use crate::constants::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;

/// HTTP 客户端工厂
pub struct HttpClientFactory;

impl HttpClientFactory {
    /// 创建通用客户端（用于网络检查、IP查询）
    pub fn new_default() -> Result<Client, reqwest::Error> {
        Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent(USER_AGENT)
            .build()
    }

    /// 创建网络检查客户端（自定义超时）
    pub fn new_network_check(timeout_secs: u64) -> Result<Client, reqwest::Error> {
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(USER_AGENT)
            .build()
    }

    /// 创建登录客户端（带完整浏览器头）
    pub fn new_login_client(referer: &str) -> Result<Client, reqwest::Error> {
        let mut headers = Self::browser_headers();
        headers.insert(
            reqwest::header::REFERER,
            HeaderValue::from_str(referer).unwrap(),
        );

        Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .default_headers(headers)
            .build()
    }

    /// 标准浏览器请求头
    fn browser_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, USER_AGENT.parse().unwrap());
        headers.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());
        headers.insert(
            reqwest::header::ACCEPT_ENCODING,
            "gzip, deflate".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
        );
        headers.insert(reqwest::header::HOST, "10.10.9.9".parse().unwrap());
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_client() {
        let client = HttpClientFactory::new_default();
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_network_check_client() {
        let client = HttpClientFactory::new_network_check(5);
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_login_client() {
        let client = HttpClientFactory::new_login_client("http://test.com");
        assert!(client.is_ok());
    }
}
