# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-02-09

### Added

- Report header and filename use cluster host local time (from first node `timestamp_local` when node inspection present; fallback UTC).
- Node Certificate Status table: Level and Issue Code columns (CERT-002/CERT-003).
- TLS Certificate Expiry table: Expired (Yes/No) column, "Days to Expiry" column name, column order aligned with node cert table; Certificate (subject) column removed.
- Short English descriptions before Node security/kernel/network and Node kernel parameters tables.
- `timestamp_local` in node script and NodeInspectionResult for report time.

### Changed

- Node disk and certificate paths in report shown in host perspective (strip `/host` prefix).
- Node Certificate Status time column labeled "Expiration Date (node local)".
- Documentation: node-inspector namespace default is `kubeowler` (not kube-system); data-collection and node-inspector-build-deploy updated; node-inspection-schema and collection-gaps updated for `timestamp_local`, path display, and cert columns.

### Fixed

- Report page footer link: corrected repository URL from username/kubeowler to Ghostwritten/kubeowler.
- `timestamp_local` now uses host timezone when `/host/etc/localtime` is available so report header/filename reflect cluster host local time (e.g. CST).

[0.1.2]: https://github.com/Ghostwritten/kubeowler/releases/tag/v0.1.2

## [0.1.1] - 2026-02-06

### Added

- HTML report: Kubescape-style layout (CSS variables, table styling, word-break). Logo shown in report; logo embedded as Data URI so the generated HTML is self-contained (no external assets folder needed when using the released binary).
- Logo assets in `assets/` (logo.png, logo.svg); README and report reference updated.

### Changed

- Documentation reorganized: topic-based filenames in `docs/` (e.g. installation.md, cli-reference.md). Example report filenames unified to `report.*` (md, json, csv, html).
- GitHub Release notes: release workflow now uses CHANGELOG content as the release body so the [Releases](https://github.com/Ghostwritten/kubeowler/releases) page shows a clear update overview.

[0.1.1]: https://github.com/Ghostwritten/kubeowler/releases/tag/v0.1.1

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
