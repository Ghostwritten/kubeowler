# Kubeowler Documentation

This directory contains the official documentation for Kubeowler, organized by number and topic. In addition to the root [README.md](../README.md), all project documentation is maintained here for reference and contribution.

---

## Document Index

| No. | Document | Description |
|-----|----------|-------------|
| 01 | [01-installation-guide.md](01-installation-guide.md) | **Installation and Running Guide** — Supported environments, Rust setup, build, run, report output, troubleshooting, and production deployment |
| 02 | [02-docker-guide.md](02-docker-guide.md) | **Docker and Kubernetes Deployment** — Image build, container run, CronJob example, and RBAC |
| 03 | [03-development-guide.md](03-development-guide.md) | **Development Guide** — Project structure, adding inspection types, scoring weights, report extension, testing, and contribution workflow |
| 04 | [04-linux-dev-setup.md](04-linux-dev-setup.md) | **Linux Development Environment** — Proxy, system dependencies, Rust install, build and run on Linux (including enterprise and offline setups) |
| 05 | [05-troubleshooting.md](05-troubleshooting.md) | **Troubleshooting** — Dependencies and build, network and proxy, runtime, logging and debugging |
| 06 | [06-node-inspector-build-deploy.md](06-node-inspector-build-deploy.md) | **Node Inspector: Build and Deploy** — Image build, push to registry, DaemonSet deployment, and integration with Kubeowler |
| 07 | [07-data-collection.md](07-data-collection.md) | **How Kubeowler Collects Cluster Data** — Connection and K8s client, cluster overview, module-based inspections, node inspection (DaemonSet and Pod logs), report structure and generation |
| 08 | [08-node-inspection-schema.md](08-node-inspection-schema.md) | **Node Inspection JSON Schema** — Structure and field definitions of the DaemonSet script output consumed by Kubeowler |
| 09 | [09-node-inspector-collection-gaps.md](09-node-inspector-collection-gaps.md) | **Node Inspector: Collection vs. Report Usage** — Fields that are collected and parsed but not currently displayed in the report |
| 10 | [10-node-inspector-limitations.md](10-node-inspector-limitations.md) | **Node Inspector: Limitations** — DaemonSet constraints, host visibility, and data-source caveats |

---

## Issue Codes and Reference Docs

Inspection findings use **issue codes** (e.g. NODE-001, POD-003). Detailed descriptions (summary, severity, symptoms, resolution, references) are in [docs/issues/](issues/). The index is at [issues/README.md](issues/README.md).

---

## Recommended Reading Order

- **First-time users:** 01 → 02 (if using container deployment).
- **Node inspection (DaemonSet):** 06, 08, 09, 10.
- **Contributors:** 03, 04, 05.
- **Data flow and report design:** 07.
