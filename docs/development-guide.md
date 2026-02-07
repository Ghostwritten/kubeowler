# Development Guide

This document describes the project layout, how to add inspection types, customize scoring and reporting, run tests, and contribute.

---

## Project Structure

### Core modules

- **K8s client (`src/k8s/`)**  
  - `client.rs`: Kubernetes API client wrapper and unified resource access.  
  - Typed APIs for the resource kinds Kubeowler uses; authentication and connection handling are managed here.

- **Inspections (`src/inspections/`)**  
  - `types.rs`: Data structures for inspection results.  
  - `runner.rs`: Orchestrates inspection types and aggregates results.  
  - `nodes.rs`, `pods.rs`, `resources.rs`, `network.rs`, `storage.rs`, `security.rs`, etc.: Domain-specific checks (node health, pod status, resource usage, network, storage, security).

- **Scoring (`src/scoring/`)**  
  - `scoring_engine.rs`: Weighted scoring and health mapping; produces overall score and priority recommendations.

- **Reporting (`src/reporting/`)**  
  - `generator.rs`: Markdown report generation; main report and optional summary; formatting and localization.

---

## Adding a New Inspection Type

### 1. Define the inspector

Create or extend a module under `src/inspections/`:

```rust
pub struct CustomInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> CustomInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        // Implement checks and return InspectionResult
    }
}
```

### 2. Register in the CLI

In `src/cli/mod.rs`, add the new variant to the inspection-type enum and wire it to the runner.

### 3. Integrate in the runner

In `src/inspections/runner.rs`, add a branch that calls your inspector and pushes its `InspectionResult` into the report.

---

## Customizing Scoring Weights

Edit the weight logic in `src/scoring/scoring_engine.rs` (e.g. `get_inspection_weight`) to assign weights per inspection type. The overall score is derived from module scores and these weights.

---

## Extending Report Output

- Add or adjust sections in `src/reporting/generator.rs` (e.g. `generate_main_report`).  
- To add a new output format, implement a separate generator under `src/reporting/` and hook it into the CLI.

---

## Testing

### Unit tests

Use `#[cfg(test)]` and `#[test]` (and `#[tokio::test]` for async) in the same crate.

### Integration tests

Place integration tests under `tests/` and use the library API or CLI as needed.

```bash
cargo test
cargo test --release
```

---

## Performance and Security

- **Concurrency:** Use async and parallel execution where appropriate (e.g. `tokio::join!`) while respecting API server load.  
- **RBAC:** Request only the permissions required for read-only inspection; avoid cluster-admin.  
- **Sensitive data:** Do not log or report secrets or tokens; sanitize output as needed.

---

## Deployment (development reference)

- **Container:** Use a multi-stage Dockerfile (Rust build stage + minimal runtime image); see [docker-and-kubernetes.md](docker-and-kubernetes.md).  
- **Kubernetes:** Run as a CronJob or one-off Job with a dedicated ServiceAccount and ClusterRole; see [docker-and-kubernetes.md](docker-and-kubernetes.md).

---

## Troubleshooting

- Build or dependency issues: [troubleshooting.md](troubleshooting.md).  
- Linux dev environment and multi-arch: [development-environment.md](development-environment.md).

---

## Contributing

- Follow Rust style (`cargo fmt`, `cargo clippy`).  
- Use clear commit messages (e.g. `feat: …`, `fix: …`, `docs: …`).  
- Open a Pull Request with a short description and reference to any related issue.

For issue codes and inspection reference docs, see [docs/issues/](issues/) and [issues/README.md](issues/README.md).
