# Kubeowler Installation Guide

This guide covers installation scenarios for Kubeowler. For building from source and running tests, see [build-and-test.md](build-and-test.md). For full CLI options, see [cli-reference.md](cli-reference.md).

---

## Supported Environments

| Item | Requirement |
|------|-------------|
| **Kubernetes** | 1.23 or later (1.24+ recommended for production) |
| **Architecture** | `amd64` (x86_64), `arm64` (aarch64) |
| **Operating system** | Linux: RHEL 7+, CentOS 7.x, Rocky Linux 8+, AlmaLinux 8+, Ubuntu 18.04+, SUSE / openSUSE Leap, OpenAnolis, Kylin, and compatible distributions |

Pre-built Linux binaries are statically linked (musl) and do not depend on glibc version. The node-inspector image supports the same architectures; see [node-inspector-build-deploy.md](node-inspector-build-deploy.md).

---

## Install from release binary

1. Download the archive for your platform from [GitHub Releases](https://github.com/Ghostwritten/kubeowler/releases).

2. Extract and install the binary to `/usr/local/bin`:

   ```bash
   tar xzf kubeowler-<version>-x86_64-linux.tar.gz
   sudo cp kubeowler /usr/local/bin/
   ```

3. Verify:

   ```bash
   kubeowler check --help
   ```

Ensure `KUBECONFIG` is set or use the default kubeconfig location. You need cluster read access (nodes, pods, namespaces, etc.).

---

## Install node inspector DaemonSet (optional)

For per-node data in the report (disk usage, service status, kernel parameters), deploy the node-inspector DaemonSet. This runs a small Pod on each node and collects host-level metrics.

```bash
kubectl apply -f deploy/node-inspector/daemonset.yaml
```

By default the DaemonSet is created in the `kubeowler` namespace. When running `kubeowler check`, use `--node-inspector-namespace kubeowler` (default) if you use another namespace, pass that flag.

For building and pushing a custom node-inspector image, see [node-inspector-build-deploy.md](node-inspector-build-deploy.md).

---

## Run with Docker

You can run Kubeowler from a container image (if available) with kubeconfig mounted. See [docker-and-kubernetes.md](docker-and-kubernetes.md) for image build, CronJob example, and RBAC.

---

## Production deployment

### System path and Cron

With the binary installed at `/usr/local/bin/kubeowler`:

```bash
# Example: daily at 09:00
0 9 * * * /usr/local/bin/kubeowler check -o /tmp/daily-report-$(date +\%Y\%m\%d).md
```

### Kubernetes CronJob

For running Kubeowler as a CronJob inside the cluster, see [docker-and-kubernetes.md](docker-and-kubernetes.md) for the Docker image and CronJob/RBAC manifests.

---

## Troubleshooting

- **Connection failures:** Verify kubeconfig (`kubectl config current-context`, `kubectl cluster-info`, `kubectl get nodes`).
- **Permission errors:** Ensure the kubeconfig identity has sufficient RBAC (e.g. `kubectl auth can-i get nodes`).

For build and runtime issues, see [troubleshooting.md](troubleshooting.md).
