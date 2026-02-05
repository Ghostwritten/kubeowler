use anyhow::Result;
use chrono::Utc;
use kube::api::ListParams;
use log::info;

use crate::k8s::K8sClient;
use crate::inspections::types::*;

pub struct StorageInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> StorageInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        info!("Starting storage inspection");

        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Check Persistent Volumes
        let pv_api = self.client.persistent_volumes();
        let pvs = pv_api.list(&ListParams::default()).await?;

        let mut total_pvs = 0;
        let mut available_pvs = 0;
        let mut bound_pvs = 0;
        let mut failed_pvs = 0;

        for pv in &pvs.items {
            let pv_name = pv.metadata.name.as_deref().unwrap_or("unknown");
            total_pvs += 1;

            if let Some(status) = &pv.status {
                match status.phase.as_deref() {
                    Some("Available") => available_pvs += 1,
                    Some("Bound") => bound_pvs += 1,
                    Some("Failed") => {
                        failed_pvs += 1;
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "PersistentVolume".to_string(),
                            description: format!("Persistent Volume {} is in Failed state", pv_name),
                            resource: Some(pv_name.to_string()),
                            recommendation: "Check PV configuration and underlying storage".to_string(),
                            rule_id: Some("STO-001".to_string()),
                        });
                    }
                    Some("Released") => {
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "PersistentVolume".to_string(),
                            description: format!("Persistent Volume {} is Released but not reclaimed", pv_name),
                            resource: Some(pv_name.to_string()),
                            recommendation: "Check reclaim policy and clean up released PVs".to_string(),
                            rule_id: Some("STO-002".to_string()),
                        });
                    }
                    _ => {}
                }
            }

            // Check PV reclaim policy
            if let Some(spec) = &pv.spec {
                match spec.persistent_volume_reclaim_policy.as_deref() {
                    Some("Delete") => {
                        // This is fine for dynamic provisioning
                    }
                    Some("Retain") => {
                        // This might accumulate orphaned PVs
                        if pv.status.as_ref().and_then(|s| s.phase.as_deref()) == Some("Released") {
                        issues.push(Issue {
                                severity: IssueSeverity::Info,
                            category: "PersistentVolume".to_string(),
                                description: format!("PV {} with Retain policy is Released", pv_name),
                                resource: Some(pv_name.to_string()),
                                recommendation: "Monitor and clean up retained PVs manually".to_string(),
                                rule_id: Some("STO-003".to_string()),
                            });
                        }
                    }
                    _ => {
                        issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "PersistentVolume".to_string(),
                            description: format!("PV {} has unclear reclaim policy", pv_name),
                            resource: Some(pv_name.to_string()),
                            recommendation: "Set explicit reclaim policy (Retain or Delete)".to_string(),
                            rule_id: Some("STO-004".to_string()),
                        });
                    }
                }
            }
        }

        // Check Persistent Volume Claims
        let pvc_api = self.client.persistent_volume_claims(namespace);
        let pvcs = pvc_api.list(&ListParams::default()).await?;

        let mut total_pvcs = 0;
        let mut bound_pvcs = 0;
        let mut _pending_pvcs = 0;

        for pvc in &pvcs.items {
            let pvc_name = pvc.metadata.name.as_deref().unwrap_or("unknown");
            let pvc_namespace = pvc.metadata.namespace.as_deref().unwrap_or("default");
            total_pvcs += 1;

            if let Some(status) = &pvc.status {
                match status.phase.as_deref() {
                    Some("Bound") => bound_pvcs += 1,
                    Some("Pending") => {
                        _pending_pvcs += 1;
                    issues.push(Issue {
                            severity: IssueSeverity::Warning,
                        category: "PersistentVolumeClaim".to_string(),
                            description: format!("PVC {}/{} is pending", pvc_namespace, pvc_name),
                            resource: Some(format!("{}/{}", pvc_namespace, pvc_name)),
                            recommendation: "Check storage class availability and node capacity".to_string(),
                            rule_id: Some("STO-005".to_string()),
                        });
                    }
                    Some("Lost") => {
                    issues.push(Issue {
                            severity: IssueSeverity::Critical,
                        category: "PersistentVolumeClaim".to_string(),
                            description: format!("PVC {}/{} is lost", pvc_namespace, pvc_name),
                            resource: Some(format!("{}/{}", pvc_namespace, pvc_name)),
                            recommendation: "Data may be lost, check backup and recovery procedures".to_string(),
                            rule_id: Some("STO-006".to_string()),
                        });
                    }
                    _ => {}
                }
            }

            // Check if PVC uses storage class
            if let Some(spec) = &pvc.spec {
                if spec.storage_class_name.is_none() {
                    issues.push(Issue {
                        severity: IssueSeverity::Info,
                        category: "PersistentVolumeClaim".to_string(),
                        description: format!("PVC {}/{} has no storage class specified", pvc_namespace, pvc_name),
                        resource: Some(format!("{}/{}", pvc_namespace, pvc_name)),
                        recommendation: "Specify storage class for better provisioning control".to_string(),
                        rule_id: Some("STO-007".to_string()),
                    });
                }
            }
        }

        // Check Storage Classes
        let sc_api = self.client.storage_classes();
        let storage_classes = sc_api.list(&ListParams::default()).await?;

        let mut total_storage_classes = 0;
        let mut default_storage_classes = 0;

        for sc in &storage_classes.items {
            let sc_name = sc.metadata.name.as_deref().unwrap_or("unknown");
            total_storage_classes += 1;

            if let Some(annotations) = &sc.metadata.annotations {
                if annotations.get("storageclass.kubernetes.io/is-default-class") == Some(&"true".to_string()) {
                    default_storage_classes += 1;
                }
            }

            // Check provisioner
            if sc.provisioner.is_empty() {
                issues.push(Issue {
                    severity: IssueSeverity::Critical,
                    category: "StorageClass".to_string(),
                    description: format!("Storage class {} has no provisioner", sc_name),
                    resource: Some(sc_name.to_string()),
                    recommendation: "Configure proper provisioner for storage class".to_string(),
                    rule_id: Some("STO-008".to_string()),
                });
            }
        }

        // Check for proper default storage class configuration
        if default_storage_classes == 0 {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "StorageClass".to_string(),
                description: "No default storage class configured".to_string(),
                resource: None,
                recommendation: "Configure a default storage class for automatic PV provisioning".to_string(),
                rule_id: Some("STO-009".to_string()),
            });
        } else if default_storage_classes > 1 {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "StorageClass".to_string(),
                description: format!("{} default storage classes configured", default_storage_classes),
                resource: None,
                recommendation: "Only one storage class should be marked as default".to_string(),
                rule_id: Some("STO-010".to_string()),
            });
        }

        // PV health check
        let pv_health_score = if total_pvs > 0 {
            ((total_pvs - failed_pvs) as f64 / total_pvs as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "Persistent Volume Health".to_string(),
            description: "Checks if persistent volumes are in healthy state".to_string(),
            status: if pv_health_score >= 95.0 {
                CheckStatus::Pass
            } else if pv_health_score >= 80.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: pv_health_score,
            max_score: 100.0,
            details: Some(format!(
                "Available: {}, Bound: {}, Failed: {}, Total: {}",
                available_pvs, bound_pvs, failed_pvs, total_pvs
            )),
            recommendations: if pv_health_score < 95.0 {
                vec!["Investigate and resolve failed persistent volumes".to_string()]
            } else {
                vec![]
            },
        });

        // PVC binding check
        let pvc_binding_score = if total_pvcs > 0 {
            (bound_pvcs as f64 / total_pvcs as f64) * 100.0
        } else {
            100.0
        };

        checks.push(CheckResult {
            name: "PVC Binding".to_string(),
            description: "Checks if persistent volume claims are properly bound".to_string(),
            status: if pvc_binding_score >= 95.0 {
                CheckStatus::Pass
            } else if pvc_binding_score >= 80.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: pvc_binding_score,
            max_score: 100.0,
            details: Some(format!("{}/{} PVCs are bound", bound_pvcs, total_pvcs)),
            recommendations: if pvc_binding_score < 95.0 {
                vec!["Resolve pending PVCs and check storage availability".to_string()]
            } else {
                vec![]
            },
        });

        // Storage class configuration check
        let sc_config_score = if total_storage_classes > 0 && default_storage_classes == 1 {
            100.0
        } else if total_storage_classes > 0 {
            70.0
        } else {
            0.0
        };

        checks.push(CheckResult {
            name: "Storage Class Configuration".to_string(),
            description: "Checks storage class setup and default configuration".to_string(),
            status: if sc_config_score >= 90.0 {
                CheckStatus::Pass
            } else if sc_config_score >= 60.0 {
                CheckStatus::Warning
            } else {
                CheckStatus::Critical
            },
            score: sc_config_score,
            max_score: 100.0,
            details: Some(format!("{} storage classes, {} default", total_storage_classes, default_storage_classes)),
            recommendations: if sc_config_score < 90.0 {
                vec!["Configure appropriate storage classes and set one as default".to_string()]
            } else {
                vec![]
            },
        });

        let overall_score = checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64;

        let summary = self.create_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Storage".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
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