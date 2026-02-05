# 多阶段构建 - 阶段 1: 编译
FROM rust:1.90-alpine AS builder

# 安装编译依赖
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# 设置工作目录
WORKDIR /build

# 复制项目源代码
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 编译项目
RUN cargo build --release

# 验证可执行文件已生成
RUN ls -lh /build/target/release/shu-net-keeper

# 多阶段构建 - 阶段 2: 运行时最小化镜像
FROM alpine:latest

# 安装运行时依赖（仅 CA 证书，用于 HTTPS 请求）
RUN apk add --no-cache ca-certificates tzdata && \
    cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
    echo "Asia/Shanghai" > /etc/timezone && \
    apk del tzdata

# 创建非 root 用户
RUN addgroup -g 1000 shunet && \
    adduser -D -u 1000 -G shunet shunet

# 设置工作目录
WORKDIR /app

# 从构建阶段复制可执行文件
COPY --from=builder /build/target/release/shu-net-keeper /app/shu-net-keeper

# 创建日志目录
RUN mkdir -p /app/logs && \
    chown -R shunet:shunet /app

# 切换到非 root 用户
USER shunet

# 声明挂载点
VOLUME ["/app/logs"]

# 健康检查（每 60 秒检查一次进程是否存在）
HEALTHCHECK --interval=60s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep -f shu-net-keeper || exit 1

# 启动程序
CMD ["/app/shu-net-keeper"]
