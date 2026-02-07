# Node Inspector: Build and Deploy

This document describes how to build, push, and deploy the **kubeowler-node-inspector** DaemonSet in a Kubernetes cluster. The DaemonSet runs a per-node script that outputs a single JSON object; Kubeowler collects this output and includes it in the inspection report.

---

## 1. Overview

### 1.1 Components

| Component | Description |
|-----------|-------------|
| **node-check-universal.sh** | Per-node script: reads `/proc`, `/sys`, `/etc`, etc., gathers resource, service, security, and kernel data; outputs one line of JSON to stdout. **Does not depend on any runtime socket**; container state counts are filled by Kubeowler via the Kubernetes API per node. |
| **Dockerfile** | Alpine-based image with bash/coreutils; script is the ENTRYPOINT. Must be built from the **project root** so `COPY scripts/` works. |
| **DaemonSet** | Deployed in `kube-system`, one Pod per node; read-only hostPath mounts for `/proc`, `/sys`, `/etc`; `NODE_NAME` injected via Downward API; container runs the script once (JSON to stdout), then `sleep infinity` so the Pod stays Running and the JSON remains in **Pod logs** for Kubeowler to fetch. |
| **Kubeowler** | Lists Pods in `kube-system` with label `app=kubeowler-node-inspector`, fetches each Pod's log, parses JSON; merges into node inspection results and writes the "Node Inspection" section of the report. |

### 1.2 Directory layout

```
kubeowler/
├── deploy/node-inspector/
│   ├── Dockerfile
│   └── daemonset.yaml
├── scripts/
│   └── node-check-universal.sh
└── docs/
    ├── node-inspection-schema.md
    └── node-inspector-build-deploy.md  (this document)
```

---

## 2. Prerequisites and Supported Environment

- **Docker** or **Podman** for building and pushing the image.
- **kubectl** configured with access to the target cluster.
- Kubernetes 1.23 or later; node architecture amd64 or arm64.

---

## 3. Building the Image

**Build from the project root.** The Dockerfile uses `COPY scripts/node-check-universal.sh`.

```bash
cd /path/to/kubeowler
docker build -f deploy/node-inspector/Dockerfile \
  -t docker.io/ghostwritten/kubeowler-node-inspector:v0.1.1 .
```

For multi-arch (linux/amd64 and linux/arm64), use Docker Buildx or run `./deploy/node-inspector/build-push-multiarch.sh v0.1.1`.

---

## 4. Pushing and Deploying

Push to your registry, then:

```bash
kubectl apply -f deploy/node-inspector/daemonset.yaml
kubectl get pods -n kube-system -l app=kubeowler-node-inspector
kubectl logs -n kube-system -l app=kubeowler-node-inspector --tail=1 -c inspector
```

---

## 5. References

- **Node inspection JSON schema:** [node-inspection-schema.md](node-inspection-schema.md)
- **Collection vs. report usage:** [node-inspector-collection-gaps.md](node-inspector-collection-gaps.md)
- **Limitations and host visibility:** [node-inspector-limitations.md](node-inspector-limitations.md)
