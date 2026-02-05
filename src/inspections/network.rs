use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use log::info;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

pub struct NetworkInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> NetworkInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        info!("Starting network connectivity inspection");

        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Check services
        let services_api = self.client.services(namespace);
        let services = services_api.list(&ListParams::default()).await?;

        let mut total_services = 0;
        let mut services_with_endpoints = 0;
        let mut _headless_services = 0;

        for service in &services.items {
            let service_name = service.metadata.name.as_deref().unwrap_or("unknown");
            let service_namespace = service.metadata.namespace.as_deref().unwrap_or("default");

            total_services += 1;

            if let Some(spec) = &service.spec {
                // Check if service is headless
                if spec.cluster_ip.as_deref() == Some("None") {
                    _headless_services += 1;
                }

                // Check service type and configuration
                match spec.type_.as_deref() {
                    Some("LoadBalancer") => {
                        if let Some(status) = &service.status {
                            if let Some(load_balancer) = &status.load_balancer {
                                if load_balancer.ingress.is_none() || load_balancer.ingress.as_ref().unwrap().is_empty() {
                                    issues.push(Issue {
                                        severity: IssueSeverity::Warning,
                                        category: "Service".to_string(),
                                        description: format!(
                                            "LoadBalancer service {}/{} has no external IP assigned",
                                            service_namespace, service_name
                                        ),
                                        resource: Some(format!("{}/{}", service_namespace, service_name)),
                                        recommendation: "Check LoadBalancer configuration and cloud provider settings".to_string(),
                                        rule_id: Some("NET-001".to_string()),
                                    });
                                }
                            }
                        }
                    }
                    Some("NodePort") => {
                        if let Some(ports) = &spec.ports {
                            for port in ports {
                                if let Some(node_port) = port.node_port {
                                    if node_port < 30000 || node_port > 32767 {
                                        issues.push(Issue {
                                            severity: IssueSeverity::Info,
                                            category: "Service".to_string(),
                                            description: format!(
                                                "Service {}/{} uses NodePort {} outside recommended range",
                                                service_namespace, service_name, node_port
                                            ),
                                            resource: Some(format!("{}/{}", service_namespace, service_name)),
                                            recommendation: "Use NodePort in range 30000-32767".to_string(),
                                            rule_id: Some("NET-002".to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Check if service has selectors (for endpoint discovery)
                if spec.selector.is_some() && !spec.selector.as_ref().unwrap().is_empty() {
                    services_with_endpoints += 1;
                } else if spec.cluster_ip.as_deref() != Some("None") {
                    // Exclude default/kubernetes (default API server service)
                    if !(service_namespace == "default" && service_name == "kubernetes") {
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "Service".to_string(),
                            description: format!(
                                "Service {}/{} has no selector and may not have endpoints",
                                service_namespace, service_name
                            ),
                            resource: Some(format!("{}/{}", service_namespace, service_name)),
                            recommendation: "Ensure service has proper selectors or manual endpoints".to_string(),
                            rule_id: Some("NET-003".to_string()),
                        });
                    }
                }
            }
        }

        // Check network policies
        let network_policies_api = self.client.network_policies(namespace);
        let network_policies = network_policies_api.list(&ListParams::default()).await?;

        let namespaces_api = self.client.namespaces();
        let namespaces_list = namespaces_api.list(&ListParams::default()).await?;
        let total_namespaces = namespaces_list.items.len();

        let mut namespaces_with_policies = std::collections::HashSet::new();
        for policy in &network_policies.items {
            if let Some(policy_namespace) = &policy.metadata.namespace {
                namespaces_with_policies.insert(policy_namespace.clone());
            }
        }

        // DNS check (simplified)
        let dns_check = self.check_dns_configuration(&mut issues).await?;

        // Service connectivity check
        let service_score = if total_services > 0 {
            (services_with_endpoints as f64 / total_services as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Service Configuration".to_string(),
            description: "Checks if services are properly configured with selectors".to_string(),
            status: if service_score >= 90.0 {
                CheckStatus::Pass
            } else if service_score >= 70.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: service_score,
            max_score: 100.0,
            details: Some(format!("{}/{} services with proper configuration", services_with_endpoints, total_services)),
            recommendations: if service_score < 90.0 {
                vec!["Review service configurations and selectors".to_string()]
            } else {
                vec![]
            },
        });

        // Network policy coverage
        let policy_coverage = if total_namespaces > 0 {
            (namespaces_with_policies.len() as f64 / total_namespaces as f64) * 100.0
        } else {
            0.0
        };

        checks.push(CheckResult {
            name: "Network Policy Coverage".to_string(),
            description: "Checks if namespaces have network policies for security".to_string(),
            status: if policy_coverage >= 70.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: policy_coverage,
            max_score: 100.0,
            details: Some(format!("{}/{} namespaces with network policies", namespaces_with_policies.len(), total_namespaces)),
            recommendations: if policy_coverage < 70.0 {
                vec!["Implement network policies for better security isolation".to_string()]
            } else {
                vec![]
            },
        });

        // DNS configuration check
        checks.push(CheckResult {
            name: "DNS Configuration".to_string(),
            description: "Checks DNS service availability".to_string(),
            status: if dns_check {
                CheckStatus::Pass
            } else {
                CheckStatus::Critical
            },
            score: if dns_check { 100.0 } else { 0.0 },
            max_score: 100.0,
            details: Some(if dns_check {
                "DNS service is available".to_string()
            } else {
                "DNS service issues detected".to_string()
            }),
            recommendations: if !dns_check {
                vec!["Check CoreDNS or kube-dns deployment".to_string()]
            } else {
                vec![]
            },
        });

        let overall_score = checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64;

        let summary = self.create_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Network Connectivity".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn check_dns_configuration(&self, issues: &mut Vec<Issue>) -> Result<bool> {
        // Check for CoreDNS or kube-dns deployment
        let deployments_api = self.client.deployments(Some("kube-system"));
        let deployments = deployments_api.list(&ListParams::default()).await?;

        let mut has_dns_deployment = false;
        for deployment in &deployments.items {
            if let Some(name) = &deployment.metadata.name {
                if name.contains("coredns") || name.contains("kube-dns") {
                    has_dns_deployment = true;

                    // Check if deployment is ready
                    if let Some(status) = &deployment.status {
                        let ready_replicas = status.ready_replicas.unwrap_or(0);
                        let desired_replicas = status.replicas.unwrap_or(0);

                        if ready_replicas < desired_replicas {
                            issues.push(Issue {
                                severity: IssueSeverity::Critical,
                                category: "Deployment".to_string(),
                                description: format!("DNS deployment {} has {}/{} replicas ready", name, ready_replicas, desired_replicas),
                                resource: Some(format!("kube-system/{}", name)),
                                recommendation: "Check DNS deployment logs and resource availability".to_string(),
                                rule_id: Some("NET-004".to_string()),
                            });
                            return Ok(false);
                        }
                    }
                    break;
                }
            }
        }

        if !has_dns_deployment {
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                category: "Namespace".to_string(),
                description: "No DNS service deployment found".to_string(),
                resource: Some("kube-system".to_string()),
                recommendation: "Deploy CoreDNS or kube-dns for cluster DNS resolution".to_string(),
                rule_id: Some("NET-005".to_string()),
            });
            return Ok(false);
        }

        Ok(true)
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