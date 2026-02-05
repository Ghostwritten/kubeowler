use anyhow::Result;
use chrono::Utc;
use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;

use crate::inspections::types::*;
use crate::k8s::K8sClient;

const METRICS_SERVER_IDENTIFIERS: [&str; 2] = ["metrics-server", "metricsserver"];
const KUBE_STATE_METRICS_IDENTIFIERS: [&str; 2] = ["kube-state-metrics", "kube_state_metrics"];
const COREDNS_IDENTIFIERS: [&str; 2] = ["coredns", "kube-dns"];
const PROMETHEUS_IDENTIFIERS: [&str; 3] = ["prometheus", "thanos", "victoriametrics"];
const LOGGING_IDENTIFIERS: [&str; 4] = ["fluent", "logstash", "loki", "vector"];

pub struct ObservabilityInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> ObservabilityInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let metrics_check = self.inspect_metrics_components(&mut issues).await?;
        let coredns_check = self.inspect_coredns(&mut issues).await?;
        let logging_check = self
            .inspect_logging_components(namespace, &mut issues)
            .await?;
        let alerting_check = self
            .inspect_alerting_components(namespace, &mut issues)
            .await?;

        checks.push(metrics_check);
        checks.push(coredns_check);
        checks.push(logging_check);
        checks.push(alerting_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Observability".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_metrics_components(&self, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        // metrics-server: typically in kube-system
        let pods_api = self.client.pods(Some("kube-system"));
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut metrics_server_found = false;
        let mut kube_state_metrics_found = false;

        for pod in &pods.items {
            if let Some(name) = pod.metadata.name.as_deref() {
                if METRICS_SERVER_IDENTIFIERS
                    .iter()
                    .any(|id| name.contains(id))
                    && is_pod_ready(pod)
                {
                    metrics_server_found = true;
                }
                if KUBE_STATE_METRICS_IDENTIFIERS
                    .iter()
                    .any(|id| name.contains(id))
                    && is_pod_ready(pod)
                {
                    kube_state_metrics_found = true;
                }
            }
        }

        // kube-state-metrics may run in prometheus or monitoring namespace
        if !kube_state_metrics_found {
            for ns in &["prometheus", "monitoring"] {
                let api = self.client.pods(Some(ns));
                if let Ok(list) = api.list(&ListParams::default()).await {
                    for pod in &list.items {
                        if let Some(name) = pod.metadata.name.as_deref() {
                            if KUBE_STATE_METRICS_IDENTIFIERS
                                .iter()
                                .any(|id| name.contains(id))
                                && is_pod_ready(pod)
                            {
                                kube_state_metrics_found = true;
                                break;
                            }
                        }
                    }
                }
                if kube_state_metrics_found {
                    break;
                }
            }
        }

        let mut score: f64 = 100.0;
        let mut recommendations = Vec::new();

        if !metrics_server_found {
            score -= 30.0;
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                category: "Observability".to_string(),
                description: "metrics-server is missing or not ready".to_string(),
                resource: Some("kube-system".to_string()),
                recommendation: "Deploy metrics-server to enable HPA and kubectl top commands."
                    .to_string(),
                rule_id: Some("OBS-001".to_string()),
            });
            recommendations.push("Install metrics-server for core metrics APIs.".to_string());
        }

        if !kube_state_metrics_found {
            score -= 20.0;
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Observability".to_string(),
                description: "kube-state-metrics is missing or not ready".to_string(),
                resource: Some("kube-system".to_string()),
                recommendation: "Deploy kube-state-metrics to expose Kubernetes object metrics."
                    .to_string(),
                rule_id: Some("OBS-002".to_string()),
            });
            recommendations.push("Install kube-state-metrics for Prometheus scraping.".to_string());
        }

        let status = if score >= 90.0 {
            CheckStatus::Pass
        } else if score >= 60.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "Metrics Pipeline".to_string(),
            description: "Checks metrics-server and kube-state-metrics availability".to_string(),
            status,
            score: score.max(0.0),
            max_score: 100.0,
            details: Some(format!(
                "metrics-server: {}, kube-state-metrics: {}",
                if metrics_server_found {
                    "present"
                } else {
                    "missing"
                },
                if kube_state_metrics_found {
                    "present"
                } else {
                    "missing"
                }
            )),
            recommendations,
        })
    }

    async fn inspect_coredns(&self, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let pods_api = self.client.pods(Some("kube-system"));
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut ready = 0u32;
        let mut total = 0u32;
        for pod in &pods.items {
            if let Some(name) = pod.metadata.name.as_deref() {
                if COREDNS_IDENTIFIERS.iter().any(|id| name.contains(id)) {
                    total += 1;
                    if is_pod_ready(pod) {
                        ready += 1;
                    }
                }
            }
        }

        let (status, score, details) = if total == 0 {
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                category: "Observability".to_string(),
                description: "CoreDNS (cluster DNS) not found in kube-system".to_string(),
                resource: Some("kube-system".to_string()),
                recommendation: "Ensure CoreDNS or kube-dns is deployed for cluster DNS."
                    .to_string(),
                rule_id: Some("OBS-003".to_string()),
            });
            (CheckStatus::Critical, 0.0, "CoreDNS: not found".to_string())
        } else if ready < total {
            (
                CheckStatus::Warning,
                (ready as f64 / total as f64) * 100.0,
                format!("CoreDNS: {}/{} ready", ready, total),
            )
        } else {
            (
                CheckStatus::Pass,
                100.0,
                format!("CoreDNS: {}/{} ready", ready, total),
            )
        };

        Ok(CheckResult {
            name: "Cluster DNS (CoreDNS)".to_string(),
            description: "Checks CoreDNS/kube-dns availability in kube-system".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(details),
            recommendations: vec![],
        })
    }

    async fn inspect_logging_components(
        &self,
        namespace: Option<&str>,
        issues: &mut Vec<Issue>,
    ) -> Result<CheckResult> {
        let target_ns = namespace.unwrap_or("kube-system");
        let pods_api = self.client.pods(Some(target_ns));
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut logging_found = false;
        for pod in &pods.items {
            if let Some(name) = pod.metadata.name.as_deref() {
                if LOGGING_IDENTIFIERS.iter().any(|id| name.contains(id)) && is_pod_ready(pod) {
                    logging_found = true;
                    break;
                }
            }
        }

        if logging_found {
            Ok(CheckResult {
                name: "Logging Stack".to_string(),
                description: "Checks whether logging collectors are running".to_string(),
                status: CheckStatus::Pass,
                score: 100.0,
                max_score: 100.0,
                details: Some(format!(
                    "Logging components detected in namespace {}",
                    target_ns
                )),
                recommendations: vec![],
            })
        } else {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Observability".to_string(),
                description: "No logging collector pods detected".to_string(),
                resource: Some(target_ns.to_string()),
                recommendation: "Deploy Fluentd/Vector/Logstash to aggregate cluster logs."
                    .to_string(),
                rule_id: Some("OBS-003".to_string()),
            });
            Ok(CheckResult {
                name: "Logging Stack".to_string(),
                description: "Checks whether logging collectors are running".to_string(),
                status: CheckStatus::Warning,
                score: 70.0,
                max_score: 100.0,
                details: Some("No logging stack found".to_string()),
                recommendations: vec![
                    "Install a logging stack (e.g., Fluent Bit + Loki).".to_string()
                ],
            })
        }
    }

    async fn inspect_alerting_components(
        &self,
        namespace: Option<&str>,
        issues: &mut Vec<Issue>,
    ) -> Result<CheckResult> {
        let potential_namespaces = [
            namespace.unwrap_or("monitoring"),
            "prometheus",
            "observability",
            "kube-system",
        ];

        let mut prometheus_found = false;
        for ns in &potential_namespaces {
            let pods_api = self.client.pods(Some(ns));
            if let Ok(pods) = pods_api.list(&ListParams::default()).await {
                for pod in pods.items {
                    if let Some(name) = pod.metadata.name.as_deref() {
                        if PROMETHEUS_IDENTIFIERS.iter().any(|id| name.contains(id))
                            && is_pod_ready(&pod)
                        {
                            prometheus_found = true;
                            break;
                        }
                    }
                }
            }
            if prometheus_found {
                break;
            }
        }

        if prometheus_found {
            Ok(CheckResult {
                name: "Monitoring & Alerting".to_string(),
                description: "Checks for Prometheus/Thanos/VictoriaMetrics components".to_string(),
                status: CheckStatus::Pass,
                score: 100.0,
                max_score: 100.0,
                details: Some("Prometheus-compatible component detected".to_string()),
                recommendations: vec![],
            })
        } else {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Observability".to_string(),
                description: "No Prometheus-compatible monitoring found".to_string(),
                resource: Some("monitoring".to_string()),
                recommendation: "Deploy Prometheus/Thanos or integrate with managed monitoring."
                    .to_string(),
                rule_id: Some("OBS-004".to_string()),
            });
            Ok(CheckResult {
                name: "Monitoring & Alerting".to_string(),
                description: "Checks for monitoring stacks".to_string(),
                status: CheckStatus::Warning,
                score: 65.0,
                max_score: 100.0,
                details: Some("No Prometheus/Thanos/VictoriaMetrics detected".to_string()),
                recommendations: vec![
                    "Install Prometheus and Alertmanager for proactive monitoring.".to_string(),
                ],
            })
        }
    }

    fn build_summary(&self, checks: &[CheckResult], issues: Vec<Issue>) -> InspectionSummary {
        let total_checks = checks.len() as u32;
        let mut passed_checks = 0;
        let mut warning_checks = 0;
        let mut critical_checks = 0;
        let mut error_checks = 0;

        for check in checks {
            match check.status {
                CheckStatus::Pass => passed_checks += 1,
                CheckStatus::Warning => warning_checks += 1,
                CheckStatus::Critical => critical_checks += 1,
                CheckStatus::Error => error_checks += 1,
            }
        }

        InspectionSummary {
            total_checks,
            passed_checks,
            warning_checks,
            critical_checks,
            error_checks,
            issues,
        }
    }
}

fn is_pod_ready(pod: &Pod) -> bool {
    if let Some(status) = &pod.status {
        if status.phase.as_deref() == Some("Running") {
            if let Some(container_statuses) = &status.container_statuses {
                return container_statuses.iter().all(|c| c.ready);
            }
            return true;
        }
    }
    false
}
