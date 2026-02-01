# 快速开始指南

5 分钟上手 SHU Net Keeper！

## 步骤 1: 获取程序

### 选项 A: 下载预编译版本（推荐）

前往 [Releases](https://github.com/yourusername/shu-net-keeper/releases) 页面下载对应系统的可执行文件。

### 选项 B: 从源码编译

```bash
git clone https://github.com/yourusername/shu-net-keeper.git
cd shu-net-keeper
cargo build --release
```

## 步骤 2: 配置

1. 复制配置文件示例：

   ```bash
   cp config.toml.example config.toml
   ```

2. 编辑 `config.toml`，填入你的学号和密码：

   ```toml
   username = "你的学号"
   password = "你的密码"
   interval = 30
   ```

## 步骤 3: 运行

### Windows

双击 `shu-net-keeper.exe` 或在命令行运行：

```cmd
shu-net-keeper.exe
```

### Linux / macOS

```bash
chmod +x shu-net-keeper
./shu-net-keeper
```

### Docker

```bash
docker-compose up -d
```

## 步骤 4: 验证

程序启动后会自动：
1. 检查网络连接状态
2. 如果未连接，尝试登录
3. 每 30 秒重复检查一次

查看日志文件确认运行状态：
- 日志位置：`logs/shu-net-keeper.log`
- 看到 `✓ 登录成功` 表示工作正常

## 步骤 5: 设置开机自启（可选）

### Windows

使用 NSSM（推荐）：
```powershell
nssm install SHUNetKeeper "C:\path\to\shu-net-keeper.exe"
nssm start SHUNetKeeper
```

### Linux

```bash
sudo cp shu-net-keeper.service /etc/systemd/system/
sudo systemctl enable shu-net-keeper
sudo systemctl start shu-net-keeper
```

### macOS

```bash
sudo cp com.shu.netkeeper.plist /Library/LaunchDaemons/
sudo launchctl load /Library/LaunchDaemons/com.shu.netkeeper.plist
```

详细部署方法请参考 [README.md](README.md#部署方式)。

## 常见问题

**Q: 如何启用邮件通知？**
A: 在 `config.toml` 中添加 `[smtp]` 配置块，参考 `config.toml.example` 中的示例。

**Q: 程序运行后没反应？**
A: 查看 `logs/shu-net-keeper.log` 文件中的错误信息。

**Q: Docker 部署后无法登录？**
A: 确保使用了 `--network host` 模式，容器需要访问宿主机的校园网环境。

**Q: 如何修改检查间隔？**
A: 在 `config.toml` 中修改 `interval` 值（单位：秒）。

更多问题请查阅 [README.md](README.md#故障排查) 或提交 [Issue](https://github.com/yourusername/shu-net-keeper/issues)。
