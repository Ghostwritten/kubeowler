# Multi-stage: fetch pre-built binary from GitHub Release (no Rust compile).
# Build with: docker build --build-arg TAR_URL=<url> --build-arg VERSION=v0.1.1 -t ghostwritten/kubeowler:v0.1.1 .

ARG TAR_URL
FROM debian:bookworm-slim AS getter
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
ARG TAR_URL
RUN mkdir -p /out && curl -sL "$TAR_URL" | tar xz -C /out

FROM debian:bookworm-slim
ARG VERSION
LABEL org.opencontainers.image.version="${VERSION}"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=getter /out/kubeowler /usr/local/bin/kubeowler
RUN chmod +x /usr/local/bin/kubeowler

RUN useradd -r -s /bin/false kubeowler
USER kubeowler

ENTRYPOINT ["kubeowler"]
CMD ["check"]
