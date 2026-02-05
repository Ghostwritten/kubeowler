# Kubeowler Installation and Running Guide

This guide describes how to install, build, and run Kubeowler. Ensure your environment meets the supported requirements before proceeding.

---

## Supported Environments

| Item | Requirement |
|------|-------------|
| **Kubernetes** | 1.23 or later (1.24+ recommended for production) |
| **Architecture** | `amd64` (x86_64), `arm64` (aarch64) |
| **Operating system** | glibc-compatible Linux: RHEL 7+, CentOS 7.x, Rocky Linux 8+, AlmaLinux 8+, Ubuntu 20.04 LTS+, SUSE Linux Enterprise 12+ / openSUSE Leap, OpenAnolis, Kylin, and compatible distributions |

Binaries are built as `*-unknown-linux-gnu` (glibc). The node-inspector image supports the same architectures (linux/amd64, linux/arm64); see [06-node-inspector-build-deploy.md](06-node-inspector-build-deploy.md).

---

## Step 1: Install Rust

Choose one of the following methods depending on your network and platform.

### Option A: Homebrew (macOS)

```bash
brew install rust
source ~/.cargo/env  # if needed
rustc --version
cargo --version
```

### Option B: Official rustup script

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustc --version
cargo --version
```

### Option C: Manual install

See [Rust installation documentation](https://forge.rust-lang.org/infra/channel-layout.html) for platform-specific packages and manual installation.

---

## Step 2: Build Kubeowler

From the project root:

### Using the build script (recommended)

```bash
chmod +x build.sh
./build.sh              # debug build
./build.sh --release     # release build
./build.sh --release --test   # build and run tests
```

### Manual build

```bash
cargo check
cargo build             # debug
cargo build --release   # release
cargo test
cargo fmt --check
cargo clippy
```

---

## Step 3: Run Kubeowler

### Basic usage

```bash
./target/release/kubeowler check --help
./target/release/kubeowler check
```

Default behavior: full cluster inspection, output to `kubeowler-report.md` (and optional summary file).

### Inspection type

```bash
./target/release/kubeowler check -i nodes
./target/release/kubeowler check -i security
./target/release/kubeowler check -i pods
# Types: all, nodes, pods, resources, network, storage, security
```

### Namespace

```bash
./target/release/kubeowler check -n kube-system
./target/release/kubeowler check -n default
```

### Output file

```bash
./target/release/kubeowler check -o my-cluster-report.md
```

### Kubeconfig

```bash
./target/release/kubeowler check -c ~/.kube/config-prod
```

### Combined example

```bash
./target/release/kubeowler check -i security -c ~/.kube/config-prod -o prod-security-report.md
```

---

## Viewing Reports

After a run, Kubeowler produces:

- **Main report** — Executive summary, overall score, detailed results, score breakdown, key findings and recommendations.
- **Summary report** (optional) — Issue statistics, critical findings, high-priority items, and remediation suggestions.

Open with any Markdown viewer or editor. To convert to HTML (if pandoc is installed):

```bash
pandoc kubeowler-report.md -o kubeowler-report.html
```

---

## Troubleshooting

- **Connection failures:** Verify kubeconfig (`kubectl config current-context`, `kubectl cluster-info`, `kubectl get nodes`).
- **Permission errors:** Ensure the kubeconfig identity has sufficient RBAC (`kubectl auth can-i get nodes`, etc.).
- **Build errors:** Try `cargo clean` then `cargo build --release`.

### Debug logging

```bash
RUST_LOG=debug ./target/release/kubeowler check
RUST_LOG=error ./target/release/kubeowler check
```

For more detail, see [05-troubleshooting.md](05-troubleshooting.md).

---

## Multi-architecture build

To build for other architectures (e.g. from x86_64 for arm64):

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

Cross-compiling to arm64 may require a target linker (e.g. `gcc-aarch64-linux-gnu` on Debian/Ubuntu). See [Rust cross-compilation](https://rust-lang.github.io/rustup/cross-compilation.html). The project script `build-multi-arch.sh` can build both targets; see [04-linux-dev-setup.md](04-linux-dev-setup.md).

---

## Production Deployment

### Install to system path

```bash
sudo cp target/release/kubeowler /usr/local/bin/
kubeowler check --help
```

### Cron

```bash
crontab -e
# Example: daily at 09:00
0 9 * * * /usr/local/bin/kubeowler check -o /tmp/daily-report-$(date +\%Y\%m\%d).md
```

### Kubernetes CronJob

See [02-docker-guide.md](02-docker-guide.md) for image build and CronJob/RBAC manifests.
