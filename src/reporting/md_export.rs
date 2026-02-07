//! Export report from Markdown: MD -> HTML (comrak), MD -> CSV (parse tables).

use anyhow::Result;
use base64::Engine;
use comrak::{markdown_to_html, ComrakOptions};

/// Logo image embedded at compile time; encoded as data URI so HTML report is self-contained.
fn embedded_logo_data_uri() -> String {
    let bytes = include_bytes!("../../assets/logo.png");
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// Convert Markdown string to a full HTML document.
pub fn md_to_html(md: &str) -> Result<String> {
    let mut opts = ComrakOptions::default();
    opts.extension.table = true;
    let body = markdown_to_html(md, &opts);
    let logo_src = embedded_logo_data_uri();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<title>Kubeowler Report</title>
<style>
:root {{
  --cell-padding-vertical: 0.25em;
  --cell-padding-horizontal: 0.25em;
  --font-family-sans: system-ui, -apple-system, sans-serif;
}}
body {{
  max-width: 60em;
  margin: auto;
  font-family: var(--font-family-sans);
}}
table {{
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  border-top: 0.1em solid #333;
  border-bottom: 0.1em solid #333;
}}
thead {{
  border-bottom: 0.1em solid #333;
}}
th, td {{
  padding-top: var(--cell-padding-vertical);
  padding-bottom: var(--cell-padding-vertical);
  padding-left: var(--cell-padding-horizontal);
  padding-right: var(--cell-padding-horizontal);
  text-align: left;
  vertical-align: top;
}}
th {{
  background: #f5f5f5;
}}
td > p {{
  margin: 0;
  word-break: break-all;
  hyphens: auto;
}}
td {{
  word-break: break-all;
  hyphens: auto;
}}
.report-logo {{
  width: 25%;
  max-width: 200px;
  float: right;
}}
</style>
</head>
<body>
<img class="report-logo" src="{}" alt="Kubeowler"/>
{}
</body>
</html>"#,
        logo_src, body
    );
    Ok(html)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Extract rule_id from a markdown link like [STO-009](url) or plain text.
fn extract_rule_id(cell: &str) -> String {
    let cell = cell.trim();
    if let Some(start) = cell.find('[') {
        if let Some(end) = cell[start..].find(']') {
            return cell[start + 1..start + end].to_string();
        }
    }
    cell.to_string()
}

/// Parse MD and convert to CSV: cluster_overview row + issue rows from per-resource tables.
pub fn md_to_csv(md: &str) -> Result<String> {
    let mut out = String::new();
    let lines: Vec<&str> = md.lines().collect();

    let mut cluster_name = String::new();
    let mut report_id = String::new();
    let mut overview: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut seen_cluster_overview = false;
    let mut in_overview_table = false;
    let mut current_section = String::new();
    let mut issue_rows: Vec<(String, String, String, String, String)> = Vec::new(); // section, resource, level, rule_id, short_title

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("**Cluster**:") {
            cluster_name = line
                .trim_start_matches("**Cluster**:")
                .trim()
                .trim_matches(' ')
                .to_string();
        } else if line.starts_with("**Report ID**:") {
            report_id = line
                .trim_start_matches("**Report ID**:")
                .trim()
                .trim_matches('`')
                .trim()
                .to_string();
        } else if line.contains("Cluster Overview")
            && (line.starts_with("##") || line.contains("🖥️"))
        {
            seen_cluster_overview = true;
        } else if (seen_cluster_overview || in_overview_table) && line.starts_with('|') {
            let cells: Vec<&str> = line
                .split('|')
                .map(|c| c.trim())
                .filter(|c| !c.is_empty())
                .collect();
            if cells.len() >= 2 && cells[0] == "Metric" && cells[1] == "Value" {
                in_overview_table = true;
                seen_cluster_overview = false;
            } else if in_overview_table
                && cells.len() >= 2
                && !cells[0].chars().all(|c| c == '-' || c == ' ')
            {
                overview.insert(cells[0].to_string(), cells[1].to_string());
            }
        } else if in_overview_table && (!line.starts_with('|') || line.trim().is_empty()) {
            in_overview_table = false;
        }
        if line.starts_with("## ") && !line.contains("Cluster Overview") {
            seen_cluster_overview = false;
        }

        if line.starts_with("### ") && !line.starts_with("#### ") {
            current_section = line.trim_start_matches("### ").trim().to_string();
        }

        if line.starts_with('|')
            && (line.contains("Resource")
                && line.contains("Level")
                && line.contains("Issue Code")
                && line.contains("Short Title"))
        {
            i += 1;
            if i < lines.len() && lines[i].contains("---") {
                i += 1;
            }
            while i < lines.len() && lines[i].starts_with('|') {
                let row = lines[i];
                let cells: Vec<&str> = row
                    .split('|')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if cells.len() >= 4 && !cells[0].chars().all(|c| c == '-') {
                    let resource = cells[0].trim_matches('`').to_string();
                    let level = cells[1].to_string();
                    let rule_id = extract_rule_id(cells.get(2).unwrap_or(&""));
                    let short_title = cells.get(3).unwrap_or(&"").to_string();
                    issue_rows.push((
                        current_section.clone(),
                        resource,
                        level,
                        rule_id,
                        short_title,
                    ));
                }
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    out.push_str("section,cluster_name,report_id,cluster_version,node_count,ready_node_count,pod_count,namespace_count,cluster_age_days\n");
    let cv = overview.get("Cluster Version").cloned().unwrap_or_default();
    let nn = overview.get("Node Count").cloned().unwrap_or_default();
    let rn = overview.get("Ready Nodes").cloned().unwrap_or_default();
    let pc = overview.get("Pod Count").cloned().unwrap_or_default();
    let ns = overview.get("Namespace Count").cloned().unwrap_or_default();
    let age = overview
        .get("Cluster Age (days)")
        .cloned()
        .unwrap_or_default();
    out.push_str(&format!(
        "cluster_overview,{},{},{},{},{},{},{},{}\n",
        escape_csv(&cluster_name),
        escape_csv(&report_id),
        escape_csv(&cv),
        escape_csv(&nn),
        escape_csv(&rn),
        escape_csv(&pc),
        escape_csv(&ns),
        escape_csv(&age)
    ));

    out.push_str(
        "section,inspection_type,severity,category,description,resource,recommendation,rule_id\n",
    );
    for (section, resource, level, rule_id, short_title) in issue_rows {
        out.push_str(&format!(
            "issue,{0},{1},{2},{3},{4},{5},{6},{7}\n",
            escape_csv(&section),
            escape_csv(&section),
            escape_csv(&level),
            escape_csv(&section),
            escape_csv(&short_title),
            escape_csv(&resource),
            escape_csv(""),
            escape_csv(&rule_id),
        ));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md_to_csv_parses_cluster_overview() {
        let md = r#"# Report
**Report ID**: `test-id`
**Cluster**: my-cluster
**Generated At**: 2026-01-01 00:00:00 UTC
## Cluster Overview
| Metric | Value |
|--------|-------|
| Cluster Version | v1.33.7 |
| Node Count | 4 |
| Ready Nodes | 4 |
| Pod Count | 83 |
| Namespace Count | 13 |
| Cluster Age (days) | 11 |
### Pod
| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `ns/pod-1` | Critical | [POD-003](http://x) | Restart count high |
"#;
        let csv = md_to_csv(md).unwrap();
        assert!(csv.contains("section,cluster_name,report_id,cluster_version,node_count,ready_node_count,pod_count,namespace_count,cluster_age_days"));
        assert!(
            csv.contains("cluster_overview"),
            "CSV must have cluster_overview row"
        );
        assert!(
            csv.contains("my-cluster") && csv.contains("test-id"),
            "cluster name and report id"
        );
        assert!(
            csv.contains("v1.33.7") && csv.contains(",4,4,83,13,11"),
            "overview metrics from Metric|Value table"
        );
        assert!(
            csv.contains("POD-003")
                && csv.contains("Restart count high")
                && csv.contains("ns/pod-1"),
            "issue row from Resource|Level|Issue Code|Short Title table"
        );
    }

    #[test]
    fn md_to_html_renders_tables() {
        let md = r#"# Report
## Overview
| Metric | Value |
|--------|-------|
| Nodes | 4 |
"#;
        let html = md_to_html(md).unwrap();
        assert!(
            html.contains("<table>"),
            "HTML should contain <table> when extension.table is enabled"
        );
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td>"));
        assert!(
            html.contains("data:image/png;base64,"),
            "HTML should embed logo as data URI for standalone report"
        );
    }
}
