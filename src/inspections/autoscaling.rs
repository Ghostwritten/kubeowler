use anyhow::Result;
use chrono::Utc;
use k8s_openapi::api::autoscaling::v2::{HPAScalingRules, MetricSpec, MetricTarget};
use kube::api::ListParams;

use crate::inspections::types::*;
use crate::k8s::K8sClient;

pub struct AutoscalingInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> AutoscalingInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let hpa_check = self.inspect_hpas(namespace, &mut issues).await?;
        checks.push(hpa_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Autoscaling".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_hpas(
        &self,
        namespace: Option<&str>,
        issues: &mut Vec<Issue>,
    ) -> Result<CheckResult> {
        let hpa_api = self.client.horizontal_pod_autoscalers(namespace);
        let hpas = hpa_api.list(&ListParams::default()).await?;

        if hpas.items.is_empty() {
            return Ok(CheckResult {
                name: "Horizontal Pod Autoscalers".to_string(),
                description: "Evaluates health and configuration of HPAs".to_string(),
                status: CheckStatus::Warning,
                score: 70.0,
                max_score: 100.0,
                details: Some("No HPAs detected in the target scope".to_string()),
                recommendations: vec![
                    "Consider deploying HPAs to improve workload elasticity.".to_string()
                ],
            });
        }

        let mut healthy = 0usize;
        for hpa in &hpas.items {
            let name = hpa
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            // Validate metrics configuration
            if let Some(spec) = &hpa.spec {
                if spec.min_replicas.unwrap_or(1) == spec.max_replicas {
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Autoscaling".to_string(),
                        description: format!("HPA {} has identical min/max replicas", name),
                        resource: Some(name.clone()),
                        recommendation: "Set a wider min/max replica range so the HPA can scale."
                            .to_string(),
                        rule_id: Some("AUTO-001".to_string()),
                    });
                }

                if let Some(metrics) = &spec.metrics {
                    for metric in metrics {
                        self.validate_metric(metric, &name, issues);
                    }
                } else {
                    issues.push(Issue {
                        severity: IssueSeverity::Critical,
                        category: "Autoscaling".to_string(),
                        description: format!("HPA {} has no metrics configured", name),
                        resource: Some(name.clone()),
                        recommendation: "Define CPU/Memory or custom metrics for this HPA."
                            .to_string(),
                        rule_id: Some("AUTO-002".to_string()),
                    });
                }

                if let Some(behavior) = &spec.behavior {
                    self.validate_behavior(behavior.scale_up.as_ref(), &name, "scale-up", issues);
                    self.validate_behavior(
                        behavior.scale_down.as_ref(),
                        &name,
                        "scale-down",
                        issues,
                    );
                }
            }

            // Evaluate status conditions
            if let Some(status) = &hpa.status {
                if let Some(conditions) = status.conditions.as_ref() {
                    if conditions.iter().all(|c| c.status.as_str() == "True") {
                        healthy += 1;
                    } else {
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "Autoscaling".to_string(),
                            description: format!("HPA {} reports unhealthy conditions", name),
                            resource: Some(name.clone()),
                            recommendation:
                                "Check target workload readiness and metrics availability."
                                    .to_string(),
                            rule_id: Some("AUTO-003".to_string()),
                        });
                    }
                }
            }
        }

        let score = (healthy as f64 / hpas.items.len() as f64) * 100.0;
        let status = if score >= 90.0 {
            CheckStatus::Pass
        } else if score >= 70.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "Horizontal Pod Autoscalers".to_string(),
            description: "Checks configuration and health of HPAs".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(format!("{}/{} HPAs healthy", healthy, hpas.items.len())),
            recommendations: if score < 100.0 {
                vec!["Ensure metrics.k8s.io and custom metric APIs are available, and verify workload readiness.".to_string()]
            } else {
                vec![]
            },
        })
    }

    fn validate_metric(&self, metric: &MetricSpec, name: &str, issues: &mut Vec<Issue>) {
        match metric.type_.as_str() {
            "Resource" => {
                if let Some(resource) = &metric.resource {
                    validate_target(&resource.target, resource.name.as_str(), name, issues);
                }
            }
            "Pods" => {
                if let Some(pods) = &metric.pods {
                    validate_target(&pods.target, pods.metric.name.as_str(), name, issues);
                }
            }
            "Object" => {
                if let Some(object) = &metric.object {
                    validate_target(&object.target, object.metric.name.as_str(), name, issues);
                }
            }
            "External" => {
                if let Some(ext) = &metric.external {
                    validate_target(&ext.target, ext.metric.name.as_str(), name, issues);
                }
            }
            "ContainerResource" => {
                if let Some(container) = &metric.container_resource {
                    validate_target(&container.target, container.name.as_str(), name, issues);
                }
            }
            _ => {}
        }
    }

    fn validate_behavior(
        &self,
        rules: Option<&HPAScalingRules>,
        name: &str,
        direction: &str,
        issues: &mut Vec<Issue>,
    ) {
        if let Some(rules) = rules {
            if let Some(select_policy) = &rules.select_policy {
                if select_policy.as_str() == "Disabled" {
                    issues.push(Issue {
                        severity: IssueSeverity::Info,
                        category: "Autoscaling".to_string(),
                        description: format!("HPA {} has {} behavior disabled", name, direction),
                        resource: Some(name.to_string()),
                        recommendation:
                            "Review HPA behavior policy to ensure scaling is permitted when needed."
                                .to_string(),
                        rule_id: Some("AUTO-004".to_string()),
                    });
                }
            }
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

fn validate_target(target: &MetricTarget, metric_name: &str, hpa: &str, issues: &mut Vec<Issue>) {
    if target.average_utilization.is_none()
        && target.average_value.is_none()
        && target.value.is_none()
    {
        issues.push(Issue {
            severity: IssueSeverity::Warning,
            category: "Autoscaling".to_string(),
            description: format!("HPA {} metric {} missing scaling target", hpa, metric_name),
            resource: Some(hpa.to_string()),
            recommendation:
                "Configure averageUtilization, averageValue, or value for the metric target."
                    .to_string(),
            rule_id: Some("AUTO-005".to_string()),
        });
    }
}
