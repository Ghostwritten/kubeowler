//! Collects node inspection JSON from kubeowler-node-inspector DaemonSet pods via exec.
//! Does not deploy the DaemonSet; only identifies and collects from existing pods.
//! Runs the script in each pod and reads JSON from stdout (script must write only JSON to stdout; use stderr for diagnostics).
//! Container state counts are filled via Kubernetes API (runtime-agnostic: works with Docker, containerd, CRI-O).

use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{AttachParams, ListParams};
use kube::Api;
use log::debug;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;

use crate::k8s::K8sClient;
use crate::node_inspection::NodeInspectionResult;

const NODE_INSPECTOR_LABEL: &str = "app=kubeowler-node-inspector";
const DEFAULT_NODE_INSPECTOR_NAMESPACE: &str = "kubeowler";
const CONTAINER_NAME: &str = "inspector";
const SCRIPT_PATH: &str = "/node-check-universal.sh";

/// Collects one NodeInspectionResult per node from DaemonSet pods.
/// Lists pods with label app=kubeowler-node-inspector in the given namespace
/// (or `kubeowler` when `namespace` is None). Runs the inspection script via exec in each pod,
/// reads JSON from stdout, parses it. Returns empty vec if DaemonSet is not deployed or no pods found.
pub async fn collect_node_inspections(
    client: &K8sClient,
    namespace: Option<&str>,
) -> Result<Vec<NodeInspectionResult>> {
    let ns = namespace.unwrap_or(DEFAULT_NODE_INSPECTOR_NAMESPACE);
    let pods_api: Api<Pod> = client.pods(Some(ns));
    let list_params = ListParams::default().labels(NODE_INSPECTOR_LABEL);
    let pods = match pods_api.list(&list_params).await {
        Ok(l) => l,
        Err(e) => {
            debug!("Node inspector DaemonSet pods list failed in {}: {}", ns, e);
            return Ok(Vec::new());
        }
    };

    if pods.items.is_empty() {
        debug!("No kubeowler-node-inspector pods found in {}", ns);
        return Ok(Vec::new());
    }

    let attach_params = AttachParams::default()
        .container(CONTAINER_NAME.to_string())
        .stdout(true)
        .stderr(true)
        .stdin(false)
        .tty(false);

    let mut results = Vec::with_capacity(pods.items.len());
    for pod in pods.items {
        let name = pod.metadata.name.as_deref().unwrap_or("unknown");
        let node_name = pod
            .spec
            .as_ref()
            .and_then(|s| s.node_name.as_deref())
            .unwrap_or("")
            .to_string();

        let mut attached = match pods_api.exec(name, [SCRIPT_PATH], &attach_params).await {
            Ok(a) => a,
            Err(e) => {
                debug!("Exec failed for pod {}: {}", name, e);
                continue;
            }
        };

        let mut stdout_buf = String::new();
        if let Some(mut stdout) = attached.stdout() {
            if let Err(e) = stdout.read_to_string(&mut stdout_buf).await {
                debug!("Read stdout failed for pod {}: {}", name, e);
                let _ = attached.join().await;
                continue;
            }
        }
        if let Err(e) = attached.join().await {
            debug!("Exec join failed for pod {}: {}", name, e);
        }

        let trimmed = stdout_buf.trim();
        if trimmed.is_empty() {
            debug!("Empty stdout for pod {}", name);
            continue;
        }

        // Script outputs a single JSON object to stdout
        let parsed: NodeInspectionResult = serde_json::from_str(trimmed).with_context(|| {
            format!("Parse node inspection JSON from pod {}: {}", name, trimmed)
        })?;

        // Prefer node name from pod spec if script didn't set it
        let mut result = parsed;
        if result.node_name.is_empty() && !node_name.is_empty() {
            result.node_name = node_name;
        }
        if result.hostname.is_empty() {
            result.hostname = result.node_name.clone();
        }
        results.push(result);
    }

    results.sort_by(|a, b| a.node_name.cmp(&b.node_name));

    // Fill container_state_counts from Kubernetes API (runtime-agnostic).
    fill_container_state_counts(client, &mut results).await;

    Ok(results)
}

/// Lists all pods cluster-wide, aggregates container states per node, and sets container_state_counts on each result.
async fn fill_container_state_counts(client: &K8sClient, results: &mut [NodeInspectionResult]) {
    let pods_api: Api<Pod> = client.pods(None);
    let list_params = ListParams::default();
    let all_pods = match pods_api.list(&list_params).await {
        Ok(l) => l,
        Err(e) => {
            debug!("List all pods for container state counts failed: {}", e);
            return;
        }
    };

    // node_name -> (running, waiting, exited)
    let mut per_node: HashMap<String, (u32, u32, u32)> = HashMap::new();
    for pod in all_pods.items {
        let node_name = pod
            .spec
            .as_ref()
            .and_then(|s| s.node_name.as_deref())
            .unwrap_or("");
        if node_name.is_empty() {
            continue;
        }
        let status = match &pod.status {
            Some(s) => s,
            None => continue,
        };
        let init = status.init_container_statuses.as_deref().unwrap_or(&[]);
        let main = status.container_statuses.as_deref().unwrap_or(&[]);
        for cs in init.iter().chain(main.iter()) {
            let entry = per_node.entry(node_name.to_string()).or_insert((0, 0, 0));
            if let Some(ref state) = cs.state {
                if state.running.is_some() {
                    entry.0 += 1;
                } else if state.waiting.is_some() {
                    entry.1 += 1;
                } else if state.terminated.is_some() {
                    entry.2 += 1;
                } else {
                    entry.1 += 1; // default waiting
                }
            }
        }
    }

    for result in results.iter_mut() {
        if let Some(&(running, waiting, exited)) = per_node.get(&result.node_name) {
            let mut counts = HashMap::new();
            if running > 0 {
                counts.insert("running".to_string(), running);
            }
            if exited > 0 {
                counts.insert("exited".to_string(), exited);
            }
            if waiting > 0 {
                counts.insert("waiting".to_string(), waiting);
            }
            if !counts.is_empty() {
                result.container_state_counts = Some(counts);
            }
        }
    }
}
