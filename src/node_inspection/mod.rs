//! Node inspection: DaemonSet-based collection and types for per-node checks.

pub mod collector;
pub mod types;

pub use collector::{collect_node_inspections, ensure_node_inspector_ready, NodeInspectorStatus};
#[allow(unused_imports)]
pub use types::{
    NodeCertificate, NodeInspectionResult, NodeKernel, NodeResources, NodeSecurity, NodeServices,
};
