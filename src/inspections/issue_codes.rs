//! Issue code registry: stable codes and short titles for report grouping and docs linking.
//! Format: prefix (NODE/POD/RES/NET/STO/SEC/CTRL/AUTO/BATCH/POLICY/OBS) + three-digit number.

/// Returns the short title for an issue code, or None if unknown.
pub fn short_title(code: &str) -> Option<&'static str> {
    match code {
        // Node
        "NODE-001" => Some("Node not ready"),
        "NODE-002" => Some("Node has resource pressure"),
        "NODE-003" => Some("Zombie processes on node"),
        "NODE-004" => Some("Node disk usage high (Warning)"),
        "NODE-005" => Some("Node disk usage critical"),
        // Pod
        "POD-001" => Some("Pod in Failed state"),
        "POD-002" => Some("Pod cannot be scheduled"),
        "POD-003" => Some("Container restart count too high"),
        "POD-004" => Some("Container in abnormal state"),
        "POD-005" => Some("ImagePullBackOff"),
        "POD-006" => Some("ErrImagePull"),
        "POD-007" => Some("CrashLoopBackOff"),
        "POD-008" => Some("ContainerCreating"),
        "POD-009" => Some("CreateContainerConfigError"),
        "POD-010" => Some("OOMKilled"),
        "POD-011" => Some("Container terminated (non-zero exit)"),
        "POD-012" => Some("Pod Running but not Ready"),
        // Resource
        "RES-001" => Some("Container has no resource requests"),
        "RES-002" => Some("Container has no resource limits"),
        "RES-003" => Some("Namespace has no resource quota"),
        "RES-004" => Some("CPU limit below request"),
        "RES-005" => Some("Memory limit below request"),
        // Network
        "NET-001" => Some("LoadBalancer has no external IP"),
        "NET-002" => Some("NodePort outside recommended range"),
        "NET-003" => Some("Service has no selector or endpoints"),
        "NET-004" => Some("DNS deployment not ready"),
        "NET-005" => Some("DNS service not found"),
        // Storage
        "STO-001" => Some("PV config or backing storage issue"),
        "STO-002" => Some("PV Released, needs cleanup"),
        "STO-003" => Some("PV Retained, manual action needed"),
        "STO-004" => Some("PV has no reclaim policy"),
        "STO-005" => Some("PVC storage class or capacity issue"),
        "STO-006" => Some("PVC has data loss risk"),
        "STO-007" => Some("PVC has no storage class"),
        "STO-008" => Some("StorageClass has no provisioner"),
        "STO-009" => Some("No default StorageClass"),
        "STO-010" => Some("Multiple StorageClasses marked default"),
        // Security
        "SEC-001" => Some("ClusterRole has excessive permissions"),
        "SEC-002" => Some("User has cluster-admin"),
        "SEC-003" => Some("ServiceAccount has cluster-admin"),
        "SEC-004" => Some("Pod runs as root"),
        "SEC-005" => Some("Container runs privileged"),
        "SEC-006" => Some("Container runs as root"),
        "SEC-007" => Some("Container allows privilege escalation"),
        "SEC-008" => Some("Insufficient network policy coverage"),
        "SEC-009" => Some("Uses default ServiceAccount"),
        // Control plane
        "CTRL-001" => Some("Control plane component not ready"),
        "CTRL-002" => Some("Static Pod not ready"),
        // Autoscaling
        "AUTO-001" => Some("HPA replica range too narrow"),
        "AUTO-002" => Some("HPA has no metrics configured"),
        "AUTO-003" => Some("HPA target workload or metrics issue"),
        "AUTO-004" => Some("HPA behavior limits scaling"),
        "AUTO-005" => Some("HPA metric target not configured"),
        // Batch
        "BATCH-001" => Some("CronJob suspended"),
        "BATCH-002" => Some("CronJob job failed"),
        "BATCH-003" => Some("CronJob schedule or controller issue"),
        "BATCH-004" => Some("Job needs backoffLimit or resource check"),
        "BATCH-005" => Some("Job Pod stuck or timeout adjustment needed"),
        // Policy
        "POLICY-001" => Some("No ResourceQuota configured"),
        "POLICY-002" => Some("No LimitRange configured"),
        "POLICY-003" => Some("Critical workload has no PDB"),
        "POLICY-004" => Some("Replica count does not satisfy PDB"),
        // Observability
        "OBS-001" => Some("metrics-server not deployed"),
        "OBS-002" => Some("kube-state-metrics not deployed"),
        "OBS-003" => Some("Log aggregation not deployed"),
        "OBS-004" => Some("Prometheus/monitoring not deployed"),
        // Certificates
        "CERT-001" => Some("CSR long Pending or abnormal"),
        "CERT-002" => Some("Certificate expiring soon"),
        "CERT-003" => Some("Certificate expired"),
        _ => None,
    }
}

/// Relative path to the issue doc from repo root (e.g. for report links).
pub fn doc_path(code: &str) -> String {
    format!("docs/issues/{}.md", code)
}
