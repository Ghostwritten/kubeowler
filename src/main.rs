use anyhow::Result;
use clap::Parser;
use log::info;
use colored::Colorize;

mod cli;
mod k8s;
mod inspections;
mod node_inspection;
mod scoring;
mod reporting;
mod utils;

use cli::{Args, Commands, ReportFormat, InspectionType};
use k8s::client::K8sClient;
use inspections::InspectionRunner;
use inspections::types::ClusterReport;
use reporting::ReportGenerator;
use reporting::generator::{parse_check_level_filter};

/// Sanitize cluster name for use in filename: replace invalid chars with `-`, collapse and trim.
fn sanitize_cluster_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if s.is_empty() {
        "cluster".to_string()
    } else {
        s
    }
}

fn output_path_with_extension(path: Option<String>, report: &ClusterReport, format: ReportFormat) -> String {
    let ext = match format {
        ReportFormat::Md => "md",
        ReportFormat::Json => "json",
        ReportFormat::Csv => "csv",
        ReportFormat::Html => "html",
    };
    let default_name = {
        let safe_name = sanitize_cluster_name(&report.cluster_name);
        let ts = report.timestamp.format("%Y-%m-%d-%H%M%S");
        format!("{}-kubernetes-inspection-report-{}.{}", safe_name, ts, ext)
    };
    let path = path.unwrap_or(default_name);
    if path.ends_with('.') || !path.contains('.') {
        format!("{}.{}", path.trim_end_matches('.'), ext)
    } else {
        path
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Commands::Check {
            cluster_name,
            namespace,
            node_inspector_namespace,
            output,
            format,
            config_file,
            level,
        } => {
            run_check_command(
                cluster_name,
                namespace,
                node_inspector_namespace,
                output,
                format,
                config_file,
                level,
            )
            .await?;
        }
    }

    Ok(())
}

async fn run_check_command(
    cluster_name: Option<String>,
    namespace: Option<String>,
    node_inspector_namespace: String,
    output: Option<String>,
    format: ReportFormat,
    config_file: Option<String>,
    level: String,
) -> Result<()> {
    println!("{}", "🔍 Kubeowler - Kubernetes Cluster Checker".bright_cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    info!("Starting Kubernetes cluster check");

    println!("📋 {}", "Configuration:".bright_yellow().bold());
    println!(
        "   Inspection scope: {}",
        namespace
            .as_deref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "all namespaces".to_string())
            .bright_green()
    );
    println!("   Node inspector DaemonSet: {}", node_inspector_namespace.bright_green());
    println!("   Output File: {}", output.as_deref().unwrap_or("(auto)").bright_green());
    println!();

    print!("🔗 Connecting to cluster... ");
    let client = match K8sClient::new(config_file.as_deref()).await {
        Ok(client) => {
            println!("{}", "✅ Success".bright_green());
            client
        }
        Err(e) => {
            println!("{}", "❌ Failed".bright_red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    println!("🔍 Running checks...");
    let runner = InspectionRunner::new(client);

    let results = match runner
        .run_inspections(
            InspectionType::All,
            namespace.as_deref(),
            &node_inspector_namespace,
            cluster_name.as_deref(),
        )
        .await
    {
        Ok(results) => {
            println!("{}", "✅ Completed".bright_green());
            results
        }
        Err(e) => {
            println!("{}", "❌ Failed".bright_red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    println!();
    println!("{}", "📊 Summary:".bright_yellow().bold());
    println!("   Overall Score: {} {:.1}/100",
        if results.overall_score >= 90.0 { "🟢" }
        else if results.overall_score >= 80.0 { "🟡" }
        else if results.overall_score >= 70.0 { "🟠" }
        else { "🔴" },
        results.overall_score
    );

    let total_issues: usize = results
        .inspections
        .iter()
        .map(|i| i.summary.issues.len())
        .sum();

    println!("   Issues Found: {}",
        if total_issues == 0 {
            format!("{}", total_issues).bright_green()
        } else {
            format!("{}", total_issues).bright_yellow()
        }
    );

    let output_path = output_path_with_extension(output, &results, format);

    print!("📝 Generating report... ");
    match format {
        ReportFormat::Json => {
            let file = std::fs::File::create(&output_path)?;
            serde_json::to_writer_pretty(file, &results)?;
            println!("{}", "✅ Done".bright_green());
            println!();
            println!("{}", "🎉 Check completed successfully!".bright_green().bold());
            println!("   Report: {}", output_path.bright_cyan());
            Ok(())
        }
        ReportFormat::Csv => {
            reporting::csv::write_report(&results, &output_path)?;
            println!("{}", "✅ Done".bright_green());
            println!();
            println!("{}", "🎉 Check completed successfully!".bright_green().bold());
            println!("   Report: {}", output_path.bright_cyan());
            Ok(())
        }
        ReportFormat::Html => {
            reporting::html::write_report(&results, &output_path)?;
            println!("{}", "✅ Done".bright_green());
            println!();
            println!("{}", "🎉 Check completed successfully!".bright_green().bold());
            println!("   Report: {}", output_path.bright_cyan());
            Ok(())
        }
        ReportFormat::Md => {
            let generator = ReportGenerator::new();
            let check_level_filter = Some(parse_check_level_filter(&level));
            generator
                .generate_report_with_filters(
                    &results,
                    &output_path,
                    None,
                    true,
                    None,
                    None,
                    check_level_filter,
                )
                .await?;
            println!("{}", "✅ Done".bright_green());
            println!();
            println!("{}", "🎉 Check completed successfully!".bright_green().bold());
            println!("   Report: {}", output_path.bright_cyan());
            Ok(())
        }
    }
}