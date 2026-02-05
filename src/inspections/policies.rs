use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use kube::Api;
use k8s_openapi::api::core::v1::{LimitRange, ResourceQuota};
use k8s_openapi::api::policy::v1::PodDisruptionBudget;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

pub struct PoliciesInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> PoliciesInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let quota_check = self.inspect_resource_quotas(namespace, &mut issues).await?;
        let limit_check = self.inspect_limit_ranges(namespace, &mut issues).await?;
        let pdb_check = self.inspect_pdbs(namespace, &mut issues).await?;

        checks.push(quota_check);
        checks.push(limit_check);
        checks.push(pdb_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Policy & Governance".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_resource_quotas(&self, namespace: Option<&str>, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let quota_api: Api<ResourceQuota> = match namespace {
            Some(ns) => Api::namespaced(self.client.client().clone(), ns),
            None => Api::all(self.client.client().clone()),
        };
        let quotas = quota_api.list(&ListParams::default()).await?;

        if namespace.is_some() {
            if quotas.items.is_empty() {
                issues.push(Issue {
                    severity: IssueSeverity::Warning,
                    category: "Policy".to_string(),
                    description: "Namespace lacks ResourceQuota".to_string(),
                    resource: namespace.map(|ns| ns.to_string()),
                    recommendation: "Define ResourceQuota to prevent resource exhaustion.".to_string(),
                    rule_id: Some("POLICY-001".to_string()),
                });
                return Ok(CheckResult {
                    name: "Resource Quotas".to_string(),
                    description: "Checks namespace-level ResourceQuota presence".to_string(),
                    status: CheckStatus::Warning,
                    score: 60.0,
                    max_score: 100.0,
                    details: Some("Namespace has no ResourceQuota".to_string()),
                    recommendations: vec!["Create ResourceQuota to enforce resource boundaries.".to_string()],
                });
            }
        } else if quotas.items.is_empty() {
            return Ok(CheckResult {
                name: "Resource Quotas".to_string(),
                description: "Checks cluster-wide ResourceQuota coverage".to_string(),
                status: CheckStatus::Warning,
                score: 60.0,
                max_score: 100.0,
                details: Some("No ResourceQuota objects found".to_string()),
                recommendations: vec!["Define ResourceQuota in multi-tenant namespaces.".to_string()],
            });
        }

        Ok(CheckResult {
            name: "Resource Quotas".to_string(),
            description: "Checks namespace quotas".to_string(),
            status: CheckStatus::Pass,
            score: 100.0,
            max_score: 100.0,
            details: Some(format!("{} quotas identified", quotas.items.len())),
            recommendations: vec![],
        })
    }

    async fn inspect_limit_ranges(&self, namespace: Option<&str>, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let limit_api: Api<LimitRange> = match namespace {
            Some(ns) => Api::namespaced(self.client.client().clone(), ns),
            None => Api::all(self.client.client().clone()),
        };
        let limits = limit_api.list(&ListParams::default()).await?;

        if limits.items.is_empty() {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Policy".to_string(),
                description: "No LimitRange defined".to_string(),
                resource: Some(namespace.map(|ns| ns.to_string()).unwrap_or_else(|| "cluster".to_string())),
                recommendation: "Define LimitRange to ensure pod resource defaults and limits.".to_string(),
                rule_id: Some("POLICY-002".to_string()),
            });
            return Ok(CheckResult {
                name: "Limit Ranges".to_string(),
                description: "Ensures namespaces have LimitRange for default resource settings".to_string(),
                status: CheckStatus::Warning,
                score: 65.0,
                max_score: 100.0,
                details: Some("No LimitRange objects found".to_string()),
                recommendations: vec!["Create LimitRange to enforce default requests/limits.".to_string()],
            });
        }

        Ok(CheckResult {
            name: "Limit Ranges".to_string(),
            description: "Checks LimitRange presence".to_string(),
            status: CheckStatus::Pass,
            score: 100.0,
            max_score: 100.0,
            details: Some(format!("{} LimitRange objects found", limits.items.len())),
            recommendations: vec![],
        })
    }

    async fn inspect_pdbs(&self, namespace: Option<&str>, issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let pdb_api: Api<PodDisruptionBudget> = match namespace {
            Some(ns) => Api::namespaced(self.client.client().clone(), ns),
            None => Api::all(self.client.client().clone()),
        };
        let pdbs = pdb_api.list(&ListParams::default()).await?;

        if pdbs.items.is_empty() {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Policy".to_string(),
                description: "No PodDisruptionBudget configured".to_string(),
                resource: namespace.map(|ns| ns.to_string()),
                recommendation: "Define PodDisruptionBudget for critical workloads to avoid voluntary eviction impact.".to_string(),
                rule_id: Some("POLICY-003".to_string()),
            });
            return Ok(CheckResult {
                name: "Pod Disruption Budgets".to_string(),
                description: "Checks PDB coverage".to_string(),
                status: CheckStatus::Warning,
                score: 70.0,
                max_score: 100.0,
                details: Some("No PDBs found".to_string()),
                recommendations: vec!["Add PDBs for stateful or critical deployments.".to_string()],
            });
        }

        let mut unhealthy = 0usize;
        for pdb in pdbs.items {
            if let Some(status) = pdb.status {
                let disruptions_allowed = status.disruptions_allowed;
                let expected_pods = status.expected_pods;
                if disruptions_allowed == 0 && expected_pods > 1 {
                    unhealthy += 1;
                    let name = pdb.metadata.name.unwrap_or_else(|| "unknown".to_string());
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Policy".to_string(),
                        description: format!("PDB {} currently blocks disruptions", name),
                        resource: Some(name.clone()),
                        recommendation: "Ensure enough replicas to satisfy PDB requirements.".to_string(),
                        rule_id: Some("POLICY-004".to_string()),
                    });
                }
            }
        }

        let score = if unhealthy == 0 { 100.0 } else { 80.0 }; // Soft penalty
        let status = if unhealthy == 0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Warning
        };

        Ok(CheckResult {
            name: "Pod Disruption Budgets".to_string(),
            description: "Evaluates PDB coverage and status".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(if unhealthy == 0 {
                "All PDBs allow disruption".to_string()
            } else {
                format!("{} PDBs currently block disruption", unhealthy)
            }),
            recommendations: if unhealthy > 0 {
                vec!["Scale workloads or adjust PDB thresholds to allow controlled disruptions.".to_string()]
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
