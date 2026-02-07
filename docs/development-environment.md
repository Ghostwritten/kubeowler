# Linux Development Environment

This guide covers setting up a Kubeowler development and build environment on Linux from scratch, including proxy and offline-friendly setups and common enterprise distributions (RHEL/CentOS/Alma/Rocky, Debian/Ubuntu).

---

## 1. Prerequisites

- Linux x86_64 host with a kubeconfig that can access the cluster (`kubectl get nodes`)
- Access to crates.io or a configured HTTP/HTTPS proxy
- `sudo` (or root)

---

## 2. Optional: Network Proxy

Add to `~/.bashrc` and source it:

```bash
export HTTP_PROXY=http://<proxy-host>:<port>
export HTTPS_PROXY=http://<proxy-host>:<port>
export NO_PROXY=127.0.0.1,localhost,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,.svc,.cluster.local
# Optional: CARGO_HTTP_PROXY=$HTTPS_PROXY
source ~/.bashrc
```

For corporate CAs, install the root certificate into the system trust store:

```bash
# RHEL/CentOS 9+
sudo cp corp-root-ca.crt /etc/pki/ca-trust/source/anchors/
sudo update-ca-trust

# Debian/Ubuntu
sudo cp corp-root-ca.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates
```

---

## 3. System Dependencies

Install the toolchain, OpenSSL development files, pkg-config, curl, and git:

```bash
# RHEL / CentOS / Alma / Rocky (dnf)
sudo dnf -y install gcc gcc-c++ make pkgconfig openssl-devel ca-certificates curl git clang
sudo update-ca-trust || true

# Debian / Ubuntu (apt)
sudo apt-get update -y
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential pkg-config libssl-dev ca-certificates curl git clang
sudo update-ca-certificates || true
```

Verify:

```bash
gcc --version
pkg-config --version
```

---

## 4. Rust Toolchain (rustup)

With proxy configured if needed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

Rust 1.70+ is recommended; use the latest stable channel.

---

## 5. Build the Project

From the project root:

```bash
cd /path/to/kubeowler
source ~/.bashrc || true
source "$HOME/.cargo/env" || true
cargo build --release
ls -lh target/release/kubeowler
```

Ensure network or proxy is available for the first dependency fetch.

### 5.1 Multi-architecture build (optional)

To build both amd64 and arm64:

```bash
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
chmod +x build-multi-arch.sh
./build-multi-arch.sh --release
```

Artifacts: `target/x86_64-unknown-linux-gnu/release/kubeowler` and `target/aarch64-unknown-linux-gnu/release/kubeowler`. For arm64 cross-compilation, some systems need the target linker (e.g. `gcc-aarch64-linux-gnu` on Debian/Ubuntu); see [installation.md](installation.md).

---

## 6. Run Examples

```bash
./target/release/kubeowler check
./target/release/kubeowler check -i nodes
./target/release/kubeowler check -i security
./target/release/kubeowler check -n kube-system
./target/release/kubeowler check -o my-report.md
./target/release/kubeowler check -c ~/.kube/config
```

Output: main report and optional summary (same base name with `-summary.md`).

---

## 7. Docker Build (optional)

```bash
docker build -t kubeowler:latest .
docker run --rm \
  -v ~/.kube/config:/home/kubeowler/.kube/config:ro \
  -v $(pwd)/reports:/app/reports \
  kubeowler:latest check -o /app/reports/report.md
```

See [docker-and-kubernetes.md](docker-and-kubernetes.md) for full Docker and Kubernetes usage.

---

## 8. Quick Reference

- **OpenSSL headers missing:** install `openssl-devel` (dnf) or `libssl-dev` (apt).
- **pkg-config missing:** install `pkgconfig` (dnf) or `pkg-config` (apt).
- **Proxy/CA:** set proxy env vars and install corporate CA; ensure `NO_PROXY` includes cluster CIDRs.
- **Slow crates.io:** use a proxy or a Cargo mirror if available.
- **Kubeconfig:** use `-c /path/to/config` or set `KUBECONFIG`.

For detailed troubleshooting, see [troubleshooting.md](troubleshooting.md).
