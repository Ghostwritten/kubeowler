use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use log::info;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

pub struct SecurityInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> SecurityInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        info!("Starting security configuration inspection");

        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Check RBAC configuration
        self.check_rbac_configuration(&mut checks, &mut issues).await?;

        // Check Pod Security Standards
        self.check_pod_security_standards(namespace, &mut checks, &mut issues).await?;

        // Check Network Policies
        self.check_network_policies(namespace, &mut checks, &mut issues).await?;

        // Check Service Account configuration
        self.check_service_accounts(namespace, &mut checks, &mut issues).await?;

        let overall_score = checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64;

        let summary = self.create_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Security Configuration".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn check_rbac_configuration(&self, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) -> Result<()> {
        // Check ClusterRoles
        let cluster_roles_api = self.client.cluster_roles();
        let cluster_roles = cluster_roles_api.list(&ListParams::default()).await?;

        let mut dangerous_cluster_roles = 0;
        let total_cluster_roles = cluster_roles.items.len();

        for role in &cluster_roles.items {
            let role_name = role.metadata.name.as_deref().unwrap_or("unknown");

            if let Some(rules) = &role.rules {
                for rule in rules {
                    // Check for overly permissive rules
                    if rule.verbs.contains(&"*".to_string()) ||
                       rule.resources.as_ref().map_or(false, |r| r.contains(&"*".to_string())) {
                        dangerous_cluster_roles += 1;

                        if !role_name.starts_with("system:") && !role_name.starts_with("cluster-admin") {
                            issues.push(Issue {
                                severity: IssueSeverity::Warning,
                                category: "ClusterRole".to_string(),
                                description: format!("ClusterRole {} has overly permissive rules", role_name),
                                resource: Some(role_name.to_string()),
                                recommendation: "Review and restrict ClusterRole permissions to minimum required".to_string(),
                                rule_id: Some("SEC-001".to_string()),
                            });
                        }
                        break;
                    }
                }
            }
        }

        // Check ClusterRoleBindings
        let cluster_role_bindings_api = self.client.cluster_role_bindings();
        let cluster_role_bindings = cluster_role_bindings_api.list(&ListParams::default()).await?;

        let mut risky_bindings = 0;
        for binding in &cluster_role_bindings.items {
            let binding_name = binding.metadata.name.as_deref().unwrap_or("unknown");

            let role_ref = &binding.role_ref;
            if role_ref.name == "cluster-admin" {
                    if let Some(subjects) = &binding.subjects {
                        for subject in subjects {
                            if subject.kind == "User" && !subject.name.starts_with("system:") {
                                risky_bindings += 1;
                                issues.push(Issue {
                                    severity: IssueSeverity::Warning,
                                    category: "ClusterRoleBinding".to_string(),
                                    description: format!("User {} has cluster-admin privileges", subject.name),
                                    resource: Some(binding_name.to_string()),
                                    recommendation: "Minimize cluster-admin privileges and use more specific roles".to_string(),
                                    rule_id: Some("SEC-002".to_string()),
                                });
                            }
                            if subject.kind == "ServiceAccount" && subject.namespace.as_deref() != Some("kube-system") {
                                risky_bindings += 1;
                                issues.push(Issue {
                                    severity: IssueSeverity::Critical,
                                    category: "ClusterRoleBinding".to_string(),
                                    description: format!(
                                        "ServiceAccount {}/{} has cluster-admin privileges",
                                        subject.namespace.as_deref().unwrap_or("default"),
                                        subject.name
                                    ),
                                    resource: Some(binding_name.to_string()),
                                    recommendation: "Review and restrict ServiceAccount permissions".to_string(),
                                    rule_id: Some("SEC-003".to_string()),
                                });
                            }
                        }
                    }
            }
        }

        let rbac_score = if total_cluster_roles > 0 {
            ((total_cluster_roles - dangerous_cluster_roles) as f64 / total_cluster_roles as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "RBAC Configuration".to_string(),
            description: "Checks for secure RBAC configuration".to_string(),
            status: if rbac_score >= 90.0 && risky_bindings == 0 {
                CheckStatus::Pass
            } else if rbac_score >= 70.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: if risky_bindings > 0 { rbac_score * 0.7 } else { rbac_score },
            max_score: 100.0,
            details: Some(format!("Risky roles: {}, Risky bindings: {}", dangerous_cluster_roles, risky_bindings)),
            recommendations: if rbac_score < 90.0 || risky_bindings > 0 {
                vec!["Review and minimize RBAC permissions".to_string()]
            } else {
                vec![]
            },
        });

        Ok(())
    }

    async fn check_pod_security_standards(&self, namespace: Option<&str>, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) -> Result<()> {
        let pods_api = self.client.pods(namespace);
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut total_pods = 0;
        let mut secure_pods = 0;
        let mut pods_running_as_root = 0;
        let mut pods_with_privileged_containers = 0;

        for pod in &pods.items {
            let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
            let pod_namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
            total_pods += 1;

            let mut pod_is_secure = true;

            if let Some(spec) = &pod.spec {
                // Check security context
                if let Some(security_context) = &spec.security_context {
                    if security_context.run_as_user.is_some() && security_context.run_as_user != Some(0) {
                        // Good - not running as root
                    } else if security_context.run_as_user == Some(0) {
                        pods_running_as_root += 1;
                        pod_is_secure = false;
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "Security".to_string(),
                            description: format!("Pod {}/{} runs as root user", pod_namespace, pod_name),
                            resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                            recommendation: "Configure runAsUser to use non-root user".to_string(),
                                rule_id: Some("SEC-004".to_string()),
                        });
                    }
                } else {
                    // No security context - potentially insecure
                    pod_is_secure = false;
                }

                // Check containers
                for container in &spec.containers {
                    if let Some(security_context) = &container.security_context {
                        if security_context.privileged == Some(true) {
                            pods_with_privileged_containers += 1;
                            pod_is_secure = false;
                            issues.push(Issue {
                                severity: IssueSeverity::Warning,
                                category: "Security".to_string(),
                                description: format!(
                                    "Container {} in pod {}/{} runs in privileged mode",
                                    container.name, pod_namespace, pod_name
                                ),
                                resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                recommendation: "Remove privileged flag unless absolutely necessary".to_string(),
                                rule_id: Some("SEC-005".to_string()),
                            });
                        }

                        if security_context.run_as_user == Some(0) {
                            pods_running_as_root += 1;
                            pod_is_secure = false;
                            issues.push(Issue {
                                severity: IssueSeverity::Warning,
                                category: "Security".to_string(),
                                description: format!(
                                    "Container {} in pod {}/{} runs as root",
                                    container.name, pod_namespace, pod_name
                                ),
                                resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                recommendation: "Configure container to run as non-root user".to_string(),
                                rule_id: Some("SEC-006".to_string()),
                            });
                        }

                        if security_context.allow_privilege_escalation == Some(true) {
                            pod_is_secure = false;
                            issues.push(Issue {
                                severity: IssueSeverity::Warning,
                                category: "Security".to_string(),
                                description: format!(
                                    "Container {} in pod {}/{} allows privilege escalation",
                                    container.name, pod_namespace, pod_name
                                ),
                                resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                                recommendation: "Disable allowPrivilegeEscalation".to_string(),
                                rule_id: Some("SEC-007".to_string()),
                            });
                        }
                    }
                }
            }

            if pod_is_secure {
                secure_pods += 1;
            }
        }

        let pod_security_score = if total_pods > 0 {
            (secure_pods as f64 / total_pods as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Pod Security Standards".to_string(),
            description: "Checks if pods follow security best practices".to_string(),
            status: if pod_security_score >= 90.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: pod_security_score,
            max_score: 100.0,
            details: Some(format!(
                "Secure pods: {}/{}, Running as root: {}, Privileged: {}",
                secure_pods, total_pods, pods_running_as_root, pods_with_privileged_containers
            )),
            recommendations: if pod_security_score < 90.0 {
                vec!["Configure security contexts for better pod security".to_string()]
            } else {
                vec![]
            },
        });

        Ok(())
    }

    async fn check_network_policies(&self, namespace: Option<&str>, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) -> Result<()> {
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

        let coverage_score = if total_namespaces > 0 {
            (namespaces_with_policies.len() as f64 / total_namespaces as f64) * 100.0
        } else {
            100.0
        };

        if coverage_score < 50.0 {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "NetworkPolicy".to_string(),
                description: "Low network policy coverage across namespaces".to_string(),
                resource: Some("cluster".to_string()),
                recommendation: "Implement network policies for traffic segmentation".to_string(),
                rule_id: Some("SEC-008".to_string()),
            });
        }

        checks.push(CheckResult {
            name: "Network Policy Coverage".to_string(),
            description: "Checks network policy implementation for traffic segmentation".to_string(),
            status: if coverage_score >= 70.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: coverage_score,
            max_score: 100.0,
            details: Some(format!("{}/{} namespaces with network policies", namespaces_with_policies.len(), total_namespaces)),
            recommendations: if coverage_score < 70.0 {
                vec!["Implement network policies for better traffic control".to_string()]
            } else {
                vec![]
            },
        });

        Ok(())
    }

    async fn check_service_accounts(&self, namespace: Option<&str>, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) -> Result<()> {
        let pods_api = self.client.pods(namespace);
        let pods = pods_api.list(&ListParams::default()).await?;

        let mut total_pods = 0;
        let mut pods_with_custom_sa = 0;
        let mut _pods_with_default_sa = 0;

        for pod in &pods.items {
            let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
            let pod_namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
            total_pods += 1;

            if let Some(spec) = &pod.spec {
                let service_account = spec.service_account_name.as_deref().unwrap_or("default");

                if service_account == "default" {
                    _pods_with_default_sa += 1;
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "ServiceAccount".to_string(),
                        description: format!("Pod {}/{} uses default service account", pod_namespace, pod_name),
                        resource: Some(format!("{}/{}", pod_namespace, pod_name)),
                        recommendation: "Create and use dedicated service accounts with minimal permissions".to_string(),
                        rule_id: Some("SEC-009".to_string()),
                    });
                } else {
                    pods_with_custom_sa += 1;
                }
            }
        }

        let sa_score = if total_pods > 0 {
            (pods_with_custom_sa as f64 / total_pods as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Service Account Usage".to_string(),
            description: "Checks if pods use dedicated service accounts".to_string(),
            status: if sa_score >= 80.0 {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            score: sa_score,
            max_score: 100.0,
            details: Some(format!("{}/{} pods use custom service accounts", pods_with_custom_sa, total_pods)),
            recommendations: if sa_score < 80.0 {
                vec!["Create dedicated service accounts for applications".to_string()]
            } else {
                vec![]
            },
        });

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