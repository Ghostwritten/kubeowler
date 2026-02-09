use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::api::ListParams;
use std::collections::HashMap;
use uuid::Uuid;

use super::types::{
    CheckResult, CheckStatus, ClusterOverview, ClusterReport, ContainerUsageRow, EventRow,
    ExecutiveSummary, HealthStatus, InspectionResult, InspectionSummary, Issue, IssueSeverity,
    NodeConditionsRow, NodeResourceSummary, NodeRow, NodeUsageRow, PodPhaseBreakdown,
    StorageSummary, WorkloadSummary,
};
use super::{
    autoscaling, batch, certificates, control_plane, namespace_summary, network, nodes,
    observability, pods, policies, resources, security, storage, upgrade,
};
use crate::cli::InspectionType;
use crate::k8s::K8sClient;
use crate::node_inspection::{
    collect_node_inspections, ensure_node_inspector_ready, NodeInspectionResult,
    NodeInspectorStatus,
};
use crate::utils::resource_quantity::{parse_cpu_str, parse_memory_str};

fn parse_cpu_quantity(q: Option<&Quantity>) -> Option<i64> {
    q.and_then(|q| parse_cpu_str(q.0.as_str()))
}

fn parse_memory_quantity(q: Option<&Quantity>) -> Option<i64> {
    q.and_then(|q| parse_memory_str(q.0.as_str()))
}

fn format_cpu_millis(millis: i64) -> String {
    if millis % 1000 == 0 {
        format!("{}", millis / 1000)
    } else {
        format!("{}m", millis)
    }
}

fn format_memory_bytes(b: i64) -> String {
    const GIB: i64 = 1024 * 1024 * 1024;
    const MIB: i64 = 1024 * 1024;
    const KIB: i64 = 1024;
    if b >= GIB && b % GIB == 0 {
        format!("{}Gi", b / GIB)
    } else if b >= MIB && b % MIB == 0 {
        format!("{}Mi", b / MIB)
    } else if b >= KIB && b % KIB == 0 {
        format!("{}Ki", b / KIB)
    } else {
        format!("{}", b)
    }
}

/// Format CPU millicores as cores for display (e.g. 330 -> "0.33", 1500 -> "1.5").
fn format_cpu_cores(millis: i64) -> String {
    if millis % 1000 == 0 {
        format!("{}", millis / 1000)
    } else {
        format!("{:.2}", millis as f64 / 1000.0)
    }
}

/// Format memory bytes as Gi for display (e.g. 2147483648 -> "2.0Gi").
fn format_memory_gi(bytes: i64) -> String {
    const GIB: i64 = 1024 * 1024 * 1024;
    if bytes >= GIB {
        format!("{:.1}Gi", bytes as f64 / GIB as f64)
    } else {
        format_memory_bytes(bytes)
    }
}

pub struct InspectionRunner {
    client: K8sClient,
}

impl InspectionRunner {
    pub fn new(client: K8sClient) -> Self {
        Self { client }
    }

    pub async fn run_inspections(
        &self,
        inspection_type: InspectionType,
        namespace: Option<&str>,
        node_inspector_namespace: &str,
        cluster_name_override: Option<&str>,
    ) -> Result<ClusterReport> {
        let mut inspections = Vec::new();

        match inspection_type {
            // Logical order: infrastructure → storage & resources → workloads → security & policy → operations
            InspectionType::All => {
                inspections.push(self.run_node_inspection().await?);
                inspections.push(self.run_control_plane_inspection().await?);
                inspections.push(self.run_network_inspection(namespace).await?);
                inspections.push(self.run_storage_inspection(namespace).await?);
                inspections.push(self.run_resource_inspection(namespace).await?);
                inspections.push(self.run_pod_inspection(namespace).await?);
                inspections.push(self.run_autoscaling_inspection(namespace).await?);
                inspections.push(self.run_batch_inspection(namespace).await?);
                inspections.push(self.run_security_inspection(namespace).await?);
                inspections.push(self.run_policy_inspection(namespace).await?);
                inspections.push(self.run_observability_inspection(namespace).await?);
                inspections.push(self.run_namespace_summary_inspection().await?);
                inspections.push(self.run_certificate_inspection().await?);
                inspections.push(self.run_upgrade_readiness_inspection().await?);
            }
            InspectionType::Nodes => {
                inspections.push(self.run_node_inspection().await?);
            }
            InspectionType::Pods => {
                inspections.push(self.run_pod_inspection(namespace).await?);
            }
            InspectionType::Resources => {
                inspections.push(self.run_resource_inspection(namespace).await?);
            }
            InspectionType::Network => {
                inspections.push(self.run_network_inspection(namespace).await?);
            }
            InspectionType::Storage => {
                inspections.push(self.run_storage_inspection(namespace).await?);
            }
            InspectionType::Security => {
                inspections.push(self.run_security_inspection(namespace).await?);
            }
            InspectionType::ControlPlane => {
                inspections.push(self.run_control_plane_inspection().await?);
            }
            InspectionType::Autoscaling => {
                inspections.push(self.run_autoscaling_inspection(namespace).await?);
            }
            InspectionType::Batch => {
                inspections.push(self.run_batch_inspection(namespace).await?);
            }
            InspectionType::Policies => {
                inspections.push(self.run_policy_inspection(namespace).await?);
            }
            InspectionType::Observability => {
                inspections.push(self.run_observability_inspection(namespace).await?);
            }
            InspectionType::Upgrade => {
                inspections.push(self.run_upgrade_readiness_inspection().await?);
            }
            InspectionType::Certificates => {
                inspections.push(self.run_certificate_inspection().await?);
            }
        }

        let mut overall_score = self.calculate_overall_score(&inspections);
        let mut executive_summary = self.generate_executive_summary(&inspections, overall_score);
        let cluster_name = cluster_name_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.client.cluster_name().unwrap_or("default").to_string());

        let cluster_overview = self.fetch_cluster_overview().await.ok();
        let recent_events = self
            .fetch_recent_events(50)
            .await
            .ok()
            .filter(|v| !v.is_empty());

        // Collect per-node inspection JSON from DaemonSet pods when doing full or node-only inspection.
        // DaemonSet is always looked up in node_inspector_namespace (e.g. kubeowler); inspection scope is namespace.
        // Pre-check: if data is stale (>24h), restart DaemonSet; if not deployed, skip with prompt.
        let node_inspection_results: Option<Vec<NodeInspectionResult>> = match inspection_type {
            InspectionType::All | InspectionType::Nodes => {
                let status =
                    ensure_node_inspector_ready(&self.client, node_inspector_namespace, 24).await;
                match status {
                    NodeInspectorStatus::NotDeployed => {
                        println!(
                            "{}  Node inspector DaemonSet not deployed in namespace '{}'. Node inspection skipped.",
                            "ℹ️".bright_blue(),
                            node_inspector_namespace.bright_green()
                        );
                        None
                    }
                    NodeInspectorStatus::RestartedAndReady => {
                        println!(
                            "{}  Node inspector data was stale (>24h). Restarted DaemonSet pods and refreshed.",
                            "⚠️".bright_yellow()
                        );
                        collect_node_inspections(&self.client, Some(node_inspector_namespace))
                            .await
                            .ok()
                    }
                    NodeInspectorStatus::Ready | NodeInspectorStatus::ReadyPartial { .. } => {
                        collect_node_inspections(&self.client, Some(node_inspector_namespace))
                            .await
                            .ok()
                    }
                }
            }
            _ => None,
        };

        // Synthetic Node Inspection result: issues for nodes with zombie processes (NODE-003).
        if let Some(ref nodes) = &node_inspection_results {
            let zombie_issues: Vec<Issue> = nodes
                .iter()
                .filter(|n| n.zombie_count.map(|c| c > 0).unwrap_or(false))
                .map(|n| {
                    let z = n.zombie_count.unwrap_or(0);
                    Issue {
                        severity: IssueSeverity::Warning,
                        category: "Node".to_string(),
                        description: format!("Node {} has {} zombie process(es)", n.node_name, z),
                        resource: Some(n.node_name.clone()),
                        recommendation: "Identify parent processes and fix reaping; see NODE-003."
                            .to_string(),
                        rule_id: Some("NODE-003".to_string()),
                    }
                })
                .collect();
            if !zombie_issues.is_empty() {
                let check = CheckResult {
                    name: "Node process health".to_string(),
                    description: "Zombie processes on nodes".to_string(),
                    status: CheckStatus::Warning,
                    score: 0.0,
                    max_score: 100.0,
                    details: Some(format!(
                        "{} node(s) with zombie processes",
                        zombie_issues.len()
                    )),
                    recommendations: vec![
                        "See NODE-003 and fix parent process reaping.".to_string()
                    ],
                };
                let summary = InspectionSummary {
                    total_checks: 1,
                    passed_checks: 0,
                    warning_checks: zombie_issues.len() as u32,
                    critical_checks: 0,
                    error_checks: 0,
                    issues: zombie_issues,
                };
                inspections.push(InspectionResult {
                    inspection_type: "Node Inspection".to_string(),
                    timestamp: Utc::now(),
                    overall_score: 0.0,
                    checks: vec![check],
                    summary,
                    certificate_expiries: None,
                    pod_container_states: None,
                    namespace_summary_rows: None,
                });
                overall_score = self.calculate_overall_score(&inspections);
                executive_summary = self.generate_executive_summary(&inspections, overall_score);
            }
        }

        let (display_timestamp, display_timestamp_filename) = node_inspection_results
            .as_ref()
            .and_then(|nodes| nodes.first())
            .and_then(|n| n.timestamp_local.as_ref())
            .and_then(|s| {
                chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
                    .ok()
                    .map(|dt| {
                        (
                            dt.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
                            dt.format("%Y-%m-%d-%H%M%S").to_string(),
                        )
                    })
            })
            .map(|(h, f)| (Some(h), Some(f)))
            .unwrap_or((None, None));

        Ok(ClusterReport {
            cluster_name,
            report_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            overall_score,
            inspections,
            executive_summary,
            cluster_overview,
            node_inspection_results,
            recent_events,
            display_timestamp,
            display_timestamp_filename,
        })
    }

    /// Fetch recent cluster events (Warning and Error only; Normal is excluded).
    async fn fetch_recent_events(&self, limit: usize) -> Result<Vec<EventRow>> {
        use k8s_openapi::api::core::v1::Event;
        use kube::Api;

        let ns_api = self.client.namespaces();
        let ns_list = ns_api.list(&ListParams::default()).await?;
        const MAX_NAMESPACES: usize = 20;
        let ns_names: Vec<String> = ns_list
            .items
            .into_iter()
            .filter_map(|n| n.metadata.name)
            .take(MAX_NAMESPACES)
            .collect();

        let mut rows: Vec<EventRow> = Vec::new();
        for ns in &ns_names {
            let events_api: Api<Event> = Api::namespaced(self.client.client().clone(), ns);
            let list_params = ListParams::default();
            let events = match events_api.list(&list_params).await {
                Ok(l) => l,
                Err(_) => continue,
            };
            for ev in events.items {
                let type_ = ev.type_.as_deref().unwrap_or("");
                if type_ != "Warning" && type_ != "Error" {
                    continue;
                }
                let namespace = ev.metadata.namespace.as_deref().unwrap_or("").to_string();
                let obj = &ev.involved_object;
                let kind = obj.kind.as_deref().unwrap_or("").to_string();
                let name = obj.name.as_deref().unwrap_or("").to_string();
                let object_ref = if kind.is_empty() || name.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", kind, name)
                };
                let last_seen = ev
                    .last_timestamp
                    .as_ref()
                    .or(ev.first_timestamp.as_ref())
                    .map(|t| t.0.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());
                let message = ev.message.as_deref().unwrap_or("").to_string();
                let message_trunc = if message.len() > 80 {
                    format!("{}...", &message[..77])
                } else {
                    message
                };
                rows.push(EventRow {
                    namespace,
                    object_ref,
                    event_type: type_.to_string(),
                    reason: ev.reason.as_deref().unwrap_or("").to_string(),
                    message: message_trunc,
                    last_seen,
                });
            }
        }
        rows.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
        rows.truncate(limit);
        Ok(rows)
    }

    /// Build cluster overview from node list (and optional server version). Used for report header.
    async fn fetch_cluster_overview(&self) -> Result<ClusterOverview> {
        let nodes_api = self.client.nodes();
        let nodes = nodes_api.list(&ListParams::default()).await?;
        let pods_api = self.client.pods(None);
        let pods = pods_api.list(&ListParams::default()).await?;
        let mut pods_per_node: HashMap<String, u32> = HashMap::new();
        for pod in &pods.items {
            if let Some(ref name) = pod.spec.as_ref().and_then(|s| s.node_name.as_ref()) {
                *pods_per_node.entry(name.to_string()).or_insert(0) += 1;
            }
        }
        let pod_count = pods.items.len() as u32;

        // Pod phase breakdown from existing pods list.
        let mut pod_phase = PodPhaseBreakdown::default();
        for pod in &pods.items {
            let phase = pod
                .status
                .as_ref()
                .and_then(|s| s.phase.as_deref())
                .unwrap_or("Unknown");
            match phase {
                "Running" => pod_phase.running += 1,
                "Pending" => pod_phase.pending += 1,
                "Succeeded" => pod_phase.succeeded += 1,
                "Failed" => pod_phase.failed += 1,
                _ => pod_phase.unknown += 1,
            }
        }

        // Namespace count.
        let ns_api = self.client.namespaces();
        let ns_list = ns_api.list(&ListParams::default()).await?;
        let namespace_count = ns_list.items.len() as u32;

        // Workload summary: Deployments, StatefulSets, DaemonSets (cluster-wide).
        let mut workload = WorkloadSummary::default();
        let dep_api = self.client.deployments(None);
        if let Ok(list) = dep_api.list(&ListParams::default()).await {
            workload.deployments_total = list.items.len() as u32;
            for d in &list.items {
                let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1) as u32;
                let ready = d
                    .status
                    .as_ref()
                    .and_then(|s| s.ready_replicas)
                    .unwrap_or(0) as u32;
                if desired > 0 && ready >= desired {
                    workload.deployments_ready += 1;
                }
            }
        }
        let sts_api = self.client.stateful_sets(None);
        if let Ok(list) = sts_api.list(&ListParams::default()).await {
            workload.statefulsets_total = list.items.len() as u32;
            for s in &list.items {
                let desired = s.spec.as_ref().and_then(|sp| sp.replicas).unwrap_or(1) as u32;
                let ready = s
                    .status
                    .as_ref()
                    .and_then(|st| st.ready_replicas)
                    .unwrap_or(0) as u32;
                if desired > 0 && ready >= desired {
                    workload.statefulsets_ready += 1;
                }
            }
        }
        let ds_api = self.client.daemon_sets(None);
        if let Ok(list) = ds_api.list(&ListParams::default()).await {
            workload.daemonsets_total = list.items.len() as u32;
            for d in &list.items {
                let desired = d
                    .status
                    .as_ref()
                    .map(|s| s.desired_number_scheduled)
                    .unwrap_or(0) as u32;
                let ready = d.status.as_ref().map(|s| s.number_ready).unwrap_or(0) as u32;
                if desired > 0 && ready >= desired {
                    workload.daemonsets_ready += 1;
                }
            }
        }

        // Storage summary: PV, PVC (all ns), StorageClass.
        let mut storage = StorageSummary::default();
        let pv_api = self.client.persistent_volumes();
        if let Ok(list) = pv_api.list(&ListParams::default()).await {
            storage.pv_total = list.items.len() as u32;
        }
        let pvc_api = self.client.persistent_volume_claims(None);
        if let Ok(list) = pvc_api.list(&ListParams::default()).await {
            storage.pvc_total = list.items.len() as u32;
            for pvc in &list.items {
                let phase = pvc
                    .status
                    .as_ref()
                    .and_then(|s| s.phase.as_deref())
                    .unwrap_or("");
                if phase == "Bound" {
                    storage.pvc_bound += 1;
                }
            }
        }
        let sc_api = self.client.storage_classes();
        if let Ok(list) = sc_api.list(&ListParams::default()).await {
            storage.storage_class_count = list.items.len() as u32;
            storage.has_default_storage_class = list.items.iter().any(|sc| {
                sc.metadata
                    .annotations
                    .as_ref()
                    .and_then(|a| a.get("storageclass.kubernetes.io/is-default-class"))
                    .map(|v| v.as_str())
                    == Some("true")
            });
        }

        let total = nodes.items.len() as u32;
        let mut ready = 0u32;
        let mut os_arch: HashMap<(String, String), u32> = HashMap::new();
        let mut kubelet_versions: Vec<String> = Vec::new();
        let mut cap_cpu_millis: i64 = 0;
        let mut cap_mem_bytes: i64 = 0;
        let mut alloc_cpu_millis: i64 = 0;
        let mut alloc_mem_bytes: i64 = 0;
        let mut node_list: Vec<NodeRow> = Vec::new();
        let mut node_conditions: Vec<NodeConditionsRow> = Vec::new();
        let mut allocatable_per_node: HashMap<String, (i64, i64, i64)> = HashMap::new();

        const CONDITION_TYPES: &[&str] =
            &["Ready", "MemoryPressure", "DiskPressure", "PIDPressure"];

        for node in &nodes.items {
            let name = node.metadata.name.as_deref().unwrap_or("").to_string();
            let mut os = "Unknown".to_string();
            let mut arch = "unknown".to_string();
            let mut kubelet_version = String::new();
            let mut os_image: Option<String> = None;
            let mut kernel_version: Option<String> = None;
            let mut container_runtime_version: Option<String> = None;
            let mut is_ready = false;
            let mut cond_map: HashMap<String, String> = CONDITION_TYPES
                .iter()
                .map(|&t| (t.to_string(), "Unknown".to_string()))
                .collect();

            if let Some(status) = &node.status {
                if let Some(conditions) = &status.conditions {
                    for c in conditions {
                        if CONDITION_TYPES.contains(&c.type_.as_str()) {
                            cond_map.insert(c.type_.clone(), c.status.clone());
                        }
                        if c.type_ == "Ready" && c.status == "True" {
                            ready += 1;
                            is_ready = true;
                        }
                    }
                }
                if let Some(ref info) = status.node_info {
                    os = info.operating_system.clone();
                    arch = info.architecture.clone();
                    kubelet_version = info.kubelet_version.clone();
                    if !info.os_image.is_empty() {
                        os_image = Some(info.os_image.clone());
                    }
                    if !info.kernel_version.is_empty() {
                        kernel_version = Some(info.kernel_version.clone());
                    }
                    if !info.container_runtime_version.is_empty() {
                        container_runtime_version = Some(info.container_runtime_version.clone());
                    }
                    if !kubelet_version.is_empty() {
                        kubelet_versions.push(kubelet_version.clone());
                    }
                    *os_arch.entry((os.clone(), arch.clone())).or_insert(0) += 1;
                }
                if let (Some(cap), Some(alloc)) = (&status.capacity, &status.allocatable) {
                    let ac = parse_cpu_quantity(alloc.get("cpu")).unwrap_or(0);
                    let am = parse_memory_quantity(alloc.get("memory")).unwrap_or(0);
                    let disk_bytes =
                        parse_memory_quantity(alloc.get("ephemeral-storage")).unwrap_or(0);
                    allocatable_per_node.insert(name.clone(), (ac, am, disk_bytes));
                    cap_cpu_millis += parse_cpu_quantity(cap.get("cpu")).unwrap_or(0);
                    cap_mem_bytes += parse_memory_quantity(cap.get("memory")).unwrap_or(0);
                    alloc_cpu_millis += parse_cpu_quantity(alloc.get("cpu")).unwrap_or(0);
                    alloc_mem_bytes += parse_memory_quantity(alloc.get("memory")).unwrap_or(0);
                }
            }

            let node_pod_count = pods_per_node.get(&name).copied().unwrap_or(0);
            let node_address = node
                .status
                .as_ref()
                .and_then(|s| s.addresses.as_ref())
                .and_then(|addrs| {
                    addrs
                        .iter()
                        .find(|a| a.type_.as_str() == "InternalIP")
                        .map(|a| a.address.clone())
                });
            node_list.push(NodeRow {
                name: name.clone(),
                operating_system: os,
                architecture: arch,
                kubelet_version,
                ready: is_ready,
                pod_count: node_pod_count,
                node_address,
                os_image,
                kernel_version,
                container_runtime_version,
            });
            node_conditions.push(NodeConditionsRow {
                node_name: name,
                ready: cond_map
                    .get("Ready")
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                memory_pressure: cond_map
                    .get("MemoryPressure")
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                disk_pressure: cond_map
                    .get("DiskPressure")
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                pid_pressure: cond_map
                    .get("PIDPressure")
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
            });
        }

        kubelet_versions.sort();
        kubelet_versions.dedup();

        let node_summary = if os_arch.is_empty() {
            None
        } else {
            let parts: Vec<String> = os_arch
                .into_iter()
                .map(|((os, arch), count)| format!("{} {} node(s) {}", os, count, arch))
                .collect();
            let mut summary = parts.join(", ");
            if let Some(kv) = kubelet_versions.first() {
                if kubelet_versions.len() == 1 {
                    summary.push_str(&format!(", kubelet {}", kv));
                } else {
                    summary.push_str(&format!(
                        ", kubelet {}..{}",
                        kv,
                        kubelet_versions.last().unwrap_or(&String::new())
                    ));
                }
            }
            Some(summary)
        };

        const GIB_BYTES: f64 = 1024.0 * 1024.0 * 1024.0;
        let alloc_disk_gi = allocatable_per_node
            .values()
            .map(|&(_c, _m, disk_bytes)| disk_bytes as f64 / GIB_BYTES)
            .sum::<f64>();
        let allocatable_disk_gi = if alloc_disk_gi > 0.0 {
            Some(alloc_disk_gi)
        } else {
            None
        };

        let node_resources = if total > 0 && (cap_cpu_millis > 0 || cap_mem_bytes > 0) {
            Some(NodeResourceSummary {
                capacity_cpu: format_cpu_millis(cap_cpu_millis),
                capacity_memory: format_memory_bytes(cap_mem_bytes),
                allocatable_cpu: format_cpu_millis(alloc_cpu_millis),
                allocatable_memory: format_memory_bytes(alloc_mem_bytes),
                allocatable_disk_gi,
            })
        } else {
            None
        };

        let cluster_version = self.client.server_version().await.ok().flatten();

        let cluster_age_days: Option<u64> = nodes
            .items
            .iter()
            .filter_map(|n| n.metadata.creation_timestamp.as_ref())
            .min()
            .map(|t| {
                let now = Utc::now();
                let creation = t.0;
                (now.signed_duration_since(creation).num_days()).max(0) as u64
            });

        let (metrics_available, node_usage, total_usage_cpu_cores, total_usage_memory_gi) =
            match self.client.node_metrics().await.ok().flatten() {
                Some(metrics) => {
                    let mut rows: Vec<NodeUsageRow> = Vec::new();
                    let mut sum_cpu_millis: i64 = 0;
                    let mut sum_mem_bytes: i64 = 0;
                    for (node_name, cpu_str, mem_str) in metrics {
                        let cpu_millis = parse_cpu_str(&cpu_str).unwrap_or(0);
                        let mem_bytes = parse_memory_str(&mem_str).unwrap_or(0);
                        sum_cpu_millis += cpu_millis;
                        sum_mem_bytes += mem_bytes;
                        let (
                            alloc_cpu_cores,
                            alloc_mem_gi,
                            disk_allocatable_gi,
                            cpu_pct,
                            memory_pct,
                        ) = allocatable_per_node
                            .get(&node_name)
                            .map(|&(alloc_cpu, alloc_mem, disk_bytes)| {
                                let cpu_pct = if alloc_cpu > 0 {
                                    Some((cpu_millis as f64 / alloc_cpu as f64) * 100.0)
                                } else {
                                    None
                                };
                                let memory_pct = if alloc_mem > 0 {
                                    Some((mem_bytes as f64 / alloc_mem as f64) * 100.0)
                                } else {
                                    None
                                };
                                let disk_gi = if disk_bytes > 0 {
                                    Some(disk_bytes as f64 / GIB_BYTES)
                                } else {
                                    None
                                };
                                let cpu_cores = Some(alloc_cpu as f64 / 1000.0);
                                let mem_gi = Some(alloc_mem as f64 / GIB_BYTES);
                                (cpu_cores, mem_gi, disk_gi, cpu_pct, memory_pct)
                            })
                            .unwrap_or((None, None, None, None, None));
                        rows.push(NodeUsageRow {
                            node_name: node_name.clone(),
                            allocatable_cpu_cores: alloc_cpu_cores,
                            cpu_usage: format_cpu_cores(cpu_millis),
                            cpu_pct,
                            allocatable_memory_gi: alloc_mem_gi,
                            memory_usage: format_memory_gi(mem_bytes),
                            memory_pct,
                            disk_allocatable_gi,
                            disk_usage_gi: None,
                            disk_pct: None,
                        });
                    }
                    let total_cpu = if rows.is_empty() {
                        None
                    } else {
                        Some(sum_cpu_millis as f64 / 1000.0)
                    };
                    let total_mem = if rows.is_empty() {
                        None
                    } else {
                        Some(sum_mem_bytes as f64 / GIB_BYTES)
                    };
                    (
                        Some(true),
                        if rows.is_empty() { None } else { Some(rows) },
                        total_cpu,
                        total_mem,
                    )
                }
                None => (Some(false), None, None, None),
            };

        /// Top N containers by high usage (usage/limit >= 80%); only these are shown in the report.
        const CONTAINER_HIGH_USAGE_TOP_N: usize = 20;
        const HIGH_USAGE_PCT: f64 = 0.80;

        let container_usage_notable: Option<Vec<ContainerUsageRow>> = if metrics_available
            != Some(true)
        {
            None
        } else {
            match self.client.pod_metrics().await.ok().flatten() {
                None => None,
                Some(metrics_list) => {
                    let pod_lookup: HashMap<(String, String), &Pod> = pods
                        .items
                        .iter()
                        .filter_map(|p| {
                            let ns = p.metadata.namespace.as_deref().unwrap_or("").to_string();
                            let name = p.metadata.name.as_deref().unwrap_or("").to_string();
                            if name.is_empty() {
                                None
                            } else {
                                Some(((ns, name), p))
                            }
                        })
                        .collect();
                    let mut high_usage_rows: Vec<(f64, ContainerUsageRow)> = Vec::new();
                    for (ns, pod_name, container_name, cpu_str, mem_str) in metrics_list {
                        let cpu_used_m = parse_cpu_str(&cpu_str).unwrap_or(0).max(0) as u64;
                        let mem_used_bytes = parse_memory_str(&mem_str).unwrap_or(0).max(0);
                        let mem_used_mib = (mem_used_bytes / (1024 * 1024)) as u64;

                        let pod = match pod_lookup.get(&(ns.clone(), pod_name.clone())) {
                            Some(p) => p,
                            None => continue,
                        };
                        let spec = match &pod.spec {
                            Some(s) => s,
                            None => continue,
                        };
                        let container = spec.containers.iter().find(|c| c.name == container_name);
                        let container = match container {
                            Some(c) => c,
                            None => continue,
                        };

                        let lim = container.resources.as_ref().and_then(|r| r.limits.as_ref());
                        let cpu_request_m = container
                            .resources
                            .as_ref()
                            .and_then(|r| r.requests.as_ref())
                            .and_then(|r| r.get("cpu"))
                            .and_then(|q| parse_cpu_str(q.0.as_str()))
                            .unwrap_or(0)
                            .max(0) as u64;
                        let mem_request_bytes = container
                            .resources
                            .as_ref()
                            .and_then(|r| r.requests.as_ref())
                            .and_then(|r| r.get("memory"))
                            .and_then(|q| parse_memory_str(q.0.as_str()))
                            .unwrap_or(0)
                            .max(0);
                        let mem_request_mib = (mem_request_bytes / (1024 * 1024)) as u64;
                        let cpu_limit_m = lim
                            .and_then(|r| r.get("cpu"))
                            .and_then(|q| parse_cpu_str(q.0.as_str()))
                            .unwrap_or(0)
                            .max(0) as u64;
                        let mem_limit_bytes = lim
                            .and_then(|r| r.get("memory"))
                            .and_then(|q| parse_memory_str(q.0.as_str()))
                            .unwrap_or(0)
                            .max(0);
                        let mem_limit_mib = (mem_limit_bytes / (1024 * 1024)) as u64;

                        let cpu_pct = if cpu_limit_m > 0 {
                            cpu_used_m as f64 / cpu_limit_m as f64
                        } else {
                            0.0
                        };
                        let mem_pct = if mem_limit_mib > 0 {
                            mem_used_mib as f64 / mem_limit_mib as f64
                        } else {
                            0.0
                        };
                        let high_usage = cpu_pct >= HIGH_USAGE_PCT || mem_pct >= HIGH_USAGE_PCT;
                        if !high_usage {
                            continue;
                        }
                        let sort_score = cpu_pct.max(mem_pct);
                        high_usage_rows.push((
                            sort_score,
                            ContainerUsageRow {
                                namespace: ns,
                                pod_name,
                                container_name,
                                cpu_used_m,
                                cpu_request_m,
                                cpu_limit_m,
                                mem_used_mib,
                                mem_request_mib,
                                mem_limit_mib,
                                notable_reason: "high_usage".to_string(),
                            },
                        ));
                    }
                    high_usage_rows
                        .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                    let rows: Vec<ContainerUsageRow> = high_usage_rows
                        .into_iter()
                        .map(|(_, r)| r)
                        .take(CONTAINER_HIGH_USAGE_TOP_N)
                        .collect();
                    if rows.is_empty() {
                        None
                    } else {
                        Some(rows)
                    }
                }
            }
        };

        Ok(ClusterOverview {
            cluster_version,
            node_count: total,
            ready_node_count: ready,
            pod_count: Some(pod_count),
            node_summary,
            node_resources,
            node_list: if node_list.is_empty() {
                None
            } else {
                Some(node_list)
            },
            metrics_available,
            node_usage,
            total_usage_cpu_cores,
            total_usage_memory_gi,
            node_conditions: if node_conditions.is_empty() {
                None
            } else {
                Some(node_conditions)
            },
            pod_phase_breakdown: Some(pod_phase),
            namespace_count: Some(namespace_count),
            workload_summary: Some(workload),
            storage_summary: Some(storage),
            cluster_age_days,
            container_usage_notable,
        })
    }

    async fn run_node_inspection(&self) -> Result<InspectionResult> {
        nodes::NodeInspector::new(&self.client).inspect().await
    }

    async fn run_pod_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        pods::PodInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_resource_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        resources::ResourceInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_network_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        network::NetworkInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_storage_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        storage::StorageInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_security_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        security::SecurityInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_control_plane_inspection(&self) -> Result<InspectionResult> {
        control_plane::ControlPlaneInspector::new(&self.client)
            .inspect()
            .await
    }

    async fn run_autoscaling_inspection(
        &self,
        namespace: Option<&str>,
    ) -> Result<InspectionResult> {
        autoscaling::AutoscalingInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_batch_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        batch::BatchInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_policy_inspection(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        policies::PoliciesInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_observability_inspection(
        &self,
        namespace: Option<&str>,
    ) -> Result<InspectionResult> {
        observability::ObservabilityInspector::new(&self.client)
            .inspect(namespace)
            .await
    }

    async fn run_namespace_summary_inspection(&self) -> Result<InspectionResult> {
        namespace_summary::NamespaceSummaryInspector::new(&self.client)
            .inspect()
            .await
    }

    async fn run_upgrade_readiness_inspection(&self) -> Result<InspectionResult> {
        upgrade::UpgradeInspector::new(&self.client).inspect().await
    }

    async fn run_certificate_inspection(&self) -> Result<InspectionResult> {
        certificates::CertificateInspector::new(&self.client)
            .inspect()
            .await
    }

    fn calculate_overall_score(&self, inspections: &[InspectionResult]) -> f64 {
        if inspections.is_empty() {
            return 0.0;
        }

        let total_score: f64 = inspections.iter().map(|i| i.overall_score).sum();
        total_score / inspections.len() as f64
    }

    fn generate_executive_summary(
        &self,
        inspections: &[InspectionResult],
        overall_score: f64,
    ) -> ExecutiveSummary {
        let health_status = match overall_score {
            s if s >= 90.0 => HealthStatus::Excellent,
            s if s >= 80.0 => HealthStatus::Good,
            s if s >= 70.0 => HealthStatus::Fair,
            s if s >= 60.0 => HealthStatus::Poor,
            _ => HealthStatus::Critical,
        };

        let mut key_findings = Vec::new();
        let mut priority_recommendations = Vec::new();
        let mut score_breakdown = HashMap::new();

        for inspection in inspections {
            score_breakdown.insert(inspection.inspection_type.clone(), inspection.overall_score);

            for issue in &inspection.summary.issues {
                if matches!(issue.severity, IssueSeverity::Critical) {
                    key_findings.push(issue.description.clone());
                    priority_recommendations.push(issue.recommendation.clone());
                }
            }
        }

        key_findings.sort();
        key_findings.dedup();
        priority_recommendations.sort();
        priority_recommendations.dedup();

        key_findings.truncate(5);
        priority_recommendations.truncate(5);

        ExecutiveSummary {
            health_status,
            key_findings,
            priority_recommendations,
            score_breakdown,
        }
    }
}
