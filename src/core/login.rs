use crate::constants::{CAMPUS_GATEWAY, LOGIN_INDEX, LOGIN_URL, USER_AGENT};
use crate::error::{LoginError, LoginResult};
use crate::rsa::PasswordEncryptor;
use tracing::{debug, error, info, warn};

pub fn network_login(username: &str, password: &str) -> LoginResult<()> {
    info!("开始网络登录，用户: {}", username);

    // 创建一个共享的 agent，确保 cookie 在整个登录流程中保持一致
    let agent = ureq::agent();

    debug!("获取登录查询字符串...");
    let query_string = get_login_query_string_with_agent(&agent)?;
    debug!("查询字符串获取成功");

    // 从 queryString 中提取 mac 字段
    let mac = extract_mac_from_query_string(&query_string)?;
    debug!("提取到的 MAC: {}", mac);

    // 拼接密码: password + ">" + mac
    let password_with_mac = format!("{}>{}", password, mac);
    debug!("拼接后的密码字符串长度: {}", password_with_mac.len());

    // 使用 RSA 加密密码
    let encryptor = PasswordEncryptor::new().map_err(|e| {
        error!("创建密码加密器失败: {}", e);
        LoginError::RequestFailed(e.to_string())
    })?;
    let encrypted_password = encryptor
        .encrypt_password(&password_with_mac)
        .map_err(|e| {
            error!("密码加密失败: {}", e);
            LoginError::RequestFailed(e.to_string())
        })?;
    debug!("密码加密成功");

    // 服务器期望 queryString 是预编码的，send_form 会再次编码（双重编码）
    let encoded_query_string = urlencoding::encode(&query_string).to_string();

    let referer = format!("{}?{}", LOGIN_INDEX, &query_string);
    debug!("Referer: {}", referer);

    // 构建表单数据
    let form_data: &[(&str, &str)] = &[
        ("userId", username),
        ("password", &encrypted_password),
        ("service", "shu"),
        ("passwordEncrypt", "true"),
        ("operatorPwd", ""),
        ("operatorUserId", ""),
        ("validcode", ""),
        ("queryString", &encoded_query_string),
    ];

    debug!("发送登录请求到 {}...", LOGIN_URL);
    let response = agent
        .post(LOGIN_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "*/*")
        .set("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .set("Host", "10.10.9.9")
        .set("Referer", &referer)
        .send_form(form_data)
        .map_err(|e| {
            error!("登录请求失败: {}", e);
            LoginError::RequestFailed(e.to_string())
        })?;

    let status = response.status();
    debug!("收到响应，状态码: {}", status);

    let body = response.into_string().map_err(|e| {
        error!("读取响应内容失败: {}", e);
        LoginError::ResponseParseFailed(e.to_string())
    })?;

    if status >= 200 && status < 300 {
        info!("✓ 登录成功");
        Ok(())
    } else {
        error!("✗ 登录失败，状态码: {}, 响应: {}", status, body);
        Err(LoginError::AuthenticationFailed {
            status,
            message: body,
        })
    }
}

fn get_login_query_string_with_agent(agent: &ureq::Agent) -> LoginResult<String> {
    debug!("开始获取登录查询字符串...");

    // 1. 访问校园网关，让客户端自动跟随重定向链
    debug!("访问校园网关 {}，跟随重定向...", CAMPUS_GATEWAY);
    let response = agent.get(CAMPUS_GATEWAY).call().map_err(|e| {
        error!("访问校园网关失败: {}", e);
        LoginError::QueryStringFailed(e.to_string())
    })?;

    // 2. 获取最终URL（跟随所有重定向后的URL）
    let final_url = response.get_url().to_string();
    debug!("最终 URL: {}", final_url);

    // 3. 读取HTML内容
    debug!("读取 HTML 响应...");
    let html = response.into_string().map_err(|e| {
        error!("读取响应失败: {}", e);
        LoginError::QueryStringFailed(e.to_string())
    })?;

    // 4. 从HTML中提取JavaScript重定向的URL
    debug!("从 HTML 中提取登录页 URL...");
    let login_url = extract_url_from_script(&html)?;
    debug!("提取到的登录 URL: {}", login_url);

    // 5. 提取 queryString（不要预先编码，send_form 会自动处理）
    let query_string = extract_query_string(&login_url)?;
    debug!("提取到的查询字符串长度: {}", query_string.len());

    Ok(query_string)
}

/// 从HTML脚本中提取重定向URL
fn extract_url_from_script(html: &str) -> LoginResult<String> {
    use regex::Regex;

    let re = Regex::new(r"location\.href='([^']+)'").map_err(|e| {
        error!("正则表达式创建失败: {}", e);
        LoginError::UrlParseFailed(e.to_string())
    })?;

    if let Some(caps) = re.captures(html) {
        if let Some(url) = caps.get(1) {
            debug!("成功从 HTML 中提取登录页 URL");
            return Ok(url.as_str().to_string());
        }
    }

    // 如果没有找到JavaScript重定向，检查是否已经在登录成功页面
    if html.contains("success") || html.contains("成功") {
        warn!("HTML 中包含成功标识，可能已经登录");
        return Err(LoginError::QueryStringFailed(
            "页面显示已登录或成功".to_string(),
        ));
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

/// 从 queryString 中提取 mac 字段
fn extract_mac_from_query_string(query_string: &str) -> LoginResult<String> {
    for pair in query_string.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "mac" {
                return Ok(value.to_string());
            }
        }
    }
    Err(LoginError::UrlParseFailed(
        "queryString 中没有 mac 字段".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_login_url() {
        // 创建 agent（会自动跟随重定向）
        let agent = ureq::agent();

        // 访问校园网关，让客户端自动跟随重定向
        let response = agent
            .get(CAMPUS_GATEWAY)
            .call()
            .map_err(|e| format!("请求失败: {}", e))
            .unwrap();

        let final_url = response.get_url().to_string();
        println!("最终 URL: {}", final_url);
        println!("状态码: {:?}", response.status());

        // 读取HTML内容
        let html = response.into_string().unwrap();
        println!("HTML 长度: {} 字节", html.len());

        // 尝试从HTML中提取登录URL
        match extract_url_from_script(&html) {
            Ok(login_url) => {
                println!("提取到的登录 URL: {}", login_url);
                if let Ok(query_string) = extract_query_string(&login_url) {
                    println!("查询字符串: {}", query_string);
                }
            }
            Err(e) => {
                println!("提取登录 URL 失败: {}", e);
                println!("HTML 内容片段: {}", &html[..html.len().min(500)]);
            }
        }
    }

    #[test]
    #[ignore] // 需要在校园网环境下手动运行，并设置环境变量
    fn test_login() {
        let username = std::env::var("SHU_USERNAME").expect("请设置 SHU_USERNAME 环境变量");
        let password = std::env::var("SHU_PASSWORD").expect("请设置 SHU_PASSWORD 环境变量");
        let result = network_login(&username, &password);
        match result {
            Ok(()) => println!("登录成功"),
            Err(e) => println!("登录失败: {:?}", e),
        }
    }
}
