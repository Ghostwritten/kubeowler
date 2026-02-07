# Troubleshooting

This document lists common issues and resolutions when building and running Kubeowler on Linux.

---

## 1. Dependencies and Build

### 1.1 OpenSSL / native dependency errors

**Symptom:** Build failures involving `openssl-sys`, `openssl`, `native-tls`, etc.

**Fix:**

```bash
# RHEL/CentOS/Alma/Rocky
dnf -y install openssl-devel pkgconfig gcc gcc-c++ make

# Debian/Ubuntu
apt-get update -y
DEBIAN_FRONTEND=noninteractive apt-get install -y libssl-dev pkg-config build-essential
```

### 1.2 pkg-config not found

**Symptom:** Logs report `pkg-config` not found.

**Fix:** Install `pkgconfig` (dnf) or `pkg-config` (apt).

### 1.3 Compile errors from dependency API changes

**Symptom:** Errors from `kube` / `k8s-openapi` (e.g. E0560, E0308) after upgrading.

**Typical adjustments:**
- Use `Config::infer()` for kubeconfig and set `KUBECONFIG` when needed.
- Use `.to_string()` where a struct now expects `String` instead of `&str`.
- For `ClusterRoleBinding.role_ref`, use it directly if the crate no longer wraps it in `Option`.

---

## 2. Network and Proxy

### 2.1 crates.io slow or unreachable

**Fix:** Set proxy and CA trust:

```bash
export HTTP_PROXY=http://<proxy>:<port>
export HTTPS_PROXY=http://<proxy>:<port>
export NO_PROXY=127.0.0.1,localhost,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,.svc,.cluster.local
source ~/.bashrc
# Install corporate CA if required
sudo cp corp-root-ca.crt /etc/pki/ca-trust/source/anchors/
sudo update-ca-trust   # or update-ca-certificates on Debian/Ubuntu
```

### 2.2 Certificate verify failed

**Symptom:** TLS errors such as "certificate verify failed".

**Fix:** Update system CA store and add the corporate root CA if behind an intercepting proxy; ensure the proxy terminates TLS correctly.

---

## 3. Runtime

### 3.1 Kubeconfig not used

**Symptom:** Kubernetes client connection failures.

**Fix:**

```bash
./target/release/kubeowler check -c ~/.kube/config
# or
export KUBECONFIG=~/.kube/config
```

### 3.2 Insufficient permissions (RBAC)

**Symptom:** List or get operations are denied.

**Fix:** Use a kubeconfig with sufficient RBAC, or assign a Role/ClusterRole and binding to the identity (e.g. ServiceAccount) used to run Kubeowler.

---

## 4. Performance

- First build may take longer while dependencies are downloaded.
- Use `cargo build --release` for a smaller, faster binary.
- Parallel inspection may be improved in future releases.

---

## 5. Logging and debugging

```bash
RUST_LOG=debug ./target/release/kubeowler check
RUST_LOG=error ./target/release/kubeowler check
cargo clean && cargo build --release 2>&1 | grep warning || true
```

---

If you hit issues not covered here (e.g. specific kernel or network setups), please open a GitHub issue with the error output so we can extend this document.
