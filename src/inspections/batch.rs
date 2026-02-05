use anyhow::Result;
use chrono::Utc;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;

use crate::inspections::types::*;
use crate::k8s::K8sClient;

pub struct BatchInspector<'a> {
    client: &'a K8sClient,
}

impl<'a> BatchInspector<'a> {
    pub fn new(client: &'a K8sClient) -> Self {
        Self { client }
    }

    pub async fn inspect(&self, namespace: Option<&str>) -> Result<InspectionResult> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        let cron_check = self.inspect_cron_jobs(namespace, &mut issues).await?;
        let job_check = self.inspect_jobs(namespace, &mut issues).await?;

        checks.push(cron_check);
        checks.push(job_check);

        let overall_score = if checks.is_empty() {
            0.0
        } else {
            checks.iter().map(|c| c.score).sum::<f64>() / checks.len() as f64
        };

        let summary = self.build_summary(&checks, issues);

        Ok(InspectionResult {
            inspection_type: "Batch Workloads".to_string(),
            timestamp: Utc::now(),
            overall_score,
            checks,
            summary,
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        })
    }

    async fn inspect_cron_jobs(
        &self,
        namespace: Option<&str>,
        issues: &mut Vec<Issue>,
    ) -> Result<CheckResult> {
        let cron_api = self.client.cron_jobs(namespace);
        let cron_jobs = cron_api.list(&ListParams::default()).await?;

        if cron_jobs.items.is_empty() {
            return Ok(CheckResult {
                name: "CronJobs".to_string(),
                description: "Evaluates CronJob health and schedules".to_string(),
                status: CheckStatus::Warning,
                score: 70.0,
                max_score: 100.0,
                details: Some("No CronJobs detected".to_string()),
                recommendations: vec![
                    "Introduce CronJobs for periodic tasks where applicable.".to_string()
                ],
            });
        }

        let mut healthy = 0usize;
        for cron in &cron_jobs.items {
            let name = cron
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            if let Some(spec) = &cron.spec {
                if spec.suspend == Some(true) {
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Batch".to_string(),
                        description: format!("CronJob {} is suspended", name),
                        resource: Some(name.clone()),
                        recommendation: "Enable CronJob or remove if no longer needed.".to_string(),
                        rule_id: Some("BATCH-001".to_string()),
                    });
                    continue;
                }
            }

            if let Some(status) = &cron.status {
                let last_schedule = status.last_schedule_time.as_ref().map(|t| t.0);
                let last_success = status.last_successful_time.as_ref().map(|t| t.0);

                if let Some(schedule_time) = last_schedule {
                    if last_success.map(|s| s < schedule_time).unwrap_or(true) {
                        issues.push(Issue {
                            severity: IssueSeverity::Critical,
                            category: "Batch".to_string(),
                            description: format!("CronJob {} last run failed", name),
                            resource: Some(name.clone()),
                            recommendation:
                                "Check CronJob job logs and fix failures before next schedule."
                                    .to_string(),
                            rule_id: Some("BATCH-002".to_string()),
                        });
                        continue;
                    }
                }

                if last_schedule.is_none() {
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Batch".to_string(),
                        description: format!("CronJob {} never executed", name),
                        resource: Some(name.clone()),
                        recommendation:
                            "Ensure CronJob schedule is correct and controller is running."
                                .to_string(),
                        rule_id: Some("BATCH-003".to_string()),
                    });
                    continue;
                }
            }
            healthy += 1;
        }

        let score = (healthy as f64 / cron_jobs.items.len() as f64) * 100.0;
        let status = if score >= 90.0 {
            CheckStatus::Pass
        } else if score >= 70.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "CronJobs".to_string(),
            description: "Checks CronJob scheduling and execution status".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(format!(
                "{}/{} CronJobs healthy",
                healthy,
                cron_jobs.items.len()
            )),
            recommendations: if score < 90.0 {
                vec!["Review CronJob failure events and tune schedule or retry policy.".to_string()]
            } else {
                vec![]
            },
        })
    }

    async fn inspect_jobs(
        &self,
        namespace: Option<&str>,
        issues: &mut Vec<Issue>,
    ) -> Result<CheckResult> {
        let job_api: kube::Api<Job> = if let Some(ns) = namespace {
            kube::Api::namespaced(self.client.client().clone(), ns)
        } else {
            kube::Api::all(self.client.client().clone())
        };
        let jobs = job_api.list(&ListParams::default()).await?;

        if jobs.items.is_empty() {
            return Ok(CheckResult {
                name: "Jobs".to_string(),
                description: "Evaluates Job completion and failure retries".to_string(),
                status: CheckStatus::Warning,
                score: 70.0,
                max_score: 100.0,
                details: Some("No Jobs detected".to_string()),
                recommendations: vec![
                    "Use Jobs for one-off batch workloads when needed.".to_string()
                ],
            });
        }

        let mut healthy = 0usize;
        for job in &jobs.items {
            let name = job
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            if let Some(status) = &job.status {
                if status.failed.unwrap_or(0) > 0 {
                    issues.push(Issue {
                        severity: IssueSeverity::Warning,
                        category: "Batch".to_string(),
                        description: format!("Job {} has failed pods", name),
                        resource: Some(name.clone()),
                        recommendation:
                            "Inspect job pod logs and adjust backoffLimit or resource requests."
                                .to_string(),
                        rule_id: Some("BATCH-004".to_string()),
                    });
                    continue;
                }

                if status.active.unwrap_or(0) > 0 && status.succeeded.unwrap_or(0) == 0 {
                    if let Some(start) = status.start_time.as_ref() {
                        let elapsed = Utc::now() - start.0;
                        if elapsed.num_minutes() > 60 {
                            issues.push(Issue {
                                severity: IssueSeverity::Warning,
                                category: "Batch".to_string(),
                                description: format!("Job {} running for over 60 minutes", name),
                                resource: Some(name.clone()),
                                recommendation:
                                    "Check for stuck pods or adjust activeDeadlineSeconds."
                                        .to_string(),
                                rule_id: Some("BATCH-005".to_string()),
                            });
                            continue;
                        }
                    }
                }
            }
            healthy += 1;
        }

        let score = (healthy as f64 / jobs.items.len() as f64) * 100.0;
        let status = if score >= 90.0 {
            CheckStatus::Pass
        } else if score >= 70.0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Critical
        };

        Ok(CheckResult {
            name: "Jobs".to_string(),
            description: "Checks Jobs for stuck or failed executions".to_string(),
            status,
            score,
            max_score: 100.0,
            details: Some(format!("{}/{} Jobs healthy", healthy, jobs.items.len())),
            recommendations: if score < 90.0 {
                vec!["Review job failure events and tune retries/backoff.".to_string()]
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
