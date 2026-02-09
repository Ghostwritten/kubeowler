# CLI Usage Reference

This document describes the `kubeowler` command and its options. For installation, see [installation.md](installation.md).

---

## Command overview

```text
kubeowler [OPTIONS] <COMMAND>
```

The only subcommand is **check**, which runs a full cluster inspection and writes a report.

---

## kubeowler check

Run cluster inspection and generate a report.

```bash
kubeowler check [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--cluster-name <NAME>` | | Cluster name used in the report title | From kubeconfig or "default" |
| `--namespace <NAMESPACE>` | `-n` | Inspect only resources in this namespace | All namespaces |
| `--node-inspector-namespace <NAMESPACE>` | | Namespace where the kubeowler-node-inspector DaemonSet runs | `kubeowler` |
| `--output <PATH>` | `-o` | Output file path for the report | `{cluster-name}-kubernetes-inspection-report-{timestamp}.{ext}` |
| `--format <FORMAT>` | `-f` | Output format: `md`, `json`, `csv`, or `html` | `md` |
| `--config-file <PATH>` | `-c` | Kubernetes config file path | `KUBECONFIG` or `~/.kube/config` |
| `--level <LEVELS>` | `-l` | Check levels to include in the report: `all` or comma-separated `info,warning,critical` | `warning,critical` |

### Examples

Full cluster check, default output file and Markdown format:

```bash
kubeowler check
```

Limit inspection to a single namespace:

```bash
kubeowler check --namespace kube-system
```

Write report to a specific file in JSON format:

```bash
kubeowler check --output prod-report.json --format json
```

Use a custom kubeconfig:

```bash
kubeowler check --config-file ~/.kube/config-prod
```

Include all severity levels (info, warning, critical) in the report:

```bash
kubeowler check --level all
```

Node inspector runs in a different namespace:

```bash
kubeowler check --node-inspector-namespace my-namespace
```

Combined:

```bash
kubeowler check -n default -o report.md -f md -l warning,critical
```

---

## Environment variables

| Variable | Description |
|----------|-------------|
| `KUBECONFIG` | Path to kubeconfig file. Overridden by `--config-file` if set. |
| `RUST_LOG` | Log level (e.g. `info`, `debug`, `error`). Useful for troubleshooting. |

---

## Output formats

- **md** (default): Markdown report with tables and issue links.
- **json**: Structured JSON for tooling or dashboards.
- **csv**: Flat CSV for spreadsheets.
- **html**: HTML report.

The default output filename is derived from the cluster name and a timestamp. When node inspection data is available, the timestamp is in **cluster host local time** (from the first node's `timestamp_local`); otherwise it is UTC. Use `--output` to override.
