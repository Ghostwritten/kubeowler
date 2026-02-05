use anyhow::Result;
use chrono::Utc;
use kube::{api::ListParams, Api};
use k8s_openapi::api::core::v1::{ComponentStatus, Pod};

/// ComponentStatus API was removed in Kubernetes 1.24; list can return 404 or "not found".
fn is_component_status_unavailable(err: &kube::Error) -> bool {
    match err {
        kube::Error::Api(ae) => {
            ae.code == 404
                || ae.code == 410
                || ae.message.contains("could not find the requested resource")
                || ae.reason.eq_ignore_ascii_case("NotFound")
        }
        _ => false,
    }
}

use crate::k8s::K8sClient;
use crate::inspections::types::*;

const CONTROL_PLANE_POD_KEYWORDS: [&str; 4] = [
    "kube-apiserver",
    "kube-controller-manager",
    "kube-scheduler",
    "etcd",
];

pub struct ControlPlaneInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> ControlPlaneInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Component status check
        let component_check = self.inspect_component_statuses(&mut issues).await?;
        checks.push(component_check);

        // Control-plane pod check
        let pod_check = self.inspect_control_plane_pods(&mut issues).await?;
        checks.push(pod_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Control Plane".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_component_statuses(&self, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let api: Api<ComponentStatus> = Api::all(self.client.client().clone());
        let statuses = match api.list(&ListParams::default()).await {
            Ok(s) => s,
            Err(e) if is_component_status_unavailable(&e) => {
                return Ok(CheckResult {
                    name: "Component Status".to_string(),
                    description: "Checks the health of core control-plane components".to_string(),
                    status: CheckStatus::Pass,
                    score: 100.0,
                    max_score: 100.0,
                    details: Some(
                        "Component Status API not available (e.g. Kubernetes 1.24+); check skipped.".to_string(),
                    ),
                    recommendations: vec![],
                });
            }
            Err(e) => return Err(e.into()),
        };

        let total = statuses.items.len();
        let mut healthy = 0usize;

        for status in statuses {
            let name = status.metadata.name.unwrap_or_else(|| "unknown".to_string());
            if let Some(conditions) = status.conditions {
                let mut component_healthy = true;
                for condition in conditions {
                    if condition.status.as_str() != "True" {
                        component_healthy = false;
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "ControlPlane".to_string(),
                            description: format!("Component {} reports {} = {}", name, condition.type_, condition.status),
                            resource: Some(name.clone()),
                            recommendation: "Inspect control-plane logs and ensure all components are running and healthy.".to_string(),
                            rule_id: Some("CTRL-001".to_string()),
                        });
                    }
                }
                if component_healthy {
                    healthy += 1;
                }
            }
        }

        let score = if total == 0 {
            0.0
        } else {
            (healthy as f64 / total as f64) * 100.0
        };

        let status = if score >= 99.9 {
            CheckStatus::Pass
        } else if score >= 80.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "Component Status".to_string(),
            description: "Checks the health of core control-plane components".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(format!("{}/{} components healthy", healthy, total)),
            recommendations: if score < 100.0 {
                vec!["Review kube-system pod logs and ensure all static pods are Running.".to_string()]
            } else {
                vec![]
            },
        })
    }

    async fn inspect_control_plane_pods(&self, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let pods_api = self.client.pods(Some("kube-system"));
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut evaluated = 0usize;
        let mut healthy = 0usize;

        for pod in pods.items {
            if let Some(name) = pod.metadata.name.clone() {
                if CONTROL_PLANE_POD_KEYWORDS.iter().any(|k| name.contains(k)) {
                    evaluated += 1;
                    if !is_pod_running(&pod) {
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "ControlPlane".to_string(),
                            description: format!("Control plane pod {} is not running", name),
                            resource: Some(name.clone()),
                            recommendation: "Check the static pod manifest and node health for this component.".to_string(),
                            rule_id: Some("CTRL-002".to_string()),
                        });
                    } else {
                        healthy += 1;
                    }
                }
            }
        }

        let score = if evaluated == 0 {
            100.0
        } else {
            (healthy as f64 / evaluated as f64) * 100.0
        };

        let status = if score >= 99.9 {
            CheckStatus::Pass
        } else if score >= 80.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "Control Plane Pods".to_string(),
            description: "Validates that key control-plane pods in kube-system are running".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(if evaluated == 0 {
                "No static control-plane pods detected (managed control plane?)".to_string()
            } else {
                format!("{}/{} control-plane pods running", healthy, evaluated)
            }),
            recommendations: if score < 100.0 {
                vec!["Ensure kube-apiserver, controller-manager, scheduler, and etcd pods are running without restarts.".to_string()]
            } else {
                vec![]
            },
        })
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

fn is_pod_running(pod: &Pod) -> bool {
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
