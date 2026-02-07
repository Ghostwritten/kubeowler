# Multi-stage Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# Build project
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy built binary
COPY --from=builder /app/target/release/kubeowler /usr/local/bin/kubeowler

# Set executable permission
RUN chmod +x /usr/local/bin/kubeowler

# Create non-root user
RUN useradd -r -s /bin/false kubeowler

USER kubeowler

ENTRYPOINT ["kubeowler"]
CMD ["check"]
