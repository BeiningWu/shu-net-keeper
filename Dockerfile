# 多阶段构建 - 阶段 1: 编译
FROM rust:1.90-alpine AS builder

# Docker 自动注入目标平台架构（amd64 / arm64）
ARG TARGETARCH

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    linux-headers \
    gcc \
    make

# 设置 OpenSSL 环境变量（静态链接）
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
ENV OPENSSL_STATIC=1

# 设置工作目录
WORKDIR /build

# 复制项目源代码
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 根据目标架构选择 Rust target，编译后统一复制到固定路径
RUN case "$TARGETARCH" in \
      "amd64") RUST_TARGET="x86_64-unknown-linux-musl" ;; \
      "arm64") RUST_TARGET="aarch64-unknown-linux-musl" ;; \
      *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    rustup target add "$RUST_TARGET" && \
    cargo build --release --target "$RUST_TARGET" && \
    cp "/build/target/$RUST_TARGET/release/shu-net-keeper" /build/shu-net-keeper

# 多阶段构建 - 阶段 2: 运行时最小化镜像
FROM alpine:latest

# 安装运行时依赖（CA 证书用于 HTTPS，procps 用于健康检查）
RUN apk add --no-cache ca-certificates tzdata procps && \
    cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
    echo "Asia/Shanghai" > /etc/timezone && \
    apk del tzdata

# 创建非 root 用户
RUN addgroup -g 1000 shunet && \
    adduser -D -u 1000 -G shunet shunet

# 设置工作目录
WORKDIR /app

# 从构建阶段复制可执行文件
COPY --from=builder /build/shu-net-keeper /app/shu-net-keeper

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
