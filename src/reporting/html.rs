//! HTML output for cluster report: minimal HTML document with tables.

use anyhow::Result;
use std::io::Write;

use crate::inspections::types::{ClusterReport, IssueSeverity};

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Writes an HTML report with cluster overview and issues tables.
pub fn write_report(report: &ClusterReport, path: &str) -> Result<()> {
    let mut f = std::fs::File::create(path)?;

    writeln!(
        f,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<title>Kubeowler Report - {}</title>
<style>
table {{ border-collapse: collapse; margin: 1em 0; }}
th, td {{ border: 1px solid #ccc; padding: 6px 10px; text-align: left; }}
th {{ background: #f5f5f5; }}
</style>
</head>
<body>
<h1>{} Kubernetes Cluster Check Report</h1>
<p><strong>Cluster</strong>: {} | <strong>Report ID</strong>: {} | <strong>Generated</strong>: {}</p>
<p><strong>Overall Score</strong>: {:.1}/100</p>
"#,
        escape_html(&report.cluster_name),
        escape_html(&report.cluster_name),
        escape_html(&report.cluster_name),
        escape_html(&report.report_id),
        report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        report.overall_score
    )?;

    if let Some(ref overview) = report.cluster_overview {
        writeln!(
            f,
            "<h2>Cluster Overview</h2><table><tr><th>Metric</th><th>Value</th></tr>"
        )?;
        if let Some(v) = &overview.cluster_version {
            writeln!(
                f,
                "<tr><td>Cluster Version</td><td>{}</td></tr>",
                escape_html(v)
            )?;
        }
        writeln!(
            f,
            "<tr><td>Node Count</td><td>{}</td></tr>",
            overview.node_count
        )?;
        writeln!(
            f,
            "<tr><td>Ready Nodes</td><td>{}</td></tr>",
            overview.ready_node_count
        )?;
        if let Some(pc) = overview.pod_count {
            writeln!(f, "<tr><td>Pod Count</td><td>{}</td></tr>", pc)?;
        }
        if let Some(nc) = overview.namespace_count {
            writeln!(f, "<tr><td>Namespace Count</td><td>{}</td></tr>", nc)?;
        }
        if let Some(age) = overview.cluster_age_days {
            writeln!(f, "<tr><td>Cluster Age (days)</td><td>{}</td></tr>", age)?;
        }
        writeln!(f, "</table>")?;
    }

    writeln!(f, "<h2>Issues</h2><table><tr><th>Module</th><th>Severity</th><th>Category</th><th>Description</th><th>Resource</th><th>Recommendation</th></tr>")?;
    for insp in &report.inspections {
        for issue in &insp.summary.issues {
            let sev = match issue.severity {
                IssueSeverity::Critical => "Critical",
                IssueSeverity::Warning => "Warning",
                IssueSeverity::Info => "Info",
            };
            writeln!(
                f,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(&insp.inspection_type),
                sev,
                escape_html(&issue.category),
                escape_html(&issue.description),
                issue
                    .resource
                    .as_deref()
                    .map(escape_html)
                    .as_deref()
                    .unwrap_or(""),
                escape_html(&issue.recommendation)
            )?;
        }
    }
    writeln!(f, "</table></body></html>")?;

    Ok(())
}
