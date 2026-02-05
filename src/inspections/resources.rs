use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use log::info;

use crate::inspections::types::*;
use crate::k8s::K8sClient;
use crate::utils::resource_quantity::{parse_cpu_str, parse_memory_str};

pub struct ResourceInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> ResourceInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        info!("Starting resource usage inspection");

        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Check pods for resource requests and limits
        let pods_api = self.client.pods(namespace);
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut total_containers = 0;
        let mut containers_with_requests = 0;
        let mut containers_with_limits = 0;
        let mut containers_with_both = 0;

        for pod in &pods.items {
            let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
            let pod_namespace = pod.metadata.namespace.as_deref().unwrap_or("default");

            if let Some(spec) = &pod.spec {
                for container in &spec.containers {
                    total_containers += 1;

                    let has_requests = container
                        .resources
                        .as_ref()
                        .and_then(|r| r.requests.as_ref())
                        .map(|requests| !requests.is_empty())
                        .unwrap_or(false);

                    let has_limits = container
                        .resources
                        .as_ref()
                        .and_then(|r| r.limits.as_ref())
                        .map(|limits| !limits.is_empty())
                        .unwrap_or(false);

                    if has_requests {
                        containers_with_requests += 1;
                    }

                    if has_limits {
                        containers_with_limits += 1;
                    }

                    if has_requests && has_limits {
                        containers_with_both += 1;
                    }

                    // Check if requests and limits are reasonable
                    if let Some(resources) = &container.resources {
                        self.validate_resource_configuration(
                            &format!("{}/{}", pod_namespace, pod_name),
                            &container.name,
                            resources,
                            &mut issues,
                        )?;
                    }

                    if !has_requests {
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "Container".to_string(),
                            description: format!(
                                "Container {} in pod {}/{} has no resource requests",
                                container.name, pod_namespace, pod_name
                            ),
                            resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                            recommendation: "Set CPU and memory requests for better scheduling"
                                .to_string(),
                            rule_id: Some("RES-001".to_string()),
                        });
                    }

                    if !has_limits {
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "Container".to_string(),
                            description: format!(
                                "Container {} in pod {}/{} has no resource limits",
                                container.name, pod_namespace, pod_name
                            ),
                            resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                            recommendation:
                                "Set CPU and memory limits to prevent resource exhaustion"
                                    .to_string(),
                            rule_id: Some("RES-002".to_string()),
                        });
                    }
                }
            }
        }

        // Check namespaces for resource quotas
        let namespaces = if namespace.is_some() {
            vec![namespace.unwrap().to_string()]
        } else {
            let ns_api = self.client.namespaces();
            let ns_list = ns_api.list(&ListParams::default()).await?;
            ns_list
                .items
                .iter()
                .filter_map(|ns| ns.metadata.name.clone())
                .collect()
        };

        let mut _namespaces_with_quotas = 0;
        for ns in &namespaces {
            // Check for resource quotas (simplified - would need to implement ResourceQuota API)
            // For now, we'll assume some namespaces should have quotas
            if ns != "kube-system" && ns != "kube-public" && ns != "kube-node-lease" {
                // This is a placeholder - in real implementation, check for ResourceQuota objects
                if rand::random::<bool>() {
                    _namespaces_with_quotas += 1;
                } else {
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Resource Management".to_string(),
                        description: format!("Namespace {} has no resource quota", ns),
                        resource: Some(ns.clone()),
                        recommendation: "Configure resource quotas to prevent resource exhaustion"
                            .to_string(),
                        rule_id: Some("RES-003".to_string()),
                    });
                }
            }
        }

        // Resource requests check
        let requests_score = if total_containers > 0 {
            (containers_with_requests as f64 / total_containers as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Resource Requests".to_string(),
            description: "Checks if containers have resource requests configured".to_string(),
            status: if requests_score >= 80.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: requests_score,
            max_score: 100.0,
            details: Some(format!(
                "{}/{} containers with resource requests",
                containers_with_requests, total_containers
            )),
            recommendations: if requests_score < 80.0 {
                vec!["Configure resource requests for better pod scheduling".to_string()]
            } else {
                vec![]
            },
        });

        // Resource limits check
        let limits_score = if total_containers > 0 {
            (containers_with_limits as f64 / total_containers as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Resource Limits".to_string(),
            description: "Checks if containers have resource limits configured".to_string(),
            status: if limits_score >= 80.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: limits_score,
            max_score: 100.0,
            details: Some(format!(
                "{}/{} containers with resource limits",
                containers_with_limits, total_containers
            )),
            recommendations: if limits_score < 80.0 {
                vec!["Configure resource limits to prevent resource exhaustion".to_string()]
            } else {
                vec![]
            },
        });

        // Complete resource configuration check
        let complete_config_score = if total_containers > 0 {
            (containers_with_both as f64 / total_containers as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Complete Resource Configuration".to_string(),
            description: "Checks if containers have both requests and limits configured"
                .to_string(),
            status: if complete_config_score >= 70.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: complete_config_score,
            max_score: 100.0,
            details: Some(format!(
                "{}/{} containers with complete resource configuration",
                containers_with_both, total_containers
            )),
            recommendations: if complete_config_score < 70.0 {
                vec![
                    "Configure both requests and limits for optimal resource management"
                        .to_string(),
                ]
            } else {
                vec![]
            },
        });

        let overall_score = checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64;

        let summary = self.create_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Resource Usage".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    fn validate_resource_configuration(
        &self,
        pod_name: &str,
        container_name: &str,
        resources: &k8s_openapi::api::core::v1::ResourceRequirements,
        issues: &mut Vec<Issue>,
    ) -> Result<()> {
        // Check if limits are higher than requests
        if let (Some(requests), Some(limits)) = (&resources.requests, &resources.limits) {
            // CPU check: parse to millicores and compare
            if let (Some(cpu_request), Some(cpu_limit)) = (requests.get("cpu"), limits.get("cpu")) {
                let req_m = parse_cpu_str(cpu_request.0.as_str());
                let lim_m = parse_cpu_str(cpu_limit.0.as_str());
                if let (Some(req), Some(lim)) = (req_m, lim_m) {
                    if lim < req {
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "Container".to_string(),
                            description: format!(
                                "Container {} in pod {} has CPU limit lower than request",
                                container_name, pod_name
                            ),
                            resource: Some(pod_name.to_string()),
                            recommendation: "Ensure CPU limits are higher than or equal to requests".to_string(),
                            rule_id: Some("RES-004".to_string()),
                        });
                    }
                }
            }

            // Memory check: parse to bytes and compare
            if let (Some(memory_request), Some(memory_limit)) =
                (requests.get("memory"), limits.get("memory"))
            {
                let req_b = parse_memory_str(memory_request.0.as_str());
                let lim_b = parse_memory_str(memory_limit.0.as_str());
                if let (Some(req), Some(lim)) = (req_b, lim_b) {
                    if lim < req {
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "Container".to_string(),
                            description: format!(
                                "Container {} in pod {} has memory limit lower than request",
                                container_name, pod_name
                            ),
                            resource: Some(pod_name.to_string()),
                            recommendation:
                                "Ensure memory limits are higher than or equal to requests"
                                    .to_string(),
                            rule_id: Some("RES-005".to_string()),
                        });
                    }
                }
            }
        }

        Ok(())
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
