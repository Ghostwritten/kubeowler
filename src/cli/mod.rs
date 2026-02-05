use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about = "Kubernetes cluster inspection tool", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run cluster inspection
    Check {
        /// Cluster name for the report title (default: from kubeconfig or "default")
        #[arg(long = "cluster-name", value_name = "NAME")]
        cluster_name: Option<String>,

        /// Namespace(s) scope for inspection: only resources in this namespace are inspected. When unset, all namespaces are inspected.
        #[arg(short, long, value_name = "NAMESPACE")]
        namespace: Option<String>,

        /// Namespace where kubeowler-node-inspector DaemonSet runs; used only for node-level data collection. Default: kubeowler.
        #[arg(
            long = "node-inspector-namespace",
            value_name = "NAMESPACE",
            default_value = "kubeowler"
        )]
        node_inspector_namespace: String,

        /// Output file path for the report; if not set, defaults to {cluster-name}-kubernetes-inspection-report-{YYYY-MM-DD-HHMMSS}.{ext}
        #[arg(short, long)]
        output: Option<String>,

        /// Output format: md (default), json, csv, or html
        #[arg(short, long, default_value = "md")]
        format: ReportFormat,

        /// Kubernetes config file path
        #[arg(short, long)]
        config_file: Option<String>,

        /// Check levels to show in report: "all" or comma-separated (Info, warning, critical). Default: warning,critical.
        #[arg(
            short = 'l',
            long = "level",
            value_name = "LEVELS",
            default_value = "warning,critical"
        )]
        level: String,
    },
}

#[derive(Clone, Copy, ValueEnum, Debug, Default)]
#[value(rename_all = "kebab-case")]
pub enum ReportFormat {
    #[default]
    Md,
    Json,
    Csv,
    Html,
}

#[derive(Clone, ValueEnum, Debug)]
#[value(rename_all = "kebab-case")]
pub enum InspectionType {
    /// Full cluster inspection (default)
    All,
    /// Node health inspection
    Nodes,
    /// Pod status inspection
    Pods,
    /// Resource usage inspection
    Resources,
    /// Network connectivity inspection
    Network,
    /// Storage inspection
    Storage,
    /// Security configuration inspection
    Security,
    /// Control plane health inspection
    ControlPlane,
    /// Autoscaling health inspection
    Autoscaling,
    /// Batch and CronJob inspection
    Batch,
    /// Namespace policies inspection (quota/limit/pdb)
    Policies,
    /// Observability components inspection
    Observability,
    /// Upgrade readiness inspection
    Upgrade,
    /// Certificate (CSR) inspection
    Certificates,
}

impl FromStr for InspectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(InspectionType::All),
            "nodes" => Ok(InspectionType::Nodes),
            "pods" => Ok(InspectionType::Pods),
            "resources" => Ok(InspectionType::Resources),
            "network" => Ok(InspectionType::Network),
            "storage" => Ok(InspectionType::Storage),
            "security" => Ok(InspectionType::Security),
            "control" | "control-plane" => Ok(InspectionType::ControlPlane),
            "autoscaling" | "hpa" => Ok(InspectionType::Autoscaling),
            "batch" | "cron" => Ok(InspectionType::Batch),
            "policies" | "policy" => Ok(InspectionType::Policies),
            "observability" | "monitoring" => Ok(InspectionType::Observability),
            "upgrade" | "upgrade-readiness" => Ok(InspectionType::Upgrade),
            "certificates" | "certificate" | "csr" => Ok(InspectionType::Certificates),
            _ => Err(format!("Unknown inspection type: {}", s)),
        }
    }
}
