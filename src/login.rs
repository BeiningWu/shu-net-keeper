use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::time::Duration;

const LOGIN_URL: &str = "http://10.10.9.9/eportal/InterFace.do?method=login";

pub fn network_login(username: &str, password: &str) -> Result<(), String> {
    let query_string = get_login_query_string()?;
    // 创建请求头
    let mut headers = create_browser_headers()?;

    // Referer
    headers.insert(
        reqwest::header::REFERER,
        format!("http://10.10.9.9/eportal/index.jsp?{}", &query_string)
            .parse()
            .unwrap(),
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 构建表单数据
    let mut form_data = HashMap::new();
    form_data.insert("userId", username);
    form_data.insert("password", password);
    form_data.insert("service", "shu");
    form_data.insert("passwordEncrypt", "false");
    form_data.insert("operatorPwd", "");
    form_data.insert("operatorUserId", "");
    form_data.insert("validcode", "");
    form_data.insert("queryString", &query_string);

    let response = client
        .post(LOGIN_URL)
        .form(&form_data)
        .send()
        .map_err(|e| {
            println!("登录请求失败: {}", e);
            format!("登录请求失败: {}", e)
        })?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|e| format!("读取响应内容失败: {}", e))?;
    if status.is_success() {
        println!("✓ 登录成功");
        Ok(())
    } else {
        println!("✗ 登录失败，状态码: {}", status);
        Err(format!("登录失败，状态码: {}, 响应: {}", status, body))
    }
}

fn create_browser_headers() -> Result<HeaderMap, String> {
    let mut headers = reqwest::header::HeaderMap::new();

    // User-Agent
    headers.insert(
        reqwest::header::USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36"
            .parse()
            .unwrap(),
    );

    // Accept
    headers.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());

    // Accept-Encoding
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        "gzip, deflate".parse().unwrap(),
    );

    // Accept-Language
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
    );

    // Host
    headers.insert(reqwest::header::HOST, "10.10.9.9".parse().unwrap());

    Ok(headers)
}

fn get_login_query_string() -> Result<String, String> {
    let client = Client::new();

    // 1. 获取重定向到登录页的 URL
    let response = client
        .get("http://123.123.123.123/")
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;

    let html = response
        .text()
        .map_err(|e| format!("重定向登录页url读取失败: {}", e))?;

    // 2. 解析出 URL
    let url = extract_url_from_script(&html)?;

    // 3. 提取 queryString
    let query_string = extract_query_string(&url)?;

    // 4. URL 编码
    let encoded = urlencoding::encode(&query_string);

    Ok(encoded.to_string())
}

fn extract_url_from_script(html: &str) -> Result<String, String> {
    // 方法 1: 使用正则表达式
    use regex::Regex;

    let re =
        Regex::new(r"location\.href='([^']+)'").map_err(|e| format!("正则表达式错误: {}", e))?;

    if let Some(caps) = re.captures(html) {
        if let Some(url) = caps.get(1) {
            return Ok(url.as_str().to_string());
        }
    }

    Err("未找到登录页 URL".to_string())
}

/// 从 URL 中提取 query string（? 后面的部分）
fn extract_query_string(url: &str) -> Result<String, String> {
    url.split('?')
        .nth(1) // 获取 ? 后面的部分
        .map(|s| s.to_string())
        .ok_or("URL 中没有查询参数".to_string())
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_get_login_url() {
        let client = Client::new();

        let response = client
            .get("http://123.123.123.123/")
            .send()
            .map_err(|e| format!("请求失败: {}", e))
            .unwrap();

        let html = response
            .text()
            .map_err(|e| format!("重定向登录页url读取失败: {}", e))
            .unwrap();

        let url = extract_url_from_script(&html).unwrap();
        let query_string = extract_query_string(&url).unwrap();
        println!("{}", query_string);
    }

    #[test]
    fn test_login() {
        let username = "12345678";
        let password = "password";
        let result = network_login(username, password);
        match result {
            Ok(()) => print!("登陆成功"),
            Err(_) => print!("登陆失败"),
        }
    }
}
