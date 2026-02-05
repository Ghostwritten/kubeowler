use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use log::info;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

/// Map container state reason to issue code (POD-004..POD-011 after renumbering; no POD-004 for "no limits", see RES-002).
fn container_state_reason_to_rule_id(state_kind: &str, reason: &str) -> &'static str {
    if state_kind == "waiting" {
        match reason {
            "ImagePullBackOff" => "POD-005",
            "ErrImagePull" => "POD-006",
            "CrashLoopBackOff" => "POD-007",
            "ContainerCreating" => "POD-008",
            "CreateContainerConfigError" => "POD-009",
            _ => "POD-004",
        }
    } else {
        match reason {
            "OOMKilled" => "POD-010",
            _ => "POD-011",
        }
    }
}

pub struct PodInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> PodInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        info!("Starting Pod status inspection");

        let pods_api = self.client.pods(namespace);
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let total_pods = pods.items.len();
        let mut running_pods = 0;
        let mut failed_pods = 0;
        let mut pending_pods = 0;
        let mut pods_with_restarts = 0;
        let mut reason_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut pod_container_states: Vec<PodContainerStateRow> = Vec::new();

        for pod in &pods.items {
            let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
            let pod_namespace = pod.metadata.namespace.as_deref().unwrap_or("default");

            if let Some(status) = &pod.status {
                // Check pod phase
                match status.phase.as_deref() {
                    Some("Running") => {
                        running_pods += 1;
                        // Running but not Ready (e.g. 0/1 Ready, readiness probe failing)
                        if let Some(conditions) = &status.conditions {
                            for condition in conditions {
                                if condition.type_ == "Ready" && condition.status == "False" {
                                    let reason = condition
                                        .reason
                                        .as_deref()
                                        .unwrap_or("NotReady")
                                        .to_string();
                                    let message = condition
                                        .message
                                        .as_deref()
                                        .unwrap_or("")
                                        .to_string();
                                    let desc = if message.is_empty() {
                                        format!(
                                            "Pod {}/{} is Running but not Ready ({})",
                                            pod_namespace, pod_name, reason
                                        )
                                    } else {
                                        format!(
                                            "Pod {}/{} is Running but not Ready ({}): {}",
                                            pod_namespace, pod_name, reason, message
                                        )
                                    };
                                    issues.push(Issue {
                                        severity: IssueSeverity::Critical,
                                        category: "Pod".to_string(),
                                        description: desc,
                                        resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                        recommendation: "Check readiness probes, container logs, and pod events (e.g. kubectl describe pod)".to_string(),
                                        rule_id: Some("POD-012".to_string()),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                    Some("Failed") => {
                        failed_pods += 1;
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "Pod".to_string(),
                            description: format!("Pod {}/{} is in Failed state", pod_namespace, pod_name),
                            resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                            recommendation: "Check pod logs and events".to_string(),
                            rule_id: Some("POD-001".to_string()),
                        });
                    }
                    Some("Pending") => {
                        pending_pods += 1;
                        if let Some(conditions) = &status.conditions {
                            for condition in conditions {
                                if condition.type_ == "PodScheduled" && condition.status == "False" {
                                    issues.push(Issue {
                                        severity: IssueSeverity::Warning,
                                        category: "Pod".to_string(),
                                        description: format!("Pod {}/{} cannot be scheduled", pod_namespace, pod_name),
                                        resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                        recommendation: "Check resource requests and node capacity".to_string(),
                                        rule_id: Some("POD-002".to_string()),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Collect all container statuses (init + main)
                let all_container_statuses: Vec<_> = status
                    .init_container_statuses
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .chain(status.container_statuses.as_deref().unwrap_or(&[]).iter())
                    .collect();

                // Check container state: waiting (e.g. ImagePullBackOff, CrashLoopBackOff) and terminated (non-zero exit)
                for container_status in &all_container_statuses {
                    if let Some(state) = &container_status.state {
                        if let Some(waiting) = &state.waiting {
                            let reason = waiting
                                .reason
                                .as_deref()
                                .unwrap_or("Waiting")
                                .to_string();
                            *reason_counts.entry(reason.clone()).or_insert(0) += 1;
                            let message = waiting.message.as_deref().unwrap_or("").to_string();
                            pod_container_states.push(PodContainerStateRow {
                                pod_ref: format!("{}/{}", pod_namespace, pod_name),
                                container_name: container_status.name.clone(),
                                state_kind: "waiting".to_string(),
                                reason: reason.clone(),
                                detail: message.clone(),
                            });
                            let desc = if message.is_empty() {
                                format!(
                                    "Pod {}/{} has container {} in state {}",
                                    pod_namespace, pod_name, container_status.name, reason
                                )
                            } else {
                                format!(
                                    "Pod {}/{} has container {} in state {}: {}",
                                    pod_namespace, pod_name, container_status.name, reason, message
                                )
                            };
                            let rule_id = container_state_reason_to_rule_id("waiting", &reason);
                            issues.push(Issue {
                                severity: IssueSeverity::Critical,
                                category: "Container".to_string(),
                                description: desc,
                                resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                recommendation: "Check image, pull secrets, and pod events (e.g. kubectl describe pod)".to_string(),
                                rule_id: Some(rule_id.to_string()),
                            });
                        }
                        if let Some(terminated) = &state.terminated {
                            if terminated.exit_code != 0 {
                                let reason = terminated
                                    .reason
                                    .as_deref()
                                    .unwrap_or("Terminated")
                                    .to_string();
                                *reason_counts.entry(reason.clone()).or_insert(0) += 1;
                                let detail = format!("exit_code={}", terminated.exit_code);
                                pod_container_states.push(PodContainerStateRow {
                                    pod_ref: format!("{}/{}", pod_namespace, pod_name),
                                    container_name: container_status.name.clone(),
                                    state_kind: "terminated".to_string(),
                                    reason: reason.clone(),
                                    detail,
                                });
                                let rule_id = container_state_reason_to_rule_id("terminated", &reason);
                                let desc = format!(
                                    "Pod {}/{} container {} terminated: reason={}, exit_code={}",
                                    pod_namespace,
                                    pod_name,
                                    container_status.name,
                                    reason,
                                    terminated.exit_code
                                );
                                issues.push(Issue {
                                    severity: IssueSeverity::Critical,
                                    category: "Container".to_string(),
                                    description: desc,
                                    resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                    recommendation: "Check container logs and events".to_string(),
                                    rule_id: Some(rule_id.to_string()),
                                });
                            }
                        }
                    }
                }

                // Check container statuses and restart counts: 0 → no issue; 1–3 → Info; 4–10 → Warning; >10 → Critical.
                // Pod Stability score: count pods that have at least one container with restart_count > 3.
                let mut pod_has_excessive_restarts = false;
                for container_status in &all_container_statuses {
                    let r = container_status.restart_count;
                    if r > 3 {
                        pod_has_excessive_restarts = true;
                    }
                    if r == 0 {
                        continue;
                    }
                    let severity = if r <= 3 {
                        IssueSeverity::Info
                    } else if r <= 10 {
                        IssueSeverity::Warning
                    } else {
                        IssueSeverity::Critical
                    };
                    issues.push(Issue {
                        severity,
                        category: "Container".to_string(),
                        description: format!(
                            "Container {} in pod {}/{} has {} restarts",
                            container_status.name, pod_namespace, pod_name, r
                        ),
                        resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                        recommendation: "Investigate container crashes and resource limits".to_string(),
                        rule_id: Some("POD-003".to_string()),
                    });
                }
                if pod_has_excessive_restarts {
                    pods_with_restarts += 1;
                }
            }
        }

        // Pod health check
        let health_score = if total_pods > 0 {
            (running_pods as f64 / total_pods as f64) * 100.0
        } else {
            100.0
        };

        let health_details = if reason_counts.is_empty() {
            format!(
                "Running: {}, Failed: {}, Pending: {}, Total: {}",
                running_pods, failed_pods, pending_pods, total_pods
            )
        } else {
            let reason_summary: Vec<String> = reason_counts
                .iter()
                .map(|(r, c)| format!("{} {}", c, r))
                .collect();
            format!(
                "Running: {}, Failed: {}, Pending: {}, Total: {}. Container states: {}",
                running_pods,
                failed_pods,
                pending_pods,
                total_pods,
                reason_summary.join(", ")
            )
        };
        checks.push(CheckResult {
            name: "Pod Health".to_string(),
            description: "Checks if pods are running successfully".to_string(),
            status: if health_score >= 95.0 {
                CheckStatus::Pass
            } else if health_score >= 80.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: health_score,
            max_score: 100.0,
            details: Some(health_details),
            recommendations: if health_score < 95.0 {
                vec!["Investigate failed and pending pods".to_string()]
            } else {
                vec![]
            },
        });

        // Restart count check
        let restart_score = if total_pods > 0 {
            ((total_pods - pods_with_restarts) as f64 / total_pods as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Pod Stability".to_string(),
            description: "Checks for excessive pod restarts".to_string(),
            status: if restart_score >= 90.0 {
                CheckStatus::Pass
            } else if restart_score >= 70.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: restart_score,
            max_score: 100.0,
            details: Some(format!("{}/{} pods with excessive restarts", pods_with_restarts, total_pods)),
            recommendations: if restart_score < 90.0 {
                vec!["Review application logs and resource limits".to_string()]
            } else {
                vec![]
            },
        });

        let overall_score = checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64;

        let summary = self.create_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Pod Status".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: if pod_container_states.is_empty() {
                None
            } else {
                Some(pod_container_states)
            },
            namespace_summary_rows: None,
        })
    }

    fn create_summary(&self, checks: &[CheckResult], issues: Vec<Issue>) -> InspectionSummary {
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