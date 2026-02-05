//! CSV output for cluster report: flat tables for overview and issues.

use anyhow::Result;
use std::io::Write;

use crate::inspections::types::ClusterReport;

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Writes a CSV report: section "cluster_overview" (one row), then "issues" (one row per issue).
pub fn write_report(report: &ClusterReport, path: &str) -> Result<()> {
    let mut f = std::fs::File::create(path)?;

    if let Some(ref overview) = report.cluster_overview {
        writeln!(
            f,
            "section,cluster_name,report_id,cluster_version,node_count,ready_node_count,pod_count,namespace_count,cluster_age_days"
        )?;
        let cv = overview.cluster_version.as_deref().unwrap_or("");
        let pc = overview.pod_count.map(|p| p.to_string()).unwrap_or_default();
        let nc = overview.namespace_count.map(|n| n.to_string()).unwrap_or_default();
        let age = overview.cluster_age_days.map(|d| d.to_string()).unwrap_or_default();
        writeln!(
            f,
            "cluster_overview,{},{},{},{},{},{},{},{}",
            escape_csv(&report.cluster_name),
            escape_csv(&report.report_id),
            escape_csv(cv),
            overview.node_count,
            overview.ready_node_count,
            escape_csv(&pc),
            escape_csv(&nc),
            escape_csv(&age)
        )?;
    }

    writeln!(f, "section,inspection_type,severity,category,description,resource,recommendation,rule_id")?;
    for insp in &report.inspections {
        for issue in &insp.summary.issues {
            let sev = format!("{:?}", issue.severity);
            let res = issue.resource.as_deref().unwrap_or("");
            let rid = issue.rule_id.as_deref().unwrap_or("");
            writeln!(
                f,
                "issue,{},{},{},{},{},{},{}",
                escape_csv(&insp.inspection_type),
                escape_csv(&sev),
                escape_csv(&issue.category),
                escape_csv(&issue.description),
                escape_csv(res),
                escape_csv(&issue.recommendation),
                escape_csv(rid)
            )?;
        }
    }

    Ok(())
}
