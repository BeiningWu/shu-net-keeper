use crate::constants::{LOGIN_INDEX, LOGIN_URL, REDIRECT_URL};
use crate::error::{LoginError, LoginResult};
use crate::http_client::HttpClientFactory;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

pub fn network_login(username: &str, password: &str) -> LoginResult<()> {
    info!("开始网络登录，用户: {}", username);

    debug!("获取登录查询字符串...");
    let query_string = get_login_query_string()?;
    debug!("查询字符串获取成功");

    // 创建登录客户端，使用工厂方法
    let referer = format!("{}?{}", LOGIN_INDEX, &query_string);
    debug!("创建 HTTP 客户端，Referer: {}", referer);

    let client = HttpClientFactory::new_login_client(&referer).map_err(|e| {
        error!("创建 HTTP 客户端失败: {}", e);
        LoginError::ClientCreationFailed(e.to_string())
    })?;

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

    debug!("发送登录请求到 {}...", LOGIN_URL);
    let response = client.post(LOGIN_URL).form(&form_data).send().map_err(|e| {
        error!("登录请求失败: {}", e);
        LoginError::RequestFailed(e.to_string())
    })?;

    let status = response.status();
    debug!("收到响应，状态码: {}", status);

    let body = response.text().map_err(|e| {
        error!("读取响应内容失败: {}", e);
        LoginError::ResponseParseFailed(e.to_string())
    })?;

    if status.is_success() {
        info!("✓ 登录成功");
        Ok(())
    } else {
        error!("✗ 登录失败，状态码: {}, 响应: {}", status, body);
        Err(LoginError::AuthenticationFailed {
            status: status.as_u16(),
            message: body,
        })
    }
}

fn get_login_query_string() -> LoginResult<String> {
    debug!("开始获取登录查询字符串...");

    // 使用工厂创建默认客户端
    let client = HttpClientFactory::new_default().map_err(|e| {
        error!("创建 HTTP 客户端失败: {}", e);
        LoginError::QueryStringFailed(e.to_string())
    })?;

    // 1. 获取重定向到登录页的 URL
    debug!("请求重定向页面...");
    let response = client.get(REDIRECT_URL).send().map_err(|e| {
        error!("请求重定向页面失败: {}", e);
        LoginError::QueryStringFailed(e.to_string())
    })?;

    debug!("读取 HTML 响应...");
    let html = response.text().map_err(|e| {
        error!("读取响应失败: {}", e);
        LoginError::QueryStringFailed(e.to_string())
    })?;

    // 2. 解析出 URL
    debug!("从脚本中提取 URL...");
    let url = extract_url_from_script(&html)?;
    debug!("提取到的 URL: {}", url);

    // 3. 提取 queryString
    let query_string = extract_query_string(&url)?;
    debug!("提取到的查询字符串长度: {}", query_string.len());

    // 4. URL 编码
    let encoded = urlencoding::encode(&query_string);
    debug!("查询字符串编码完成");

    Ok(encoded.to_string())
}

fn extract_url_from_script(html: &str) -> LoginResult<String> {
    // 方法 1: 使用正则表达式
    use regex::Regex;

    let re = Regex::new(r"location\.href='([^']+)'").map_err(|e| {
        error!("正则表达式创建失败: {}", e);
        LoginError::UrlParseFailed(e.to_string())
    })?;

    if let Some(caps) = re.captures(html) {
        if let Some(url) = caps.get(1) {
            debug!("成功提取登录页 URL");
            return Ok(url.as_str().to_string());
        }
    }

    warn!("未在 HTML 中找到登录页 URL");
    Err(LoginError::UrlParseFailed("未找到登录页 URL".to_string()))
}

/// 从 URL 中提取 query string（? 后面的部分）
fn extract_query_string(url: &str) -> LoginResult<String> {
    url.split('?')
        .nth(1) // 获取 ? 后面的部分
        .map(|s| s.to_string())
        .ok_or_else(|| LoginError::UrlParseFailed("URL 中没有查询参数".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_login_url() {
        let client = HttpClientFactory::new_default().unwrap();

        let response = client
            .get(REDIRECT_URL)
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
