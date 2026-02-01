use reqwest::blocking::Client;

pub fn get_login_page() -> Result<String, String> {
    let client = Client::new();

    // 1. 先访问任意网页，获取重定向到登录页的 URL
    let response = client
        .get("http://www.baidu.com")
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;

    // 2. 从重定向的 URL 中提取 queryString
    let final_url = response.url().as_str();

    if final_url.contains("10.10.9.9") {
        // 提取查询参数
        let query_string = final_url.split('?').nth(1).ok_or("未找到查询参数")?;

        // URL 编码
        Ok(urlencoding::encode(query_string).to_string())
    } else {
        Err("未被重定向到登录页".to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn get_login_query_string() {
        let result = get_login_page();

        match result {
            Ok(query_string) => {
                println!("✓ 获取到 queryString:");
                println!("{}", query_string);

                // 验证包含必要的参数
                assert!(query_string.contains("wlanuserip") || query_string.contains("%"));
            }
            Err(e) => {
                println!("✗ 获取失败: {}", e);
            }
        }
    }
}
