# 多阶段构建 Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# 构建项目
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装必要的运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/kubeowler /usr/local/bin/kubeowler

# 设置执行权限
RUN chmod +x /usr/local/bin/kubeowler

# 创建非root用户
RUN useradd -r -s /bin/false kubeowler

USER kubeowler

ENTRYPOINT ["kubeowler"]
CMD ["check"]