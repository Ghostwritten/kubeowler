# Node inspector image: runs node-check-universal.sh with read-only host mounts.
# No runtime socket or crictl: container state counts are filled by kubeowler via Kubernetes API (runtime-agnostic).
FROM alpine:3.18
LABEL org.opencontainers.image.title="kubeowler-node-inspector" \
      org.opencontainers.image.description="Node inspection script for Kubeowler DaemonSet"
RUN apk add --no-cache bash coreutils openssl
COPY scripts/node-check-universal.sh /node-check-universal.sh
RUN chmod +x /node-check-universal.sh
# NODE_NAME injected via env from DaemonSet (fieldRef spec.nodeName)
# Keep container running after JSON output so DaemonSet (restartPolicy: Always) does not restart repeatedly
ENTRYPOINT ["sh", "-c", "/node-check-universal.sh; exec sleep infinity"]
