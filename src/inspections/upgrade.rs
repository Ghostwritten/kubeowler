use anyhow::Result;
use chrono::Utc;
use kube::Api;
use k8s_openapi::api::core::v1::Node;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

pub struct UpgradeInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> UpgradeInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let version_check = self.inspect_versions().await?;
        let deprecated_check = self.inspect_deprecated_api_usage(&mut issues).await?;
        checks.push(version_check);
        checks.push(deprecated_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Upgrade Readiness".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_versions(&self) -> Result<CheckResult> {
        let nodes_api: Api<Node> = Api::all(self.client.client().clone());
        let nodes = nodes_api.list(&Default::default()).await?;

        if nodes.items.is_empty() {
            return Ok(CheckResult {
                name: "Cluster Version".to_string(),
                description: "Checks control plane and kubelet versions".to_string(),
                status: CheckStatus::Warning,
                score: 60.0,
                max_score: 100.0,
                details: Some("No nodes discovered".to_string()),
                recommendations: vec!["Ensure kubeconfig has cluster-admin access.".to_string()],
            });
        }

        let mut kubelet_versions = Vec::new();
        for node in &nodes.items {
            if let Some(status) = &node.status {
                if let Some(node_info) = &status.node_info {
                    kubelet_versions.push(node_info.kubelet_version.clone());
                }
            }
        }

        kubelet_versions.sort();
        kubelet_versions.dedup();

        let mut recommendations = Vec::new();
        let mut score = 100.0;

        if kubelet_versions.len() > 1 {
            score -= 10.0;
            recommendations.push("Kubelet versions differ; align node upgrades for consistency.".to_string());
        }

        Ok(CheckResult {
            name: "Kubelet Versions".to_string(),
            description: "Collects kubelet versions for upgrade planning".to_string(),
            status: if score >= 90.0 { CheckStatus::Pass } else { CheckStatus::Warning },
            score,
            max_score: 100.0,
            details: Some(format!("Detected kubelet versions: {:?}", kubelet_versions)),
            recommendations,
        })
    }

    /// Informational check: cluster version and recommendation to audit deprecated APIs.
    /// Typed list only returns current API version; full audit requires raw/discovery API.
    async fn inspect_deprecated_api_usage(&self, _issues: &mut Vec<Issue>) -> Result<CheckResult> {
        let cluster_version = self
            .client
            .server_version()
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "unknown".to_string());

        let details = format!(
            "Cluster version: {}. Use kubectl or the official deprecation guide to audit resources for deprecated API versions (e.g. extensions/v1beta1, apps/v1beta1).",
            cluster_version
        );

        Ok(CheckResult {
            name: "Deprecated API usage".to_string(),
            description: "Reminds to audit resources for deprecated or removed API versions before upgrade".to_string(),
            status: CheckStatus::Pass,
            score: 100.0,
            max_score: 100.0,
            details: Some(details),
            recommendations: vec!["Migrate workloads to current API versions before upgrading. See https://kubernetes.io/docs/reference/using-api/deprecation-guide/".to_string()],
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
