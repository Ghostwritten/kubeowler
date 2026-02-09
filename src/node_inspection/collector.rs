//! Collects node inspection JSON from kubeowler-node-inspector DaemonSet pods via Pod logs.
//! Does not deploy the DaemonSet; only identifies and collects from existing pods.
//! The container runs the script once at startup and writes JSON to stdout (Pod logs).
//! Kubeowler fetches each pod's log and parses the JSON. Data is from container start time;
//! restart DaemonSet pods to refresh. Container state counts are filled via Kubernetes API.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use k8s_openapi::api::apps::v1::DaemonSet;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, LogParams, Patch, PatchParams};
use kube::Api;
use log::debug;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::k8s::K8sClient;
use crate::node_inspection::NodeInspectionResult;

const NODE_INSPECTOR_LABEL: &str = "app=kubeowler-node-inspector";
const DEFAULT_NODE_INSPECTOR_NAMESPACE: &str = "kubeowler";
const CONTAINER_NAME: &str = "inspector";
const DAEMONSET_NAME: &str = "kubeowler-node-inspector";
#[allow(dead_code)]
const STALENESS_THRESHOLD_HOURS: u64 = 24;
const ROLLOUT_WAIT_TIMEOUT_SECS: u64 = 180;
const LOG_POLL_INTERVAL_SECS: u64 = 6;
const LOG_POLL_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Status of node inspector pre-check before collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeInspectorStatus {
    /// No DaemonSet pods found in the given namespace.
    NotDeployed,
    /// Data is fresh (within staleness threshold), ready to collect.
    Ready,
    /// Data was stale, DaemonSet pods were restarted and are ready for collection.
    RestartedAndReady,
    /// Timeout waiting for logs; proceed with partial data (ready of total pods have logs).
    ReadyPartial { ready: usize, total: usize },
}

/// Polls for non-empty logs from Running pods. Returns (timestamps, ready_count, total_running, timed_out).
async fn poll_for_logs(
    pods_api: &Api<Pod>,
    running_pod_names: &[String],
    log_params: &LogParams,
) -> (Vec<DateTime<Utc>>, usize, usize, bool) {
    let total = running_pod_names.len();
    let deadline = Instant::now() + Duration::from_secs(LOG_POLL_TIMEOUT_SECS);
    let mut elapsed_secs: u64 = 0;

    loop {
        let mut timestamps: Vec<DateTime<Utc>> = Vec::with_capacity(total);
        let mut ready_count = 0usize;
        for name in running_pod_names {
            let log_content = match pods_api.logs(name, log_params).await {
                Ok(s) => s,
                Err(_) => continue,
            };
            let trimmed = log_content.trim();
            if trimmed.is_empty() {
                continue;
            }
            ready_count += 1;
            let v: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let ts_str = match v.get("timestamp").and_then(|t| t.as_str()) {
                Some(s) if !s.is_empty() => s,
                _ => continue,
            };
            if let Ok(dt) = DateTime::parse_from_rfc3339(ts_str) {
                timestamps.push(dt.with_timezone(&Utc));
            }
        }

        if ready_count >= total {
            return (timestamps, ready_count, total, false);
        }
        if Instant::now() >= deadline {
            return (timestamps, ready_count, total, true);
        }

        println!(
            "   {}  ({}, {}/{} pods have logs)",
            "Waiting for node inspector logs...".bright_yellow(),
            format_duration(elapsed_secs),
            ready_count,
            total
        );
        sleep(Duration::from_secs(LOG_POLL_INTERVAL_SECS)).await;
        elapsed_secs += LOG_POLL_INTERVAL_SECS;
    }
}

fn format_duration(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

fn is_pod_running(pod: &Pod) -> bool {
    pod.status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .map(|p| p == "Running")
        .unwrap_or(false)
}

/// Ensures node inspector data is fresh before collection.
/// 1. No pods running → NotDeployed. 2. Pods running but no logs → poll (6s interval, 5 min timeout).
/// 3. Has logs → check staleness; if >24h restart DaemonSet and poll again.
/// On timeout: proceed with partial data (ReadyPartial).
pub async fn ensure_node_inspector_ready(
    client: &K8sClient,
    namespace: &str,
    staleness_hours: u64,
) -> NodeInspectorStatus {
    let pods_api: Api<Pod> = client.pods(Some(namespace));
    let list_params = ListParams::default().labels(NODE_INSPECTOR_LABEL);
    let pods = match pods_api.list(&list_params).await {
        Ok(l) => l,
        Err(e) => {
            debug!(
                "Node inspector DaemonSet pods list failed in {}: {}",
                namespace, e
            );
            return NodeInspectorStatus::NotDeployed;
        }
    };

    if pods.items.is_empty() {
        debug!("No kubeowler-node-inspector pods found in {}", namespace);
        return NodeInspectorStatus::NotDeployed;
    }

    let running_pod_names: Vec<String> = pods
        .items
        .iter()
        .filter(|p| is_pod_running(p))
        .filter_map(|p| p.metadata.name.clone())
        .collect();

    if running_pod_names.is_empty() {
        debug!("No Running kubeowler-node-inspector pods in {}", namespace);
        return NodeInspectorStatus::NotDeployed;
    }

    let log_params = LogParams {
        container: Some(CONTAINER_NAME.to_string()),
        ..LogParams::default()
    };

    // Poll for logs (6s interval, 5 min timeout)
    let (timestamps, ready_count, total, timed_out) =
        poll_for_logs(&pods_api, &running_pod_names, &log_params).await;

    if timed_out {
        println!(
            "{}  Node inspector: {}/{} pods have logs (timeout 5 min). Proceeding with partial data.",
            "⚠️".bright_yellow(),
            ready_count,
            total
        );
        return NodeInspectorStatus::ReadyPartial {
            ready: ready_count,
            total,
        };
    }

    // Check staleness
    let oldest = timestamps.iter().min().copied();
    let now = Utc::now();
    let needs_restart = match oldest {
        Some(oldest_ts) => {
            let age_hours = (now - oldest_ts).num_seconds() as u64 / 3600;
            age_hours >= staleness_hours
        }
        None => false,
    };

    if !needs_restart {
        return NodeInspectorStatus::Ready;
    }

    // Patch DaemonSet to trigger rollout restart
    let ds_api: Api<DaemonSet> = client.daemon_sets(Some(namespace));
    let restarted_at = now.to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": restarted_at
                    }
                }
            }
        }
    });
    if ds_api
        .patch(
            DAEMONSET_NAME,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
        .is_err()
    {
        debug!(
            "Failed to patch DaemonSet {} in {}",
            DAEMONSET_NAME, namespace
        );
        return NodeInspectorStatus::NotDeployed;
    }

    // Wait for rollout
    let deadline = Instant::now() + Duration::from_secs(ROLLOUT_WAIT_TIMEOUT_SECS);
    while Instant::now() < deadline {
        let ds = match ds_api.get(DAEMONSET_NAME).await {
            Ok(d) => d,
            Err(_) => {
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };
        let status = match &ds.status {
            Some(s) => s,
            None => {
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };
        let desired = status.desired_number_scheduled;
        let ready = status.number_ready;
        if desired > 0 && ready >= desired {
            break;
        }
        sleep(Duration::from_secs(2)).await;
    }

    // Re-list and poll for logs again (new pods after restart)
    let pods2 = match pods_api.list(&list_params).await {
        Ok(l) => l,
        Err(_) => return NodeInspectorStatus::NotDeployed,
    };
    let running_pod_names2: Vec<String> = pods2
        .items
        .iter()
        .filter(|p| is_pod_running(p))
        .filter_map(|p| p.metadata.name.clone())
        .collect();
    if running_pod_names2.is_empty() {
        return NodeInspectorStatus::NotDeployed;
    }

    let (_, ready_count2, total2, timed_out2) =
        poll_for_logs(&pods_api, &running_pod_names2, &log_params).await;

    if timed_out2 {
        println!(
            "{}  Node inspector: restarted; {}/{} pods have logs (timeout 5 min). Proceeding with partial data.",
            "⚠️".bright_yellow(),
            ready_count2,
            total2
        );
        return NodeInspectorStatus::ReadyPartial {
            ready: ready_count2,
            total: total2,
        };
    }

    NodeInspectorStatus::RestartedAndReady
}

/// Collects one NodeInspectionResult per node from DaemonSet pods.
/// Lists pods with label app=kubeowler-node-inspector in the given namespace
/// (or `kubeowler` when `namespace` is None). Fetches each pod's container log
/// (script output from startup), parses JSON. Returns empty vec if DaemonSet is not deployed or no pods found.
/// Note: Data reflects node state at pod start time; restart pods to refresh.
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

    let log_params = LogParams {
        container: Some(CONTAINER_NAME.to_string()),
        ..LogParams::default()
    };

    let mut results = Vec::with_capacity(pods.items.len());
    for pod in pods.items {
        let name = pod.metadata.name.as_deref().unwrap_or("unknown");
        let node_name = pod
            .spec
            .as_ref()
            .and_then(|s| s.node_name.as_deref())
            .unwrap_or("")
            .to_string();

        let log_content = match pods_api.logs(name, &log_params).await {
            Ok(s) => s,
            Err(e) => {
                debug!("Fetch logs failed for pod {}: {}", name, e);
                continue;
            }
        };

        let trimmed = log_content.trim();
        if trimmed.is_empty() {
            debug!("Empty logs for pod {}", name);
            continue;
        }

        // Script outputs a single JSON object to stdout at container start
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
