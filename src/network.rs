use std::time::Duration;

pub fn get_ip_address() {}

pub fn check_network_connection(url: Option<&str>, timeout_sec: Option<u64>) -> bool {
    let url = url.unwrap_or("https://www.baidu.com");
    let timeout_sec = timeout_sec.unwrap_or(5);
    let retries = 5;
    // 设置超时时间
    let timeout = Duration::from_secs(timeout_sec);

    // 创建 HTTP 客户端
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap();

    for attempt in 1..=retries {
        if let Ok(response) = client.get(url).send() {
            if response.status().is_success() {
                return true;
            }
        }

        if attempt < retries {
            std::thread::sleep(Duration::from_secs(timeout_sec));
        }
    }
    false
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
        assert!(result_mock);
        mock.assert();

        let result_baidu = check_network_connection(Some("https://www.baidu.com"), None);
        assert!(result_baidu);
    }

    #[test]
    fn test_network_connection_failure() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/").with_status(400).create();

        let result = check_network_connection(Some(&server.url()), None);

        assert!(!result);
    }

    #[test]
    fn test_network_connection_timeout() {
        let result = check_network_connection(Some("http://192.0.2.1:9999"), Some(1));

        assert!(!result);
    }
}
