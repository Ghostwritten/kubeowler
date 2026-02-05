# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-05

### Added

- Initial release.
- Cluster inspection via `kubeowler check`: nodes, pods, network, storage, security, resources, control plane, autoscaling, batch/cron, policies, observability, upgrade readiness, certificates.
- Report formats: Markdown (default), JSON, CSV, HTML.
- Node inspector integration: optional DaemonSet for per-node data (resources, services, kernel, certificates); see `deploy/node-inspector/` and docs.
- Check level filter for report: `all` or comma-separated `Info`, `warning`, `critical`.
- Documentation in `docs/`: installation, Docker, development, troubleshooting, node inspector build/deploy, data collection, node inspection schema and limitations.
- Example reports in `example/` (md, json, csv, html).
- Build scripts: `build.sh`, `build-multi-arch.sh`; Dockerfile for container image.

[0.1.0]: https://github.com/Ghostwritten/kubeowler/releases/tag/v0.1.0
