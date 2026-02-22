// 网络端点常量
pub const CAMPUS_GATEWAY: &str = "http://10.10.9.9";
pub const LOGIN_URL: &str = "http://10.10.9.9/eportal/InterFace.do?method=login";
pub const LOGIN_INDEX: &str = "http://10.10.9.9/eportal/index.jsp";
pub const ONLINE_INFO_URL: &str = "http://10.10.9.9/eportal/InterFace.do?method=getOnlineUserInfo";

// User-Agent
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36";

// 配置默认值
pub const DEFAULT_CHECK_INTERVAL: u64 = 600;
pub const REQUIRED_USERNAME_LENGTH: usize = 8;
