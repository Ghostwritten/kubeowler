# Node Inspector: Build and Deploy

This document describes how to build, push, and deploy the **kubeowler-node-inspector** DaemonSet in a Kubernetes cluster. The DaemonSet runs a per-node script that outputs a single JSON object; Kubeowler collects this output and includes it in the inspection report.

---

## 1. Overview

### 1.1 Components

| Component | Description |
|-----------|-------------|
| **node-check-universal.sh** | Per-node script: reads `/proc`, `/sys`, `/etc`, etc., gathers resource, service, security, and kernel data; detects container runtime (containerd/docker/cri-o) and NTP sync (timedatectl/chronyc); outputs one line of JSON to stdout. **Does not depend on any runtime socket**; container state counts are filled by Kubeowler via the Kubernetes API per node. |
| **Dockerfile** | Alpine-based image with bash/coreutils; script is the ENTRYPOINT. Must be built from the **project root** so `COPY scripts/` works. |
| **DaemonSet** | Deployed in `kube-system`, one Pod per node; read-only hostPath mounts for `/proc`, `/sys`, `/etc` (**no runtime sockets**); `NODE_NAME` injected via Downward API; container runs the script once (JSON to stdout), then `sleep infinity` so the Pod stays Running and the JSON remains in **Pod logs** for Kubeowler to fetch. The same DaemonSet works on Docker, containerd, and CRI-O (mixed or single). |
| **Kubeowler** | Lists Pods in `kube-system` with label `app=kubeowler-node-inspector`, fetches each Pod’s log, parses JSON; uses the Kubernetes API to list Pods per node and fill `container_state_counts` (running/exited/waiting); merges into node inspection results and writes the “Node Inspection” section of the report. |

### 1.2 Directory layout

```
kubeowler/
├── deploy/node-inspector/
│   ├── Dockerfile
│   └── daemonset.yaml
├── scripts/
│   └── node-check-universal.sh
└── docs/
    ├── 08-node-inspection-schema.md
    └── 06-node-inspector-build-deploy.md  (this document)
```

---

## 2. Prerequisites and Supported Environment

### 2.1 Prerequisites

- **Docker** or **Podman** for building and pushing the image.
- **kubectl** configured with access to the target cluster (deploy DaemonSet, list Pods, read logs).
- For Docker Hub: account and, recommended, a **Personal Access Token** with Write permission.

### 2.2 Kubernetes and node environment

| Item | Requirement |
|------|-------------|
| **Kubernetes** | 1.23 or later (1.24+ recommended) |
| **Node architecture** | amd64, arm64 |
| **Node OS** | RHEL 7+, CentOS 7.x, Rocky Linux 8+, AlmaLinux 8+, Ubuntu 20.04+, SUSE 12+, and other glibc-compatible Linux distributions |

The image is Alpine-based and runs on any Linux host that supports containers.

---

## 3. Building the Image

### 3.1 Build context

**Build from the project root.** The Dockerfile uses:

```dockerfile
COPY scripts/node-check-universal.sh /node-check-universal.sh
```

Building from `deploy/node-inspector/` would not find `scripts/`.

### 3.2 Podman

```bash
cd /path/to/kubeowler
podman build -f deploy/node-inspector/Dockerfile \
  -t docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0 .
```

Replace `ghostwritten` with your registry username.

### 3.3 Docker

```bash
cd /path/to/kubeowler
docker build -f deploy/node-inspector/Dockerfile \
  -t docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0 .
```

### 3.4 Multi-architecture (linux/amd64 and linux/arm64)

**Docker Buildx:**

```bash
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 \
  -f deploy/node-inspector/Dockerfile \
  -t docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0 \
  --push .
```

Replace the tag with your version (e.g. `v0.1.0` or `latest`). The same image name will serve both architectures.

**Optional script:** From the repo root (default tag is v0.1.0):

```bash
./deploy/node-inspector/build-push-multiarch.sh v0.1.0
```

---

## 4. Pushing to Docker Hub

### 4.1 Login

Use a **Personal Access Token** to avoid “login succeeded but push denied”:

```bash
# Podman
podman logout docker.io
podman login docker.io -u ghostwritten -p <YOUR_ACCESS_TOKEN>

# Docker
docker logout
docker login -u ghostwritten -p <YOUR_ACCESS_TOKEN>
```

Create the token at [Docker Hub → Account Settings → Security → New Access Token](https://hub.docker.com/settings/security) with at least **Read, Write, Delete**.

### 4.2 Push

```bash
podman push docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0
# or
docker push docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0
```

---

## 5. Deploying the DaemonSet

### 5.1 Image and namespace

- **Image:** `deploy/node-inspector/daemonset.yaml` references `docker.io/ghostwritten/kubeowler-node-inspector:v0.1.0`. Bump the tag when the script or image changes (e.g. v0.2.0).
- **Namespace:** DaemonSet is in `kube-system`.

### 5.2 Apply

```bash
kubectl apply -f deploy/node-inspector/daemonset.yaml
```

### 5.3 Verify

```bash
kubectl get daemonset -n kube-system kubeowler-node-inspector
kubectl get pods -n kube-system -l app=kubeowler-node-inspector
kubectl logs -n kube-system -l app=kubeowler-node-inspector --tail=1 -c inspector
```

The container runs the script (JSON to stdout), then `sleep infinity`, so the Pod stays Running and the JSON is in the **Pod log**. Kubeowler fetches that log via the Pod Logs API. DaemonSet only supports `restartPolicy: Always`; without sleep the container would exit and restart (CrashLoopBackOff).

### 5.4 Update and rollback

Re-apply after changing image or YAML. To roll back, edit the `image` in `daemonset.yaml` and apply again.

---

## 6. DaemonSet Configuration

### 6.1 One-shot output and long-running container

The script runs once per node and prints JSON. Because DaemonSet Pods must use `restartPolicy: Always`, the entrypoint runs the script then `exec sleep infinity` so the process does not exit and the log is preserved. To re-collect on a node, delete that Pod; the DaemonSet will recreate it and run the script again.

### 6.2 Runtime-agnostic design

The script only uses read-only mounts (`/proc`, `/sys`, `/etc`); **no runtime sockets** are mounted. One DaemonSet works for Docker, containerd, and CRI-O. Container state counts (running/exited/waiting) are computed by Kubeowler from the Kubernetes API per node, not from the script.

### 6.3 Where the JSON lives and how Kubeowler collects it

| Step | Description |
|------|-------------|
| 1. Output | Script prints one JSON line to **container stdout**; Kubernetes stores that as **Pod logs**. |
| 2. Trigger | User runs `kubeowler check` (all or nodes). |
| 3. Collection | Code lists Pods in `kube-system` with label `app=kubeowler-node-inspector`, then calls the Pod log API per Pod to get the log body. |
| 4. Parse | Log is parsed as JSON into `NodeInspectionResult`; results are sorted by node and stored in `ClusterReport.node_inspection_results`. |
| 5. Report | Report generator renders the “Node Inspection” section (summary table, resources/services/security/kernel tables, node issues). |

JSON is not written to a PVC or host path; it exists only in Pod logs. Kubeowler fetches it via the Kubernetes client.

### 6.4 Resources and security

| Setting | Value | Note |
|---------|--------|------|
| resources.requests | cpu: 10m, memory: 32Mi | Scheduling and minimum resources. |
| resources.limits | cpu: 100m, memory: 64Mi | Limit per Pod. |
| securityContext | runAsUser: 0, allowPrivilegeEscalation: false, capabilities.drop: ALL | Root for hostPath reads; no privilege escalation or extra capabilities. |
| hostNetwork / hostPID | false | Not used by default; see [10-node-inspector-limitations.md](10-node-inspector-limitations.md) for optional host visibility. |

### 6.5 Mounts

| Volume | Host path | Mount path | Read-only | Purpose |
|--------|-----------|------------|-----------|---------|
| host-proc | /proc | /proc | yes | CPU, memory, load (e.g. /proc/cpuinfo, /proc/meminfo, /proc/loadavg). |
| host-sys | /sys | /sys | yes | Kernel/driver info. |
| host-etc | /etc | /etc | yes | os-release, resolv.conf, etc. |

No runtime sockets are mounted. Container state is aggregated by Kubeowler via the API.

### 6.6 Environment

- **NODE_NAME:** Injected via Downward API (`spec.nodeName`); script uses it for the `node_name` field in JSON.

### 6.7 Scheduling

- **tolerations: operator: Exists** — Allows scheduling onto all nodes (including tainted nodes) so every node runs an inspector Pod.

---

## 7. Integration with Kubeowler

1. **Trigger:** `kubeowler check` (all or nodes) calls `collect_node_inspections()`.
2. **Collection:** List Pods in `kube-system` with label `app=kubeowler-node-inspector`; for each Pod, fetch the `inspector` container log (optionally `previous=true` first), parse one line as JSON.
3. **Report:** Results are written to `ClusterReport.node_inspection_results`; the report includes the “Node Inspection” section (summary, per-category tables, node issues).

If the DaemonSet is not deployed or no matching Pods exist, Kubeowler does not error; the report simply omits node inspection data.

---

## 8. Troubleshooting

| Symptom | Possible cause | Action |
|---------|-----------------|--------|
| ImagePullBackOff | Image missing or private registry without imagePullSecrets | Check image name and tag; ensure cluster can reach the registry; add imagePullSecrets if private. |
| CrashLoopBackOff | Script failure (missing deps, mount issues) | `kubectl logs -n kube-system <pod> -c inspector --previous` to see exit logs. |
| No Node Inspection in report | No DaemonSet Pods or wrong label | Confirm `kubectl get pods -n kube-system -l app=kubeowler-node-inspector` shows Pods and Kubeowler can read Pod logs. |
| Push denied (access to resource denied) | No token or token without Write | Use a Docker Hub Personal Access Token with Read, Write, Delete and log in again. |

---

## 9. References

- **Node inspection JSON schema:** [08-node-inspection-schema.md](08-node-inspection-schema.md)
- **Collection vs. report usage:** [09-node-inspector-collection-gaps.md](09-node-inspector-collection-gaps.md)
- **Limitations and host visibility:** [10-node-inspector-limitations.md](10-node-inspector-limitations.md)
