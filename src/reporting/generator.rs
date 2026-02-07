use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use crate::inspections::issue_codes;
use crate::inspections::types::*;
use crate::node_inspection::NodeInspectionResult;
use crate::reporting::report_resource::{issue_to_resource_key, REPORT_RESOURCE_ORDER};
use crate::scoring::scoring_engine::ScoringEngine;
use crate::utils::format::truncate_string;

const DEFAULT_MAX_RECOMMENDATIONS: usize = 5;

/// Which check statuses to include in the Check Results table. Default is Warning, Critical, Error (exclude Pass).
#[derive(Clone, Debug)]
pub enum CheckLevelFilter {
    All,
    Only(Vec<CheckStatus>),
}

/// Parse --check-level string: "all" or comma-separated e.g. "warning,critical,error".
pub fn parse_check_level_filter(s: &str) -> CheckLevelFilter {
    let s = s.trim().to_lowercase();
    if s == "all" {
        return CheckLevelFilter::All;
    }
    let mut only = Vec::new();
    for part in s.split(',') {
        match part.trim() {
            "pass" => only.push(CheckStatus::Pass),
            "warning" => only.push(CheckStatus::Warning),
            "critical" => only.push(CheckStatus::Critical),
            "error" => only.push(CheckStatus::Error),
            _ => {}
        }
    }
    if only.is_empty() {
        CheckLevelFilter::Only(vec![
            CheckStatus::Warning,
            CheckStatus::Critical,
            CheckStatus::Error,
        ])
    } else {
        CheckLevelFilter::Only(only)
    }
}

/// Flatten all issues from inspections and group by canonical resource key.
fn group_issues_by_resource(report: &ClusterReport) -> HashMap<String, Vec<Issue>> {
    let mut map: HashMap<String, Vec<Issue>> = HashMap::new();
    for inspection in &report.inspections {
        for issue in &inspection.summary.issues {
            let key = issue_to_resource_key(issue);
            map.entry(key).or_default().push(issue.clone());
        }
    }
    map
}

/// Maps inspection type name to a cluster-recognizable resource object for the Check Results table.
fn inspection_type_to_resource(inspection_type: &str) -> &'static str {
    match inspection_type {
        "Node Health" | "Node Inspection" => "Node",
        "Control Plane" => "Control Plane",
        "Network Connectivity" => "Service",
        "Storage" => "PersistentVolume",
        "Resource Usage" => "Pod",
        "Pod Status" => "Pod",
        "Autoscaling" => "HorizontalPodAutoscaler",
        "Batch Workloads" => "Job",
        "Security Configuration" => "NetworkPolicy",
        "Policy & Governance" => "ResourceQuota",
        "Observability" => "Observability",
        "Namespace" => "Namespace",
        "Certificates" => "Certificate",
        "Upgrade Readiness" => "Node",
        _ => "Other",
    }
}

/// Format affected resources for table cells: one resource per line (Markdown line break: "  \n").
fn format_affected_resources(resources: &[String]) -> String {
    resources
        .iter()
        .map(|r| format!("`{}`", r))
        .collect::<Vec<_>>()
        .join("  \n")
}

/// Build a Markdown anchor slug from a module title (e.g. "Node Health" -> "node-health").
fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            let lower = if c.is_ascii_uppercase() {
                (c as u8 + 32) as char
            } else {
                c
            };
            out.push(lower);
        } else if (c == ' ' || c == '-') && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub struct ReportGenerator {
    #[allow(dead_code)]
    scoring_engine: ScoringEngine,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            scoring_engine: ScoringEngine::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn generate_report(
        &self,
        cluster_report: &ClusterReport,
        output_path: &str,
    ) -> Result<()> {
        self.generate_report_with_filters(
            cluster_report,
            output_path,
            None,
            false,
            None,
            None,
            None,
        )
        .await
    }

    /// Returns the main report as Markdown string (same filtering as generate_report_with_filters, no disk write).
    pub fn generate_markdown_string(
        &self,
        cluster_report: &ClusterReport,
        filter_category: Option<&Vec<String>>,
        max_recommendations: Option<usize>,
        min_severity: Option<IssueSeverity>,
        check_level_filter: Option<CheckLevelFilter>,
    ) -> Result<String> {
        let filtered = if let Some(min) = min_severity {
            Self::apply_severity_filter(cluster_report, min)
        } else {
            cluster_report.clone()
        };
        let filtered = if let Some(filters) = filter_category {
            Self::apply_category_filters(&filtered, filters, max_recommendations)
        } else {
            filtered
        };
        self.generate_main_report(&filtered, max_recommendations, check_level_filter)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_report_with_filters(
        &self,
        cluster_report: &ClusterReport,
        output_path: &str,
        filter_category: Option<&Vec<String>>,
        no_summary: bool,
        max_recommendations: Option<usize>,
        min_severity: Option<IssueSeverity>,
        check_level_filter: Option<CheckLevelFilter>,
    ) -> Result<()> {
        let main_report = self.generate_markdown_string(
            cluster_report,
            filter_category,
            max_recommendations,
            min_severity.clone(),
            check_level_filter,
        )?;
        fs::write(output_path, main_report)?;

        if !no_summary {
            let filtered = if let Some(min) = min_severity {
                Self::apply_severity_filter(cluster_report, min)
            } else {
                cluster_report.clone()
            };
            let filtered = if let Some(filters) = filter_category {
                Self::apply_category_filters(&filtered, filters, max_recommendations)
            } else {
                filtered
            };
            let summary_report = self.generate_summary_report(&filtered)?;
            let summary_path = output_path.replace(".md", "-summary.md");
            fs::write(summary_path, summary_report)?;
        }

        Ok(())
    }

    /// Filter report to only include issues with severity >= min_severity; recalc executive summary.
    fn apply_severity_filter(report: &ClusterReport, min_severity: IssueSeverity) -> ClusterReport {
        let mut new_report = report.clone();
        new_report.inspections = report
            .inspections
            .iter()
            .map(|ins| {
                let mut ins_clone = ins.clone();
                ins_clone.summary.issues = ins
                    .summary
                    .issues
                    .iter()
                    .filter(|iss| iss.severity >= min_severity)
                    .cloned()
                    .collect();
                ins_clone
            })
            .collect();

        let engine = ScoringEngine::new();
        let overall = engine.calculate_weighted_score(&new_report.inspections);
        let health = engine.get_health_status(overall);
        let score_breakdown_details = engine.generate_score_breakdown(&new_report.inspections);
        let mut score_breakdown: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for (k, v) in score_breakdown_details.into_iter() {
            score_breakdown.insert(k, v.score);
        }
        let max_r = DEFAULT_MAX_RECOMMENDATIONS;
        let key_findings = Self::build_aggregated_findings_error_only(&new_report);
        let priority_recommendations = Self::build_aggregated_recommendations(&new_report, max_r);
        new_report.overall_score = overall;
        new_report.executive_summary = ExecutiveSummary {
            health_status: health,
            key_findings,
            priority_recommendations,
            score_breakdown,
        };
        new_report
    }

    fn apply_category_filters(
        report: &ClusterReport,
        filters: &[String],
        max_recommendations: Option<usize>,
    ) -> ClusterReport {
        let lower: Vec<String> = filters.iter().map(|s| s.to_lowercase()).collect();
        let mut new_report = report.clone();
        // Keep only inspection modules that have issues matching the category filter; recalc scores and summary.
        new_report.inspections = report
            .inspections
            .iter()
            .filter_map(|ins| {
                let mut ins_clone = ins.clone();
                ins_clone.summary.issues = ins
                    .summary
                    .issues
                    .iter()
                    .filter(|iss| {
                        lower
                            .iter()
                            .any(|f| iss.category.to_lowercase().contains(f))
                    })
                    .cloned()
                    .collect();

                if ins_clone.summary.issues.is_empty() {
                    return None;
                }

                // Keep checks list unchanged; overall_score remains per-module to avoid misleading stats.

                // Re-aggregate summary counts; checks counts stay as original.
                ins_clone.summary.total_checks = ins.summary.total_checks;
                ins_clone.summary.passed_checks = ins.summary.passed_checks;
                ins_clone.summary.warning_checks = ins.summary.warning_checks;
                ins_clone.summary.critical_checks = ins.summary.critical_checks;
                ins_clone.summary.error_checks = ins.summary.error_checks;
                Some(ins_clone)
            })
            .collect();

        // Rebuild executive summary from remaining modules.
        let engine = ScoringEngine::new();
        let overall = engine.calculate_weighted_score(&new_report.inspections);
        let health = engine.get_health_status(overall);
        let score_breakdown_details = engine.generate_score_breakdown(&new_report.inspections);
        let mut score_breakdown: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for (k, v) in score_breakdown_details.into_iter() {
            score_breakdown.insert(k, v.score);
        }

        let max_r = max_recommendations.unwrap_or(DEFAULT_MAX_RECOMMENDATIONS);
        let key_findings = Self::build_aggregated_findings_error_only(&new_report);
        let priority_recommendations = Self::build_aggregated_recommendations(&new_report, max_r);

        new_report.overall_score = overall;
        new_report.executive_summary = ExecutiveSummary {
            health_status: health,
            key_findings,
            priority_recommendations,
            score_breakdown,
        };

        new_report
    }

    /// Build aggregated key findings from Critical issues: group by rule_id when present, else (category, recommendation).
    /// Output one line per group: code + short title + doc link + count + affected resources (or legacy description/rec).
    #[allow(dead_code)]
    fn build_aggregated_findings(report: &ClusterReport, max_items: usize) -> Vec<String> {
        fn severity_ord(s: &IssueSeverity) -> u8 {
            match s {
                IssueSeverity::Critical => 0,
                IssueSeverity::Warning => 1,
                IssueSeverity::Info => 2,
            }
        }
        type GroupKey = (Option<String>, String, String);
        let mut groups: HashMap<GroupKey, (IssueSeverity, String, String, Vec<String>)> =
            HashMap::new();
        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                if issue.severity != IssueSeverity::Critical {
                    continue;
                }
                let key: GroupKey = if let Some(ref rid) = issue.rule_id {
                    (Some(rid.clone()), String::new(), String::new())
                } else {
                    (None, issue.category.clone(), issue.recommendation.clone())
                };
                let title = issue
                    .rule_id
                    .as_ref()
                    .and_then(|c| issue_codes::short_title(c).map(String::from))
                    .unwrap_or_else(|| issue.description.clone());
                let entry = groups.entry(key).or_insert_with(|| {
                    (
                        issue.severity.clone(),
                        title,
                        issue.recommendation.clone(),
                        Vec::new(),
                    )
                });
                if severity_ord(&issue.severity) < severity_ord(&entry.0) {
                    entry.0 = issue.severity.clone();
                    entry.1 = issue
                        .rule_id
                        .as_ref()
                        .and_then(|c| issue_codes::short_title(c).map(String::from))
                        .unwrap_or_else(|| issue.description.clone());
                    entry.2 = issue.recommendation.clone();
                }
                if let Some(r) = &issue.resource {
                    entry.3.push(r.clone());
                }
            }
        }
        #[allow(clippy::type_complexity)]
        let mut rows: Vec<(IssueSeverity, Option<String>, String, String, Vec<String>)> = groups
            .into_iter()
            .map(|((rid, _cat, _rec), (sev, title, rec, resources))| {
                (sev, rid, title, rec, resources)
            })
            .collect();
        rows.sort_by(|a, b| {
            let sev_order = |s: &IssueSeverity| match s {
                IssueSeverity::Critical => 0,
                IssueSeverity::Warning => 1,
                IssueSeverity::Info => 2,
            };
            sev_order(&a.0)
                .cmp(&sev_order(&b.0))
                .then_with(|| b.4.len().cmp(&a.4.len()))
        });
        rows.truncate(max_items);
        rows.into_iter()
            .map(|(sev, rule_id, title, rec, resources)| {
                let severity_label = match sev {
                    IssueSeverity::Critical => "Critical",
                    IssueSeverity::Warning => "Warning",
                    IssueSeverity::Info => "Info",
                };
                let n = resources.len();
                let resource_list = format_affected_resources(&resources);
                if let Some(ref code) = rule_id {
                    let doc = issue_codes::doc_path(code);
                    if resource_list.is_empty() {
                        format!(
                            "[{}] **{}** {} ({}). [Doc]({})",
                            severity_label, code, title, n, doc
                        )
                    } else {
                        format!(
                            "[{}] **{}** {} ({}). [Doc]({}). Affected: {}",
                            severity_label, code, title, n, doc, resource_list
                        )
                    }
                } else if resource_list.is_empty() {
                    format!("[{}] {}: Recommendation: {}", severity_label, title, rec)
                } else {
                    format!(
                        "[{}] {} ({} issues): {}. Affected: {}. Recommendation: {}",
                        severity_label, title, n, title, resource_list, rec
                    )
                }
            })
            .collect()
    }

    /// Aggregated key findings for executive summary: error (Critical) level only, no limit.
    fn build_aggregated_findings_error_only(report: &ClusterReport) -> Vec<String> {
        let mut rows = Vec::new();
        type GroupKey = (Option<String>, String, String);
        let mut groups: HashMap<GroupKey, (String, String, Vec<String>)> = HashMap::new();
        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                if issue.severity != IssueSeverity::Critical {
                    continue;
                }
                let key: GroupKey = if let Some(ref rid) = issue.rule_id {
                    (Some(rid.clone()), String::new(), String::new())
                } else {
                    (None, issue.category.clone(), issue.recommendation.clone())
                };
                let title = issue
                    .rule_id
                    .as_ref()
                    .and_then(|c| issue_codes::short_title(c).map(String::from))
                    .unwrap_or_else(|| issue.description.clone());
                let entry = groups
                    .entry(key)
                    .or_insert_with(|| (title, issue.recommendation.clone(), Vec::new()));
                if let Some(r) = &issue.resource {
                    entry.2.push(r.clone());
                }
            }
        }
        let mut rows_vec: Vec<_> = groups
            .into_iter()
            .map(|((rid, _cat, _rec), (title, rec, resources))| (rid, title, rec, resources))
            .collect();
        rows_vec.sort_by(|a, b| b.3.len().cmp(&a.3.len()));
        for (rule_id, title, rec, resources) in rows_vec {
            let n = resources.len();
            let resource_list = format_affected_resources(&resources);
            if let Some(ref code) = rule_id {
                let doc = issue_codes::doc_path(code);
                if resource_list.is_empty() {
                    rows.push(format!(
                        "[error] **{}** {} ({}). [Doc]({})",
                        code, title, n, doc
                    ));
                } else {
                    rows.push(format!(
                        "[error] **{}** {} ({}). [Doc]({}). Affected: {}",
                        code, title, n, doc, resource_list
                    ));
                }
            } else if resource_list.is_empty() {
                rows.push(format!("[error] {}: Recommendation: {}", title, rec));
            } else {
                rows.push(format!(
                    "[error] {} ({} issues): {}. Affected: {}. Recommendation: {}",
                    title, n, title, resource_list, rec
                ));
            }
        }
        rows
    }

    /// Key findings as table rows (error/Critical only): one row per resource (resource, code_link, title).
    /// Issue Code is rendered as a link to the doc; no separate Doc column.
    #[allow(dead_code)]
    fn build_key_findings_table_rows(report: &ClusterReport) -> Vec<(String, String, String)> {
        type GroupKey = (Option<String>, String, String);
        let mut groups: HashMap<GroupKey, (String, String, Vec<String>)> = HashMap::new();
        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                if issue.severity != IssueSeverity::Critical {
                    continue;
                }
                let key: GroupKey = if let Some(ref rid) = issue.rule_id {
                    (Some(rid.clone()), String::new(), String::new())
                } else {
                    (None, issue.category.clone(), issue.recommendation.clone())
                };
                let title = issue
                    .rule_id
                    .as_ref()
                    .and_then(|c| issue_codes::short_title(c).map(String::from))
                    .unwrap_or_else(|| issue.description.clone());
                let entry = groups
                    .entry(key)
                    .or_insert_with(|| (title, issue.recommendation.clone(), Vec::new()));
                if let Some(r) = &issue.resource {
                    entry.2.push(r.clone());
                }
            }
        }
        let mut out: Vec<(String, String, String)> = Vec::new();
        for ((rid, _cat, _), (title, _rec, resources)) in groups {
            let code_link = rid
                .as_ref()
                .map(|c| format!("[{}]({})", c, issue_codes::doc_path(c)))
                .unwrap_or_else(|| "-".to_string());
            if resources.is_empty() {
                out.push(("-".to_string(), code_link, title));
            } else {
                for r in resources {
                    out.push((r, code_link.clone(), title.clone()));
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        out
    }

    /// Group issues by severity; within severity, group by rule_id when present, else by (category, recommendation).
    /// Each group yields (rule_id, title, recommendation, resources). Title is short_title(code) or first description.
    #[allow(clippy::type_complexity)]
    fn group_issues_by_severity_and_type(
        issues: &[Issue],
    ) -> HashMap<IssueSeverity, Vec<(Option<String>, String, String, Vec<String>)>> {
        // Key: when rule_id present use (Some(rule_id), "", ""); else (None, category, recommendation)
        type Key = (Option<String>, String, String);
        #[allow(clippy::type_complexity)]
        let mut by_sev: HashMap<IssueSeverity, HashMap<Key, (String, String, Vec<String>)>> =
            HashMap::new();
        for issue in issues {
            let key: Key = if let Some(ref rid) = issue.rule_id {
                (Some(rid.clone()), String::new(), String::new())
            } else {
                (None, issue.category.clone(), issue.recommendation.clone())
            };
            let entry = by_sev
                .entry(issue.severity.clone())
                .or_default()
                .entry(key)
                .or_insert_with(|| {
                    let title = issue
                        .rule_id
                        .as_ref()
                        .and_then(|c| issue_codes::short_title(c).map(String::from))
                        .unwrap_or_else(|| issue.description.clone());
                    (title, issue.recommendation.clone(), Vec::new())
                });
            if let Some(r) = &issue.resource {
                entry.2.push(r.clone());
            }
        }
        by_sev
            .into_iter()
            .map(|(sev, groups)| {
                let vec: Vec<_> = groups
                    .into_iter()
                    .map(|(k, (title, rec, resources))| (k.0, title, rec, resources))
                    .collect();
                (sev, vec)
            })
            .collect()
    }

    /// Build priority recommendations from error (Critical) issues only; dedup by text, sort by count (desc), take top N.
    fn build_aggregated_recommendations(report: &ClusterReport, max_items: usize) -> Vec<String> {
        let mut rec_counts: HashMap<String, usize> = HashMap::new();
        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                if issue.severity == IssueSeverity::Critical {
                    *rec_counts.entry(issue.recommendation.clone()).or_insert(0) += 1;
                }
            }
        }
        let mut rows: Vec<(String, usize)> = rec_counts.into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1));
        rows.truncate(max_items);
        rows.into_iter().map(|(rec, _)| rec).collect()
    }

    #[allow(dead_code)]
    fn build_statistics_section(&self, report: &ClusterReport) -> String {
        use std::collections::HashMap;

        let mut total_checks: u32 = 0;
        let mut severity_counts: HashMap<IssueSeverity, u32> = HashMap::new();
        let mut category_counts: HashMap<String, u32> = HashMap::new();
        let mut best_module: Option<(&String, f64)> = None;
        let mut worst_module: Option<(&String, f64)> = None;

        for inspection in &report.inspections {
            total_checks += inspection.summary.total_checks;

            let score = inspection.overall_score;
            match best_module {
                Some((_, best_score)) if score > best_score => {
                    best_module = Some((&inspection.inspection_type, score))
                }
                None => best_module = Some((&inspection.inspection_type, score)),
                _ => {}
            }
            match worst_module {
                Some((_, worst_score)) if score < worst_score => {
                    worst_module = Some((&inspection.inspection_type, score))
                }
                None => worst_module = Some((&inspection.inspection_type, score)),
                _ => {}
            }

            for issue in &inspection.summary.issues {
                *severity_counts.entry(issue.severity.clone()).or_insert(0) += 1;
                *category_counts.entry(issue.category.clone()).or_insert(0) += 1;
            }
        }

        let total_issues: u32 = severity_counts.values().sum();
        let mut content = String::new();
        content.push_str("### 📈 Cluster Statistics\n\n");
        content.push_str("| Metric | Value |\n");
        content.push_str("|--------|-------|\n");
        content.push_str(&format!(
            "| Modules Checked | {} |\n",
            report.inspections.len()
        ));
        content.push_str(&format!("| Total Checks | {} |\n", total_checks));
        content.push_str(&format!("| Total Issues | {} |\n", total_issues));
        content.push_str(&format!(
            "| Distinct Resource Categories | {} |\n\n",
            category_counts.len()
        ));

        if total_issues > 0 {
            content.push_str("| Severity | Count | Ratio |\n");
            content.push_str("|----------|-------|-------|\n");
            let severities = [
                IssueSeverity::Critical,
                IssueSeverity::Warning,
                IssueSeverity::Info,
            ];
            for severity in &severities {
                if let Some(count) = severity_counts.get(severity) {
                    let label = match severity {
                        IssueSeverity::Critical => "Critical",
                        IssueSeverity::Warning => "Warning",
                        IssueSeverity::Info => "Info",
                    };
                    content.push_str(&format!(
                        "| {} | {} | {:.1}% |\n",
                        label,
                        count,
                        (*count as f64 / total_issues as f64) * 100.0
                    ));
                }
            }
            content.push('\n');
        }

        if !category_counts.is_empty() {
            let mut top_categories: Vec<(String, u32)> = category_counts.into_iter().collect();
            top_categories.sort_by(|a, b| b.1.cmp(&a.1));
            top_categories.truncate(5);
            content.push_str("**Top Resource Categories by Issue Count (Top 5)**\n\n");
            for (category, count) in top_categories {
                content.push_str(&format!("- {}: {} issues\n", category, count));
            }
            content.push('\n');
        }

        if let Some((module, score)) = best_module {
            content.push_str(&format!(
                "**Best Module**: {} ({:.1} points)\n\n",
                module, score
            ));
        }
        if let Some((module, score)) = worst_module {
            content.push_str(&format!(
                "**Worst Module**: {} ({:.1} points)\n\n",
                module, score
            ));
        }

        content
    }

    #[allow(dead_code)]
    fn node_inspection_status(n: &NodeInspectionResult) -> &'static str {
        let has_error = n.resources.status == "error"
            || n.services.status == "error"
            || n.security.status == "error"
            || n.kernel.status == "error";
        let has_warning = n.resources.status == "warning"
            || n.services.status == "warning"
            || n.security.status == "warning"
            || n.kernel.status == "warning";
        if has_error {
            "error"
        } else if has_warning || n.issue_count > 0 {
            "warning"
        } else {
            "ok"
        }
    }

    /// Renders Node Inspection section: Summary table + Node General Information + Node resources / services / security / kernel / certificates tables.
    fn format_node_inspection_section(report: &ClusterReport) -> String {
        let nodes = report.node_inspection_results.as_deref().unwrap_or(&[]);
        let node_address_map: HashMap<String, String> = report
            .cluster_overview
            .as_ref()
            .and_then(|o| o.node_list.as_ref())
            .map(|list| {
                list.iter()
                    .filter_map(|r| r.node_address.as_ref().map(|a| (r.name.clone(), a.clone())))
                    .collect()
            })
            .unwrap_or_default();

        let node_api_os_kernel: HashMap<String, (Option<String>, Option<String>)> = report
            .cluster_overview
            .as_ref()
            .and_then(|o| o.node_list.as_ref())
            .map(|list| {
                list.iter()
                    .map(|r| {
                        (
                            r.name.clone(),
                            (r.os_image.clone(), r.kernel_version.clone()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let _node_api_runtime: HashMap<String, String> = report
            .cluster_overview
            .as_ref()
            .and_then(|o| o.node_list.as_ref())
            .map(|list| {
                list.iter()
                    .filter_map(|r| {
                        r.container_runtime_version
                            .as_ref()
                            .filter(|s| !s.is_empty())
                            .map(|v| (r.name.clone(), v.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Node Status (Ready/NotReady/Unknown) from node_conditions or node_list.ready (unused when Node services table is service×node)
        let _node_status: HashMap<String, &'static str> = report
            .cluster_overview
            .as_ref()
            .and_then(|o| o.node_conditions.as_ref())
            .map(|rows| {
                rows.iter()
                    .map(|r| {
                        let s = match r.ready.as_str() {
                            "True" => "Ready",
                            "False" => "NotReady",
                            _ => "Unknown",
                        };
                        (r.node_name.clone(), s)
                    })
                    .collect()
            })
            .or_else(|| {
                report
                    .cluster_overview
                    .as_ref()
                    .and_then(|o| o.node_list.as_ref())
                    .map(|list| {
                        list.iter()
                            .map(|r| (r.name.clone(), if r.ready { "Ready" } else { "NotReady" }))
                            .collect()
                    })
            })
            .unwrap_or_default();

        let mut out = String::new();
        out.push_str("## Node Inspection\n\n");
        out.push_str("Per-node checks from kubeowler-node-inspector DaemonSet.\n\n");

        // (0) Node General Information: Node | OS Version | IP Address | Kernel Version | Uptime (API preferred for OS/Kernel when available)
        out.push_str("### Node General Information\n\n");
        out.push_str("| Node | OS Version | IP Address | Kernel Version | Uptime |\n");
        out.push_str("|------|-------------|------------|----------------|--------|\n");
        for n in nodes {
            let (api_os, api_kernel) = node_api_os_kernel
                .get(&n.node_name)
                .cloned()
                .unwrap_or((None, None));
            let os_ver = api_os.as_deref().or(n.os_version.as_deref()).unwrap_or("-");
            let ip = node_address_map
                .get(&n.node_name)
                .map(|s| s.as_str())
                .unwrap_or("-");
            let kernel = api_kernel
                .as_deref()
                .or(n.kernel_version.as_deref())
                .unwrap_or("-");
            let uptime = n.uptime.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                n.node_name, os_ver, ip, kernel, uptime
            ));
        }
        out.push('\n');

        // (1) Node resources: CPU, Mem, Swap, Load (CPU Used/CPU % placeholder "-" until script provides)
        out.push_str("### Node resources\n\n");
        out.push_str("| Node | CPU (cores) | CPU Used | CPU % | Mem Total (Gi) | Mem Used (Gi) | Mem % | Swap Total (Gi) | Swap Used (Gi) | Swap % | Load (1m, 5m, 15m) |\n");
        out.push_str("|------|-------------|----------|-------|----------------|---------------|-------|----------------|---------------|-------|---------------------|\n");
        for n in nodes {
            let cpu = n
                .resources
                .cpu_cores
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            let cpu_used = n
                .resources
                .cpu_used
                .map(|u| format!("{:.2}", u))
                .unwrap_or_else(|| "-".to_string());
            let cpu_pct = n
                .resources
                .cpu_used_pct
                .map(|p| format!("{:.1}%", p))
                .unwrap_or_else(|| "-".to_string());
            let mem_total_g = n
                .resources
                .memory_total_mib
                .map(|m| format!("{:.1}", m as f64 / 1024.0))
                .unwrap_or_else(|| "-".to_string());
            let mem_used_g = n
                .resources
                .memory_used_mib
                .map(|m| format!("{:.1}", m as f64 / 1024.0))
                .unwrap_or_else(|| "-".to_string());
            let mem_pct = n
                .resources
                .memory_used_pct
                .map(|p| format!("{:.1}%", p))
                .unwrap_or_else(|| "-".to_string());
            let swap_total_g = n
                .resources
                .swap_total_g
                .map(|g| format!("{:.2}", g))
                .unwrap_or_else(|| "-".to_string());
            let swap_used_g = n
                .resources
                .swap_used_g
                .map(|g| format!("{:.2}", g))
                .unwrap_or_else(|| "-".to_string());
            let swap_pct = n
                .resources
                .swap_used_pct
                .map(|p| format!("{:.1}%", p))
                .unwrap_or_else(|| "-".to_string());
            let load_1m = n.resources.load_1m.as_deref().unwrap_or("-");
            let load_5m = n.resources.load_5m.as_deref().unwrap_or("-");
            let load_15m = n.resources.load_15m.as_deref().unwrap_or("-");
            let load_merged = format!("{}, {}, {}", load_1m, load_5m, load_15m);
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                n.node_name,
                cpu,
                cpu_used,
                cpu_pct,
                mem_total_g,
                mem_used_g,
                mem_pct,
                swap_total_g,
                swap_used_g,
                swap_pct,
                load_merged
            ));
        }
        out.push('\n');

        // (1a) Node disk usage: per node show top 3 by used% + all with used% > 60%; Status: Info (<60%), Warning (60–90%), Critical (>=90%)
        out.push_str("### Node disk usage\n\n");
        out.push_str("Per-node filesystem usage by mount. Status: Info (<60% used), Warning (60–90%), Critical (≥90%).\n\n");
        out.push_str(
            "| Node | Mount Point | Device | FSType | Total (Gi) | Used (Gi) | Used % | Status |\n",
        );
        out.push_str("|------|-------------|--------|--------|------------|------------|--------|--------|\n");
        let node_004_link = format!("[NODE-004]({})", issue_codes::doc_path("NODE-004"));
        let node_005_link = format!("[NODE-005]({})", issue_codes::doc_path("NODE-005"));
        for n in nodes {
            let disks = n.node_disks.as_deref().unwrap_or(&[]);
            if disks.is_empty() {
                out.push_str(&format!(
                    "| {} | - | - | - | - | - | - | - |\n",
                    n.node_name
                ));
            } else {
                let mut order: Vec<usize> = (0..disks.len()).collect();
                order.sort_by(|&i, &j| {
                    let a = disks[i].used_pct.unwrap_or(0.0);
                    let b = disks[j].used_pct.unwrap_or(0.0);
                    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
                });
                let mut to_show_idx: std::collections::HashSet<usize> =
                    order.iter().take(3).copied().collect();
                for (idx, d) in disks.iter().enumerate() {
                    if d.used_pct.map(|p| p > 60.0).unwrap_or(false) {
                        to_show_idx.insert(idx);
                    }
                }
                let mut to_show: Vec<usize> = to_show_idx.into_iter().collect();
                to_show.sort_by(|&i, &j| {
                    let a = disks[i].used_pct.unwrap_or(0.0);
                    let b = disks[j].used_pct.unwrap_or(0.0);
                    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
                });
                for i in to_show {
                    let d = &disks[i];
                    let total_g = d
                        .total_g
                        .map(|g| format!("{:.1}", g))
                        .unwrap_or_else(|| "-".to_string());
                    let used_g = d
                        .used_g
                        .map(|g| format!("{:.1}", g))
                        .unwrap_or_else(|| "-".to_string());
                    let used_pct_str = d
                        .used_pct
                        .map(|p| format!("{:.1}%", p))
                        .unwrap_or_else(|| "-".to_string());
                    let status = match d.used_pct {
                        Some(p) if p >= 90.0 => format!("Critical {}", node_005_link),
                        Some(p) if p >= 60.0 => format!("Warning {}", node_004_link),
                        Some(_) => "Info".to_string(),
                        None => "-".to_string(),
                    };
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                        n.node_name,
                        if d.mount_point.is_empty() {
                            "-".to_string()
                        } else {
                            d.mount_point.clone()
                        },
                        if d.device.is_empty() {
                            "-".to_string()
                        } else {
                            d.device.clone()
                        },
                        if d.fstype.is_empty() {
                            "-".to_string()
                        } else {
                            d.fstype.clone()
                        },
                        total_g,
                        used_g,
                        used_pct_str,
                        status
                    ));
                }
            }
        }
        out.push('\n');

        // (1b) Node container state counts: Node | Running | Waiting | Exited
        out.push_str("### Node container state counts\n\n");
        out.push_str("| Node | Running | Waiting | Exited |\n");
        out.push_str("|------|---------|---------|--------|\n");
        for n in nodes {
            let counts = n.container_state_counts.as_ref();
            let running = counts.and_then(|c| c.get("running")).copied().unwrap_or(0);
            let waiting = counts.and_then(|c| c.get("waiting")).copied().unwrap_or(0);
            let exited = counts.and_then(|c| c.get("exited")).copied().unwrap_or(0);
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                n.node_name, running, waiting, exited
            ));
        }
        out.push('\n');

        // (2) Node services: rows = nodes, columns = Node/Service, NTP synced, Journald, Crontab; cell = enabled/disabled/None
        out.push_str("## Node service status\n\n");
        fn service_cell(v: Option<bool>) -> &'static str {
            match v {
                Some(true) => "enabled",
                Some(false) => "disabled",
                None => "None",
            }
        }
        out.push_str("| Node/Service | NTP synced | Journald | Crontab |\n");
        out.push_str("|------|------------|----------|----------|\n");
        for n in nodes {
            let ntp = service_cell(n.services.ntp_synced);
            let journald = service_cell(n.services.journald_active);
            let crontab = service_cell(n.services.crontab_present);
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                n.node_name, ntp, journald, crontab
            ));
        }
        out.push('\n');

        // (3) Node security: Node | SELinux | Firewalld | IPVS loaded
        out.push_str("### Node security\n\n");
        out.push_str("| Node | SELinux | Firewalld | IPVS loaded |\n");
        out.push_str("|------|---------|------------|-------------|\n");
        for n in nodes {
            let fw = n
                .security
                .firewalld_active
                .map(|b| if b { "Active" } else { "Inactive" })
                .unwrap_or("-");
            let ipvs = n
                .security
                .ipvs_loaded
                .map(|b| if b { "Yes" } else { "No" })
                .unwrap_or("-");
            let se = n.security.selinux.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                n.node_name, se, fw, ipvs
            ));
        }
        out.push('\n');

        // (4) Node kernel parameters: Node | net.ipv4.ip_forward | vm.swappiness | net.core.somaxconn
        out.push_str("### Node kernel parameters\n\n");
        out.push_str("| Node | net.ipv4.ip_forward | vm.swappiness | net.core.somaxconn |\n");
        out.push_str("|------|---------------------|--------------|--------------------|\n");
        for n in nodes {
            let fwd = n.kernel.net_ipv4_ip_forward.as_deref().unwrap_or("-");
            let sw = n.kernel.vm_swappiness.as_deref().unwrap_or("-");
            let somax = n.kernel.net_core_somaxconn.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                n.node_name, fwd, sw, somax
            ));
        }
        out.push('\n');

        // (5) Node process health: Node | Zombie count | Issue code (NODE-003 when > 0)
        out.push_str("### Node process health\n\n");
        out.push_str("| Node | Zombie count | Issue code |\n");
        out.push_str("|------|------------|----------|\n");
        let node_003_link = format!("[NODE-003]({})", issue_codes::doc_path("NODE-003"));
        for n in nodes {
            let z = n.zombie_count.unwrap_or(0);
            let code_cell = if z > 0 { node_003_link.as_str() } else { "-" };
            out.push_str(&format!("| {} | {} | {} |\n", n.node_name, z, code_cell));
        }
        out.push('\n');

        // (6) Node Certificate Status: Node | Path | Expired | Expiration Date | Days to Expiry
        out.push_str("### Node Certificate Status\n\n");
        out.push_str("| Node | Path | Expired | Expiration Date | Days to Expiry |\n");
        out.push_str("|------|------|---------|-----------------|----------------|\n");
        for n in nodes {
            let certs = n.node_certificates.as_deref().unwrap_or(&[]);
            if certs.is_empty() {
                out.push_str(&format!("| {} | - | - | - | - |\n", n.node_name));
            } else {
                for c in certs {
                    let expired = if c.status == "Expired" { "Yes" } else { "No" };
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        n.node_name, c.path, expired, c.expiration_date, c.days_remaining
                    ));
                }
            }
        }
        out.push('\n');

        out
    }

    fn generate_main_report(
        &self,
        report: &ClusterReport,
        max_recommendations: Option<usize>,
        check_level_filter: Option<CheckLevelFilter>,
    ) -> Result<String> {
        let _max_r = max_recommendations.unwrap_or(DEFAULT_MAX_RECOMMENDATIONS);
        let check_filter = check_level_filter.unwrap_or(CheckLevelFilter::Only(vec![
            CheckStatus::Warning,
            CheckStatus::Critical,
            CheckStatus::Error,
        ]));
        let mut content = String::new();

        // Header (title includes cluster name)
        content.push_str(&format!(
            "# {} Kubernetes Cluster Check Report\n\n",
            report.cluster_name
        ));

        content.push_str(&format!("**Report ID**: `{}`\n\n", report.report_id));

        content.push_str(&format!("**Cluster**: {}\n\n", report.cluster_name));

        content.push_str(&format!(
            "**Generated At**: {}\n\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Cluster Overview: always output section (placeholder if no data); core metrics in table
        content.push_str("## 🖥️ Cluster Overview\n\n");
        if let Some(ref overview) = report.cluster_overview {
            content.push_str("| Metric | Value |\n");
            content.push_str("|--------|-------|\n");
            if let Some(ref v) = overview.cluster_version {
                content.push_str(&format!("| Cluster Version | {} |\n", v));
            }
            content.push_str(&format!("| Node Count | {} |\n", overview.node_count));
            content.push_str(&format!(
                "| Ready Nodes | {} |\n",
                overview.ready_node_count
            ));
            if let Some(pc) = overview.pod_count {
                content.push_str(&format!("| Pod Count | {} |\n", pc));
            }
            if let Some(nc) = overview.namespace_count {
                content.push_str(&format!("| Namespace Count | {} |\n", nc));
            }
            if let Some(age) = overview.cluster_age_days {
                content.push_str(&format!("| Cluster Age (days) | {} |\n", age));
            }
            if let Some(ref node_list) = overview.node_list {
                let runtimes: std::collections::HashSet<&str> = node_list
                    .iter()
                    .filter_map(|r| r.container_runtime_version.as_deref())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !runtimes.is_empty() {
                    let rt_str: Vec<&str> = runtimes.into_iter().collect();
                    content.push_str(&format!("| Container Runtime | {} |\n", rt_str.join(", ")));
                }
            }
            let health_emoji = match report.executive_summary.health_status {
                HealthStatus::Excellent => "🟢",
                HealthStatus::Good => "🟡",
                HealthStatus::Fair => "🟠",
                HealthStatus::Poor => "🔴",
                HealthStatus::Critical => "🚨",
            };
            let health_text = match report.executive_summary.health_status {
                HealthStatus::Excellent => "Excellent",
                HealthStatus::Good => "Good",
                HealthStatus::Fair => "Fair",
                HealthStatus::Poor => "Poor",
                HealthStatus::Critical => "Critical",
            };
            content.push_str(&format!(
                "| Overall Health | {} {} (Score: {:.1}) |\n",
                health_emoji, health_text, report.overall_score
            ));
            content.push('\n');
            if let Some(ref conds) = overview.node_conditions {
                if !conds.is_empty() {
                    content.push_str("### Node conditions\n\n");
                    content.push_str(
                        "| Node | Ready | MemoryPressure | DiskPressure | PIDPressure |\n",
                    );
                    content.push_str(
                        "|------|-------|----------------|--------------|-------------|\n",
                    );
                    for r in conds {
                        content.push_str(&format!(
                            "| {} | {} | {} | {} | {} |\n",
                            r.node_name,
                            r.ready,
                            r.memory_pressure,
                            r.disk_pressure,
                            r.pid_pressure
                        ));
                    }
                    content.push('\n');
                }
            }
            // Workload summary
            if let Some(ref wl) = overview.workload_summary {
                content.push_str("### Workload summary\n\n");
                content.push_str("| Controller | Total | Ready |\n");
                content.push_str("|------------|-------|-------|\n");
                content.push_str(&format!(
                    "| Deployment | {} | {} |\n",
                    wl.deployments_total, wl.deployments_ready
                ));
                content.push_str(&format!(
                    "| StatefulSet | {} | {} |\n",
                    wl.statefulsets_total, wl.statefulsets_ready
                ));
                content.push_str(&format!(
                    "| DaemonSet | {} | {} |\n\n",
                    wl.daemonsets_total, wl.daemonsets_ready
                ));
            }
            // Storage summary
            if let Some(ref st) = overview.storage_summary {
                content.push_str("### Storage summary\n\n");
                content.push_str("| Metric | Value |\n");
                content.push_str("|--------|-------|\n");
                content.push_str(&format!("| PV total | {} |\n", st.pv_total));
                content.push_str(&format!("| PVC total | {} |\n", st.pvc_total));
                content.push_str(&format!("| PVC Bound | {} |\n", st.pvc_bound));
                content.push_str(&format!(
                    "| StorageClass count | {} |\n",
                    st.storage_class_count
                ));
                content.push_str(&format!(
                    "| Default StorageClass | {} |\n\n",
                    if st.has_default_storage_class {
                        "Yes"
                    } else {
                        "No"
                    }
                ));
            }
            // Container resource usage: top 20 high usage (usage/limit >= 80%); shown only when metrics available
            if overview.metrics_available == Some(true) {
                if let Some(ref rows) = overview.container_usage_notable {
                    if !rows.is_empty() {
                        content.push_str("### Container resource usage (top 20 high usage)\n\n");
                        content.push_str("Top 20 containers by usage vs limit (CPU or memory ≥ 80% of limit). Data from **metrics-server** (Pod metrics API) and **Pod spec** (limits). This section is **omitted when metrics-server is unavailable**.\n\n");
                        content.push_str("| Namespace | Pod | Container | CPU used (m) | CPU request (m) | CPU limit (m) | Mem used (Mi) | Mem request (Mi) | Mem limit (Mi) | Note |\n");
                        content.push_str("|-----------|-----|-----------|--------------|-----------------|---------------|---------------|------------------|----------------|------|\n");
                        for r in rows {
                            let note = match r.notable_reason.as_str() {
                                "high_usage" => "High usage",
                                "low_usage" => "Low usage",
                                "no_request_no_limit" => "No request",
                                _ => r.notable_reason.as_str(),
                            };
                            content.push_str(&format!(
                                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                                r.namespace,
                                r.pod_name,
                                r.container_name,
                                r.cpu_used_m,
                                r.cpu_request_m,
                                r.cpu_limit_m,
                                r.mem_used_mib,
                                r.mem_request_mib,
                                r.mem_limit_mib,
                                note
                            ));
                        }
                        content.push('\n');
                    }
                }
            }
        } else {
            content.push_str("Cluster overview is not available (ensure cluster is reachable and the tool has been rebuilt).\n\n");
        }

        // Node Inspection (from DaemonSet): Summary + category tables, or placeholder when no data
        match report.node_inspection_results.as_deref() {
            Some(nodes) if !nodes.is_empty() => {
                content.push_str(&Self::format_node_inspection_section(report));
            }
            _ => {
                content.push_str("## Node Inspection\n\n");
                content.push_str("No data (kubeowler-node-inspector DaemonSet not deployed or exec failed / no pods ready).\n\n");
            }
        }

        // Recent cluster events (Warning / Error only)
        if let Some(ref events) = report.recent_events {
            if !events.is_empty() {
                content.push_str("## Recent cluster events (Warning / Error)\n\n");
                content.push_str("| Namespace | Object | Level | Reason | Message | Last seen |\n");
                content.push_str("|-----------|--------|-------|--------|---------|----------|\n");
                for e in events {
                    let level = match e.event_type.as_str() {
                        "Error" => "Critical",
                        "Warning" => "Warning",
                        "Normal" => "Info",
                        _ => e.event_type.as_str(),
                    };
                    content.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} |\n",
                        e.namespace,
                        e.object_ref,
                        level,
                        e.reason,
                        truncate_string(&e.message, 60),
                        e.last_seen
                    ));
                }
                content.push('\n');
            }
        }

        // Detailed results grouped by Kubernetes resource object
        content.push_str("## 📋 Detailed Results\n\n");

        // Check Results: first column = cluster resource object; filter by check level (default: exclude Pass)
        content.push_str("### Check Results\n\n");
        content.push_str("| Resource | Check Item | Status | Score | Details |\n");
        content.push_str("|----------|------------|--------|-------|----------|\n");
        const DETAILS_MAX_LEN: usize = 60;
        for inspection in &report.inspections {
            let resource = inspection_type_to_resource(&inspection.inspection_type);
            for check in &inspection.checks {
                let include = match &check_filter {
                    CheckLevelFilter::All => true,
                    CheckLevelFilter::Only(list) => list.contains(&check.status),
                };
                if !include {
                    continue;
                }
                let status_text = match check.status {
                    CheckStatus::Pass => "✅ Pass",
                    CheckStatus::Warning => "⚠️ Warning",
                    CheckStatus::Critical => "❌ Critical",
                    CheckStatus::Error => "💥 Error",
                };
                let details_str = check.details.as_deref().unwrap_or("-");
                let details_short = truncate_string(details_str, DETAILS_MAX_LEN);
                content.push_str(&format!(
                    "| {} | {} | {} | {:.1}/{:.1} | {} |\n",
                    resource, check.name, status_text, check.score, check.max_score, details_short
                ));
            }
        }
        content.push('\n');

        // Namespace summary table (from Namespace inspection)
        if let Some(rows) = report
            .inspections
            .iter()
            .find_map(|i| i.namespace_summary_rows.as_ref().filter(|v| !v.is_empty()))
        {
            content.push_str("### Namespace summary\n\n");
            content.push_str(
                "| Namespace | Pods | Deployments | NetworkPolicy | ResourceQuota | LimitRange |\n",
            );
            content.push_str(
                "|-----------|------|-------------|---------------|---------------|------------|\n",
            );
            for r in rows {
                content.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    r.name,
                    r.pod_count,
                    r.deployment_count,
                    if r.has_network_policy { "Yes" } else { "No" },
                    if r.has_resource_quota { "Yes" } else { "No" },
                    if r.has_limit_range { "Yes" } else { "No" },
                ));
            }
            content.push('\n');
        }

        // Per-resource sections: only emit if at least one issue or one detail block (Pod container state table omitted)
        let by_resource = group_issues_by_resource(report);
        let cert_expiries = report.inspections.iter().find_map(|i| {
            i.certificate_expiries
                .as_ref()
                .filter(|v| !v.is_empty())
                .map(|v| v.as_slice())
        });

        for &resource in REPORT_RESOURCE_ORDER {
            let issues = by_resource
                .get(resource)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let has_cert_expiries = resource == "Certificate" && cert_expiries.is_some();
            if issues.is_empty() && !has_cert_expiries {
                continue;
            }
            let slug = slugify(resource);
            content.push_str(&format!("<a id=\"{}\"></a>\n\n", slug));
            content.push_str(&format!("### {}\n\n", resource));
            if has_cert_expiries {
                if let Some(expiries) = cert_expiries {
                    content.push_str("#### TLS Certificate Expiry\n\n");
                    content.push_str("| Secret (namespace/name) | Certificate (subject) | Expiry (UTC) | Days until expiry | Level | Issue Code |\n");
                    content.push_str("|--------------------------|-----------------------|--------------|-------------------|-------|------------|\n");
                    for row in expiries {
                        let (level, code_link) = if row.days_until_expiry < 0 {
                            (
                                "Critical",
                                format!("[CERT-003]({})", issue_codes::doc_path("CERT-003")),
                            )
                        } else if row.days_until_expiry <= 30 {
                            (
                                "Warning",
                                format!("[CERT-002]({})", issue_codes::doc_path("CERT-002")),
                            )
                        } else {
                            (
                                "Info",
                                format!("[CERT-002]({})", issue_codes::doc_path("CERT-002")),
                            )
                        };
                        let secret_cell = format!("{}/{}", row.secret_namespace, row.secret_name);
                        content.push_str(&format!(
                            "| {} | {} | {} | {} | {} | {} |\n",
                            secret_cell,
                            truncate_string(&row.subject_or_cn, 50),
                            row.expiry_utc,
                            row.days_until_expiry,
                            level,
                            code_link
                        ));
                    }
                    content.push('\n');
                }
            }
            if !issues.is_empty() {
                content.push_str("| Resource | Level | Issue Code | Short Title |\n");
                content.push_str("|----------|-------|------------|-------------|\n");
                let grouped = Self::group_issues_by_severity_and_type(issues);
                let severity_to_level = |s: &IssueSeverity| -> &'static str {
                    match s {
                        IssueSeverity::Critical => "Critical",
                        IssueSeverity::Warning => "Warning",
                        IssueSeverity::Info => "Info",
                    }
                };
                for sev in &[
                    IssueSeverity::Critical,
                    IssueSeverity::Warning,
                    IssueSeverity::Info,
                ] {
                    // Default: only Warning and Critical (exclude Info). With --check-level all, show Info too.
                    if matches!(sev, IssueSeverity::Info)
                        && !matches!(&check_filter, CheckLevelFilter::All)
                    {
                        continue;
                    }
                    let level = severity_to_level(sev);
                    if let Some(groups) = grouped.get(sev) {
                        for (rule_id, title, _rec, resources) in groups {
                            let code_link = rule_id
                                .as_ref()
                                .map(|c| format!("[{}]({})", c, issue_codes::doc_path(c)))
                                .unwrap_or_else(|| "-".to_string());
                            if resources.is_empty() {
                                content.push_str(&format!(
                                    "| {} | {} | {} | {} |\n",
                                    resource, level, code_link, title
                                ));
                            } else {
                                for r in resources {
                                    content.push_str(&format!(
                                        "| `{}` | {} | {} | {} |\n",
                                        r, level, code_link, title
                                    ));
                                }
                            }
                        }
                    }
                }
                content.push('\n');
            }
            content.push_str("---\n\n");
        }

        // Footer
        content.push_str("---\n\n");
        content.push_str(
            "*Report generated by [kubeowler](https://github.com/username/kubeowler).*\n",
        );

        Ok(content)
    }

    fn generate_summary_report(&self, report: &ClusterReport) -> Result<String> {
        let mut content = String::new();

        content.push_str("# Cluster Inspection – Exception Summary\n\n");

        content.push_str(&format!("**Cluster**: {}\n\n", report.cluster_name));

        content.push_str(&format!(
            "**Generated At**: {}\n\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Group by 3 severities
        let mut critical_issues = Vec::new();
        let mut warning_issues = Vec::new();
        let mut info_issues = Vec::new();

        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                match issue.severity {
                    IssueSeverity::Critical => critical_issues.push((inspection, issue)),
                    IssueSeverity::Warning => warning_issues.push((inspection, issue)),
                    IssueSeverity::Info => info_issues.push((inspection, issue)),
                }
            }
        }

        // Summary statistics
        content.push_str("## Issue Statistics\n\n");
        content.push_str("| Severity | Count | Ratio |\n");
        content.push_str("|----------|-------|-------|\n");

        let total_issues = critical_issues.len() + warning_issues.len() + info_issues.len();

        if total_issues > 0 {
            content.push_str(&format!(
                "| Critical | {} | {:.1}% |\n",
                critical_issues.len(),
                (critical_issues.len() as f64 / total_issues as f64) * 100.0
            ));
            content.push_str(&format!(
                "| Warning | {} | {:.1}% |\n",
                warning_issues.len(),
                (warning_issues.len() as f64 / total_issues as f64) * 100.0
            ));
            content.push_str(&format!(
                "| Info | {} | {:.1}% |\n",
                info_issues.len(),
                (info_issues.len() as f64 / total_issues as f64) * 100.0
            ));
        }
        content.push('\n');

        // Critical: one table
        let critical_flat: Vec<_> = critical_issues.iter().map(|(_, i)| (*i).clone()).collect();
        let critical_grouped = Self::group_issues_by_severity_and_type(&critical_flat);

        if let Some(groups) = critical_grouped.get(&IssueSeverity::Critical) {
            content.push_str("## Critical Issues\n\n");
            content.push_str("> Immediate action required.\n\n");
            content.push_str("| Resource | Issue Code | Short Title |\n");
            content.push_str("|----------|------------|-------------|\n");
            for (rule_id, title, _rec, resources) in groups {
                let code_link = rule_id
                    .as_ref()
                    .map(|c| format!("[{}]({})", c, issue_codes::doc_path(c)))
                    .unwrap_or_else(|| "-".to_string());
                if resources.is_empty() {
                    content.push_str(&format!("| - | {} | {} |\n", code_link, title));
                } else {
                    for r in resources {
                        content.push_str(&format!("| `{}` | {} | {} |\n", r, code_link, title));
                    }
                }
            }
            content.push('\n');
        }

        // Warning and Info: single "Other Issues" table
        if !warning_issues.is_empty() || !info_issues.is_empty() {
            content.push_str("## Other Issues\n\n");
            content.push_str(
                "| Code | Severity | Category | Count | Sample Resource | Recommendation |\n",
            );
            content.push_str(
                "|------|----------|----------|-------|-----------------|----------------|\n",
            );

            let warning_groups = Self::group_issues_for_summary_table_with_code(&warning_issues);
            for (code, cat, rec, count, sample) in warning_groups {
                let sample_short = truncate_string(&sample, 40);
                content.push_str(&format!(
                    "| {} | Warning | {} | {} | {} | {} |\n",
                    code,
                    cat,
                    count,
                    sample_short,
                    truncate_string(&rec, 50)
                ));
            }
            let info_groups = Self::group_issues_for_summary_table_with_code(&info_issues);
            for (code, cat, rec, count, sample) in info_groups {
                let sample_short = truncate_string(&sample, 40);
                content.push_str(&format!(
                    "| {} | Info | {} | {} | {} | {} |\n",
                    code,
                    cat,
                    count,
                    sample_short,
                    truncate_string(&rec, 50)
                ));
            }
            content.push('\n');
        }

        // Recommendations by category: sort by issue count, show "N issues" per recommendation
        content.push_str("## 🎯 Recommendations by Category\n\n");

        let mut category_rec_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for inspection in &report.inspections {
            for issue in &inspection.summary.issues {
                let rec_map = category_rec_counts
                    .entry(issue.category.clone())
                    .or_default();
                *rec_map.entry(issue.recommendation.clone()).or_insert(0) += 1;
            }
        }
        let mut category_totals: Vec<(String, usize)> = category_rec_counts
            .iter()
            .map(|(cat, rec_map)| (cat.clone(), rec_map.values().sum()))
            .collect();
        category_totals.sort_by(|a, b| b.1.cmp(&a.1));

        for (category, _total) in category_totals {
            if let Some(rec_map) = category_rec_counts.get(&category) {
                let mut rec_list: Vec<(String, usize)> =
                    rec_map.iter().map(|(r, c)| (r.clone(), *c)).collect();
                rec_list.sort_by(|a, b| b.1.cmp(&a.1));
                content.push_str(&format!("### {}\n\n", category));
                for (recommendation, count) in rec_list {
                    content.push_str(&format!("- {} ({} issues)\n", recommendation, count));
                }
                content.push('\n');
            }
        }

        Ok(content)
    }

    /// Group issues by (category, recommendation), return (category, recommendation, count, sample_resource).
    #[allow(dead_code)]
    fn group_issues_for_summary_table(
        issues: &[(&InspectionResult, &Issue)],
    ) -> Vec<(String, String, usize, String)> {
        let mut groups: HashMap<(String, String), (usize, String)> = HashMap::new();
        for (_inspection, issue) in issues {
            let key = (issue.category.clone(), issue.recommendation.clone());
            let entry = groups.entry(key).or_insert((0, String::new()));
            entry.0 += 1;
            if entry.1.is_empty() {
                entry.1 = issue.resource.clone().unwrap_or_default();
            }
        }
        groups
            .into_iter()
            .map(|((cat, rec), (count, sample))| (cat, rec, count, sample))
            .collect()
    }

    /// Like group_issues_for_summary_table but includes issue code (rule_id or "-"). Returns (code, category, recommendation, count, sample).
    fn group_issues_for_summary_table_with_code(
        issues: &[(&InspectionResult, &Issue)],
    ) -> Vec<(String, String, String, usize, String)> {
        let mut groups: HashMap<(Option<String>, String, String), (usize, String)> = HashMap::new();
        for (_inspection, issue) in issues {
            let key = (
                issue.rule_id.clone(),
                issue.category.clone(),
                issue.recommendation.clone(),
            );
            let entry = groups.entry(key).or_insert((0, String::new()));
            entry.0 += 1;
            if entry.1.is_empty() {
                entry.1 = issue.resource.clone().unwrap_or_default();
            }
        }
        groups
            .into_iter()
            .map(|((code, cat, rec), (count, sample))| {
                let code_str = code.as_deref().unwrap_or("-").to_string();
                (code_str, cat, rec, count, sample)
            })
            .collect()
    }

    #[allow(dead_code)]
    fn format_inspection_result(&self, inspection: &InspectionResult) -> Result<String> {
        let mut content = String::new();

        let slug = slugify(&inspection.inspection_type);
        content.push_str(&format!("<a id=\"{}\"></a>\n\n", slug));
        content.push_str(&format!(
            "### {} (Score: {:.1}/100)\n\n",
            inspection.inspection_type, inspection.overall_score
        ));

        // Summary
        content.push_str(&format!(
            "**Check Items**: {} | **Pass**: {} | **Warning**: {} | **Critical**: {} | **Error**: {}\n\n",
            inspection.summary.total_checks,
            inspection.summary.passed_checks,
            inspection.summary.warning_checks,
            inspection.summary.critical_checks,
            inspection.summary.error_checks
        ));

        // Check results
        content.push_str("#### Check Results\n\n");
        content.push_str("| Check Item | Status | Score | Details |\n");
        content.push_str("|------------|--------|-------|----------|\n");

        const DETAILS_MAX_LEN: usize = 60;
        for check in &inspection.checks {
            let status_text = match check.status {
                CheckStatus::Pass => "✅ Pass",
                CheckStatus::Warning => "⚠️ Warning",
                CheckStatus::Critical => "❌ Critical",
                CheckStatus::Error => "💥 Error",
            };
            let details_str = check.details.as_deref().unwrap_or("-");
            let details_short = truncate_string(details_str, DETAILS_MAX_LEN);

            content.push_str(&format!(
                "| {} | {} | {:.1}/{:.1} | {} |\n",
                check.name, status_text, check.score, check.max_score, details_short
            ));
        }
        content.push('\n');

        // TLS certificate expiry table (Certificates inspection only)
        if let Some(ref expiries) = inspection.certificate_expiries {
            if !expiries.is_empty() {
                content.push_str("#### TLS Certificate Expiry\n\n");
                content.push_str("| Secret (namespace/name) | Certificate (subject) | Expiry (UTC) | Days until expiry | Level | Issue Code |\n");
                content.push_str("|--------------------------|-----------------------|--------------|-------------------|-------|------------|\n");
                for row in expiries {
                    let (level, code_link) = if row.days_until_expiry < 0 {
                        (
                            "Critical",
                            format!("[CERT-003]({})", issue_codes::doc_path("CERT-003")),
                        )
                    } else if row.days_until_expiry <= 30 {
                        (
                            "Warning",
                            format!("[CERT-002]({})", issue_codes::doc_path("CERT-002")),
                        )
                    } else {
                        (
                            "Info",
                            format!("[CERT-002]({})", issue_codes::doc_path("CERT-002")),
                        )
                    };
                    let secret_cell = format!("{}/{}", row.secret_namespace, row.secret_name);
                    content.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} |\n",
                        secret_cell,
                        truncate_string(&row.subject_or_cn, 50),
                        row.expiry_utc,
                        row.days_until_expiry,
                        level,
                        code_link
                    ));
                }
                content.push('\n');
            }
        }

        // Issues: flat table with Level column (Error/Critical/Warning/Pass). Issue Code is link to doc.
        if !inspection.summary.issues.is_empty() {
            let grouped = Self::group_issues_by_severity_and_type(&inspection.summary.issues);
            let severity_to_level = |s: &IssueSeverity| -> &'static str {
                match s {
                    IssueSeverity::Critical => "Critical",
                    IssueSeverity::Warning => "Warning",
                    IssueSeverity::Info => "Info",
                }
            };
            content.push_str("| Resource | Level | Issue Code | Short Title |\n");
            content.push_str("|----------|-------|------------|-------------|\n");
            for sev in &[
                IssueSeverity::Critical,
                IssueSeverity::Warning,
                IssueSeverity::Info,
            ] {
                let level = severity_to_level(sev);
                if let Some(groups) = grouped.get(sev) {
                    for (rule_id, title, _rec, resources) in groups {
                        let code_link = rule_id
                            .as_ref()
                            .map(|c| format!("[{}]({})", c, issue_codes::doc_path(c)))
                            .unwrap_or_else(|| "-".to_string());
                        if resources.is_empty() {
                            let res_label = inspection_type_to_resource(&inspection.inspection_type);
                            content.push_str(&format!(
                                "| {} | {} | {} | {} |\n",
                                res_label, level, code_link, title
                            ));
                        } else {
                            for r in resources {
                                content.push_str(&format!(
                                    "| `{}` | {} | {} | {} |\n",
                                    r, level, code_link, title
                                ));
                            }
                        }
                    }
                }
            }
            content.push('\n');
        }

        content.push_str("---\n\n");

        Ok(content)
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}
