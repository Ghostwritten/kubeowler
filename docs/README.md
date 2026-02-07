# Kubeowler Documentation

Official documentation for [Kubeowler](https://github.com/Ghostwritten/kubeowler) — a Kubernetes cluster health checking tool. Documentation is organized by audience and topic, following common open-source doc conventions.

---

## Documentation Structure

### Getting Started

| Document | Description |
|----------|-------------|
| [Installation](installation.md) | Supported environments, install from release binary, node inspector DaemonSet, Docker, and production deployment |
| [Docker and Kubernetes](docker-and-kubernetes.md) | Image build, container run, CronJob example, and RBAC for running Kubeowler in cluster |

### User Guide

| Document | Description |
|----------|-------------|
| [CLI Reference](cli-reference.md) | `kubeowler check` options, examples, and output formats (MD, JSON, CSV, HTML) |

### Concepts

| Document | Description |
|----------|-------------|
| [Data Collection](data-collection.md) | How Kubeowler connects to the cluster, runs module-based inspections, collects node inspector output, and generates the report |

### Node Inspector

Per-node inspection is provided by an optional DaemonSet that runs a script on each node. Kubeowler aggregates its JSON output into the report.

| Document | Description |
|----------|-------------|
| [Build and Deploy](node-inspector-build-deploy.md) | Build and push the node-inspector image, deploy the DaemonSet, integrate with Kubeowler |
| [Node Inspection Schema](node-inspection-schema.md) | JSON structure and field definitions of the DaemonSet script output |
| [Collection vs. Report Usage](node-inspector-collection-gaps.md) | Fields that are collected and parsed but not yet displayed in the report |
| [Limitations](node-inspector-limitations.md) | DaemonSet constraints, host visibility, and data-source caveats |

### Development

| Document | Description |
|----------|-------------|
| [Development Guide](development-guide.md) | Project structure, adding inspection types, scoring weights, report extension, contribution workflow |
| [Development Environment](development-environment.md) | Linux dev setup: proxy, system dependencies, Rust install, build and run (including enterprise and offline) |
| [Build and Test](build-and-test.md) | Rust toolchain, clone, build, test, format and clippy, multi-arch and cross builds |

### Operations

| Document | Description |
|----------|-------------|
| [Troubleshooting](troubleshooting.md) | Dependencies and build, network and proxy, runtime, logging and debugging |

---

## Issue Codes Reference

Inspection findings use **issue codes** (e.g. `NODE-001`, `POD-003`). Detailed descriptions (summary, severity, symptoms, resolution, references) are in [docs/issues/](issues/). Index: [issues/README.md](issues/README.md).

---

## Recommended Reading

- **First-time users:** [Installation](installation.md) → [CLI Reference](cli-reference.md); [Docker and Kubernetes](docker-and-kubernetes.md) if using containers.
- **Node inspection (DaemonSet):** [Build and Deploy](node-inspector-build-deploy.md) → [Node Inspection Schema](node-inspection-schema.md) → [Collection vs. Report Usage](node-inspector-collection-gaps.md) → [Limitations](node-inspector-limitations.md).
- **Contributors:** [Development Guide](development-guide.md) → [Development Environment](development-environment.md) → [Build and Test](build-and-test.md).
- **Data flow and report design:** [Data Collection](data-collection.md).
