<div align="center">
  <img src="assets/logo.png" alt="Kubeowler" width="200"/>
</div>

# Kubeowler - Kubernetes Cluster Checker

> 🔍 A high-performance Kubernetes cluster health checking tool written in Rust

## 📖 Overview

Kubeowler is built for platform/SRE teams to evaluate Kubernetes clusters. It checks health, security posture, and resource efficiency, then generates a detailed Markdown report (English by default).

## ✨ Features

- **🔍 Comprehensive Checks**: nodes, pods, network, storage, security, resources
- **📊 Intelligent Scoring**: weighted scoring to highlight weak areas
- **📋 Detailed Report**: single report by default (Markdown, JSON, CSV, or HTML); detailed results are **grouped by Kubernetes resource object** (Node, Pod, Service, etc.) for easier review
- **🎯 Actionable Advice**: concrete remediation tips per issue
- **⚡ High Performance**: asynchronous Rust implementation

## 🏗️ Architecture

```
kubeowler/
├── src/
│   ├── main.rs             # binary entry
│   ├── lib.rs              # library entry
│   ├── cli/                # CLI parsing
│   ├── k8s/                # Kubernetes client wrappers
│   ├── inspections/        # check modules
│   ├── scoring/            # scoring engine
│   ├── reporting/          # report generation
│   └── utils/              # shared helpers
└── tests/                  # integration tests
```

## 🚀 Getting Started

### Requirements

- Rust 1.70+ (stable) for building from source
- **Kubernetes 1.23+** (1.24+ recommended for production)
- Access to a Kubernetes cluster and kubeconfig with read permissions

### Supported platforms

- **Kubernetes**: 1.23 or later (see [docs/installation.md](docs/installation.md) for details).
- **Architectures**: `amd64` (x86_64), `arm64` (aarch64) for both the kubeowler binary and the node-inspector image.
- **Operating systems** (Linux): Pre-built Linux binaries are **statically linked (musl)** and do not depend on glibc version, so they run on RHEL 7/8/9, CentOS 7.x, Rocky Linux 8+, AlmaLinux 8+, Ubuntu 18.04+, SUSE / openSUSE, OpenAnolis, Kylin, and other distros. The node-inspector DaemonSet image runs on the same OS when used on cluster nodes.

### Installation

Pre-built binaries are on [GitHub Releases](https://github.com/Ghostwritten/kubeowler/releases). Download, install to `/usr/local/bin`, and verify:

| Platform | Architecture | File |
|----------|--------------|------|
| Linux    | amd64        | `kubeowler-<version>-x86_64-linux.tar.gz` |
| Linux    | arm64        | `kubeowler-<version>-aarch64-linux.tar.gz` |

```bash
curl -sSL https://github.com/Ghostwritten/kubeowler/releases/download/v0.1.1/kubeowler-v0.1.1-x86_64-linux.tar.gz | tar xz
sudo cp kubeowler /usr/local/bin/
kubeowler check --help
```

### Node inspector (optional)

For per-node data (disk, services, kernel parameters) in the report, deploy the DaemonSet:

```bash
kubectl apply -f deploy/node-inspector/daemonset.yaml
```

See [docs/node-inspector-build-deploy.md](docs/node-inspector-build-deploy.md) for image build and details.

### Build from source

```bash
git clone https://github.com/Ghostwritten/kubeowler.git
cd kubeowler
cargo build --release
```

## 📚 Usage

```bash
# Full cluster check (default)
kubeowler check

# Specify namespace
kubeowler check --namespace kube-system

# Custom output file and format (md, json, csv, html)
kubeowler check --output my-report.md
kubeowler check -o report.json -f json

# Custom kubeconfig
kubeowler check --config-file ~/.kube/config

# Node inspector DaemonSet namespace (default: kubeowler)
kubeowler check --node-inspector-namespace kubeowler

# Report levels: all, or comma-separated (e.g. warning,critical)
kubeowler check --level warning,critical
```

Set `KUBECONFIG` if not using the default. For more options see [docs/cli-reference.md](docs/cli-reference.md).

## 🧪 Testing

```bash
cargo test
```

## 📈 Reports

A single report file is generated per run. Default name: `{cluster-name}-kubernetes-inspection-report-{YYYY-MM-DD-HHMMSS}.{ext}`. Formats: Markdown (default), JSON, CSV, HTML. Sample reports are in the [example/](example/) directory.

## 📚 Documentation

All project docs (installation, Docker, development, troubleshooting, etc.) live in **[docs/](docs/)** with numbered filenames. See [docs/README.md](docs/README.md) for the index.

## 📄 License

MIT