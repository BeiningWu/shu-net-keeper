# SHU Net Keeper

<div align="center">

**上海大学校园网自动登录助手**

一个基于 Rust 开发的轻量级网络守护程序，自动检测并恢复校园网连接

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

## 功能特性

- 🔄 **自动登录** - 定期检查网络状态，断网时自动重新登录
- 📧 **邮件通知** - 支持 SMTP 邮件通知，登录成功后发送提醒
- 🌐 **IP 监控** - 检测 IP 地址变化并及时通知
- ⚙️ **灵活配置** - 通过 TOML 配置文件自定义登录参数和检查间隔
- 📝 **详细日志** - 完整的日志记录系统，支持文件输出和按日期归档
- 🚀 **低资源占用** - Rust 编写，内存占用少，性能高效
- 🐳 **容器化部署** - 提供 Docker 支持，跨平台部署

## 快速开始

### 配置文件

在可执行文件同目录下创建 `config.toml`：

```toml
# 必填：校园网账号信息
username = "your_student_id"
password = "your_password"

# 可选：检查间隔（秒），默认 30 秒
interval = 30

# 可选：SMTP 邮件通知配置
[smtp]
server = "smtp.example.com"    # SMTP 服务器地址
port = 587                      # SMTP 端口（常用：587/465/25）
username = "your_email@example.com"
password = "your_email_password"
from = "your_email@example.com"
to = "recipient@example.com"
use_tls = true                  # 是否使用 TLS（587端口推荐 true，465端口推荐 false）
```

### 配置示例

**基础配置（仅自动登录）**：
```toml
username = "20221234567"
password = "mypassword123"
interval = 60
```

**完整配置（包含邮件通知）**：
```toml
username = "20221234567"
password = "mypassword123"
interval = 30

[smtp]
server = "smtp.qq.com"
port = 587
username = "123456789@qq.com"
password = "授权码"
from = "123456789@qq.com"
to = "notify@example.com"
use_tls = true
```

## 部署方式

### Windows 系统

#### 方式一：Windows 服务（推荐）

1. **使用 NSSM 创建服务**

   下载 NSSM：https://nssm.cc/download

   ```powershell
   # 以管理员身份运行 PowerShell
   nssm install SHUNetKeeper "C:\path\to\shu-net-keeper.exe"
   nssm set SHUNetKeeper AppDirectory "C:\path\to\"
   nssm start SHUNetKeeper
   ```

2. **管理服务**

   ```powershell
   # 查看服务状态
   nssm status SHUNetKeeper

   # 停止服务
   nssm stop SHUNetKeeper

   # 重启服务
   nssm restart SHUNetKeeper

   # 卸载服务
   nssm remove SHUNetKeeper confirm
   ```

#### 方式二：任务计划程序

1. 打开"任务计划程序"
2. 创建基本任务 → 选择"当计算机启动时"触发
3. 操作选择"启动程序"，浏览到 `shu-net-keeper.exe`
4. 勾选"使用最高权限运行"

### macOS 系统

使用 launchd 创建系统守护进程：

1. **创建 plist 文件**

   ```bash
   sudo nano /Library/LaunchDaemons/com.shu.netkeeper.plist
   ```

2. **填入以下内容**

   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>com.shu.netkeeper</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/local/bin/shu-net-keeper</string>
       </array>
       <key>WorkingDirectory</key>
       <string>/usr/local/bin</string>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
       <key>StandardOutPath</key>
       <string>/var/log/shu-net-keeper.log</string>
       <key>StandardErrorPath</key>
       <string>/var/log/shu-net-keeper.error.log</string>
   </dict>
   </plist>
   ```

3. **部署和管理**

   ```bash
   # 复制可执行文件
   sudo cp shu-net-keeper /usr/local/bin/
   sudo cp config.toml /usr/local/bin/
   sudo chmod +x /usr/local/bin/shu-net-keeper

   # 加载服务
   sudo launchctl load /Library/LaunchDaemons/com.shu.netkeeper.plist

   # 启动服务
   sudo launchctl start com.shu.netkeeper

   # 停止服务
   sudo launchctl stop com.shu.netkeeper

   # 卸载服务
   sudo launchctl unload /Library/LaunchDaemons/com.shu.netkeeper.plist
   ```

### Linux 系统

使用 systemd 创建系统服务：

1. **创建 systemd 服务文件**

   ```bash
   sudo nano /etc/systemd/system/shu-net-keeper.service
   ```

2. **填入以下内容**

   ```ini
   [Unit]
   Description=SHU Network Auto Login Daemon
   After=network.target
   Wants=network-online.target

   [Service]
   Type=simple
   User=root
   WorkingDirectory=/opt/shu-net-keeper
   ExecStart=/opt/shu-net-keeper/shu-net-keeper
   Restart=on-failure
   RestartSec=10

   # 日志配置
   StandardOutput=journal
   StandardError=journal

   [Install]
   WantedBy=multi-user.target
   ```

3. **部署和管理**

   ```bash
   # 创建目录并复制文件
   sudo mkdir -p /opt/shu-net-keeper
   sudo cp shu-net-keeper /opt/shu-net-keeper/
   sudo cp config.toml /opt/shu-net-keeper/
   sudo chmod +x /opt/shu-net-keeper/shu-net-keeper

   # 重新加载 systemd 配置
   sudo systemctl daemon-reload

   # 启用开机自启
   sudo systemctl enable shu-net-keeper

   # 启动服务
   sudo systemctl start shu-net-keeper

   # 查看服务状态
   sudo systemctl status shu-net-keeper

   # 查看日志
   sudo journalctl -u shu-net-keeper -f

   # 停止服务
   sudo systemctl stop shu-net-keeper

   # 重启服务
   sudo systemctl restart shu-net-keeper
   ```

### Docker 部署

#### 方式一：使用 Docker Compose（推荐）

1. **创建 docker-compose.yml**

   ```yaml
   version: '3.8'

   services:
     shu-net-keeper:
       build: .
       container_name: shu-net-keeper
       restart: unless-stopped
       network_mode: host  # 使用宿主机网络，确保能访问校园网
       volumes:
         - ./config.toml:/app/config.toml:ro
         - ./logs:/app/logs
       environment:
         - TZ=Asia/Shanghai
   ```

2. **启动服务**

   ```bash
   docker-compose up -d

   # 查看日志
   docker-compose logs -f

   # 停止服务
   docker-compose down
   ```

#### 方式二：使用 Docker 命令

```bash
# 构建镜像
docker build -t shu-net-keeper .

# 运行容器
docker run -d \
  --name shu-net-keeper \
  --network host \
  --restart unless-stopped \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/logs:/app/logs \
  -e TZ=Asia/Shanghai \
  shu-net-keeper

# 查看日志
docker logs -f shu-net-keeper

# 停止容器
docker stop shu-net-keeper

# 重启容器
docker restart shu-net-keeper
```

## 从源码构建

### 前置要求

- Rust 1.70 或更高版本
- Cargo（Rust 包管理器）

### 编译步骤

```bash
# 克隆仓库
git clone https://github.com/yourusername/shu-net-keeper.git
cd shu-net-keeper

# 编译 Release 版本
cargo build --release

# 可执行文件位于
# Linux/macOS: ./target/release/shu-net-keeper
# Windows: .\target\release\shu-net-keeper.exe
```

### 交叉编译

```bash
# 安装交叉编译工具
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-musl

# 编译 Windows 版本
cargo build --release --target x86_64-pc-windows-gnu

# 编译 macOS 版本
cargo build --release --target x86_64-apple-darwin

# 编译 Linux 版本（静态链接）
cargo build --release --target x86_64-unknown-linux-musl
```

## 日志说明

程序会在可执行文件同目录下生成 `logs/` 目录：

```
logs/
├── shu-net-keeper.log          # 当前日志文件
├── shu-net-keeper.2024-01-15.log  # 按日期归档的历史日志
└── shu-net-keeper.2024-01-14.log
```

日志级别：
- `INFO` - 正常运行信息
- `WARN` - 警告信息（如网络断开）
- `ERROR` - 错误信息（如登录失败）
- `DEBUG` - 调试信息（仅在调试模式下显示）

## 故障排查

### 问题：程序无法启动

- 检查 `config.toml` 是否存在且格式正确
- 查看日志文件中的错误信息
- 确认账号密码是否正确

### 问题：自动登录失败

- 检查网络连接是否正常
- 确认是否在校园网环境内
- 查看日志中的具体错误信息
- 手动访问 `http://10.10.9.9` 测试登录页面是否可访问

### 问题：邮件通知发送失败

- 确认 SMTP 服务器地址和端口正确
- 检查邮箱密码（部分邮箱需要使用授权码，而非登录密码）
- 确认邮箱已开启 SMTP 服务
- 检查 `use_tls` 设置是否与端口匹配

### 问题：Docker 容器无法连接网络

- 确保使用 `--network host` 模式
- 检查宿主机是否在校园网环境
- 查看容器日志：`docker logs shu-net-keeper`

## 常见邮箱 SMTP 配置

| 邮箱服务商 | SMTP 服务器 | 端口 | use_tls | 备注 |
|-----------|------------|------|---------|------|
| QQ 邮箱 | smtp.qq.com | 587 | true | 需要使用授权码 |
| 163 邮箱 | smtp.163.com | 465 | false | 需要使用授权码 |
| Gmail | smtp.gmail.com | 587 | true | 需要开启两步验证并使用应用专用密码 |
| Outlook | smtp.office365.com | 587 | true | 直接使用邮箱密码 |

## 安全建议

1. **配置文件权限**：确保 `config.toml` 仅当前用户可读
   ```bash
   chmod 600 config.toml  # Linux/macOS
   ```

2. **密码保护**：避免在配置文件中使用明文密码（项目计划支持环境变量）

3. **定期更新**：保持程序更新到最新版本

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 免责声明

本工具仅供学习交流使用，请遵守学校网络使用规定。使用本工具产生的任何问题由使用者自行承担。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解版本更新历史
