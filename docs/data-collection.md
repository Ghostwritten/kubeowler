# How Kubeowler Collects and Organizes Cluster Data

This document describes how Kubeowler gathers information from a Kubernetes cluster and turns it into a structured inspection report. It is intended for operators and contributors who need a clear picture of the data flow and design choices.

---

## 1. Overview

Kubeowler is a cluster inspection tool that:

1. Connects to a Kubernetes cluster using standard kubeconfig.
2. Collects data from the Kubernetes API (and, optionally, from node-level agents).
3. Evaluates that data through inspection modules (node health, pods, network, storage, security, etc.).
4. Aggregates results into a single in-memory report and renders it as Markdown (and optionally a summary file).

All collection is read-only: Kubeowler does not create, update, or delete cluster resources. It only lists and reads objects and, for node inspection, reads Pod logs.

---

## 2. Connection and Kubernetes API Client

- At startup, Kubeowler uses the kubeconfig active in the environment (e.g. KUBECONFIG or ~/.kube/config).
- It builds a Kubernetes API client (via the kube crate) from that config. Authentication (client certificate, token, or exec) is handled according to the kubeconfig.
- The cluster name in the report comes from the current context of the kubeconfig, not from the cluster API.

The codebase wraps the client in a **K8sClient** that exposes typed APIs for Nodes, Pods, Services, Namespaces, Secrets, PVs, PVCs, Deployments, ReplicaSets, DaemonSets, StatefulSets, Jobs, CronJobs, NetworkPolicies, StorageClasses, RBAC, and related resources. Inspectors use these APIs to list and get resources; no write operations are performed.

---

## 3. Data Collection Paths

Kubeowler has three main data paths:

1. **Cluster overview** — nodes, version, optional metrics.
2. **Module-based inspections** — API-only checks per domain.
3. **Node inspection** — DaemonSet Pod logs, parsed as JSON.

### 3.1 Cluster overview

Kubeowler optionally builds a cluster overview: API server version (from /version), node list (Nodes API: name, osImage, architecture, kubeletVersion, Ready, pod count), and optionally node resource usage (metrics.k8s.io if metrics-server is present). This is stored in ClusterReport.cluster_overview and rendered at the top of the report. No node-level agent is required.

### 3.2 Module-based inspections (API-only)

Inspection modules use K8sClient to list/get resources, run domain-specific checks, and produce an InspectionResult (checks, summary with issues, optional tables). Examples: Node Health, Control Plane, Network, Storage, Resource Usage, Pod Status, Security, Certificates, Observability, Batch, Policies. The InspectionRunner runs a subset or all modules, computes overall score and executive summary, and stores results in ClusterReport.inspections. No DaemonSet is required for this path.

### 3.3 Node inspection (DaemonSet + Pod logs)

For per-node host-level data (CPU, memory, root disk, load, runtime, journald, SELinux, sysctl), Kubeowler relies on an optional DaemonSet. One Pod per node runs a script that writes one JSON object to stdout; that stdout is the Pod log. Kubeowler does not read files from PVC or node; it only reads Pod logs via the Kubernetes API. When the user runs `kubeowler check` with type all or nodes, the code lists Pods in the node-inspector namespace (default **kubeowler**) with label app=kubeowler-node-inspector, fetches each Pod log, parses JSON into NodeInspectionResult, and stores in ClusterReport.node_inspection_results. If no DaemonSet Pods exist, node_inspection_results is empty and the report omits the Node Inspection section.

---

## 4. In-Memory Report Structure

ClusterReport holds: cluster_name, report_id, timestamp, overall_score, inspections (list of InspectionResult), executive_summary, cluster_overview (optional), node_inspection_results (optional), display_timestamp (optional, from first node's timestamp_local for report header), display_timestamp_filename (optional, for filename in cluster local time). No database or external storage is used.

---

## 5. Report Generation

The report generator takes ClusterReport and produces Markdown. It does not re-query the cluster. It renders: Cluster Overview, Node Inspection (if node_inspection_results present), Executive Summary, Detailed Results (check results, issues grouped by resource), Key Findings and Recommendations. Filters are applied at generation time. Node disk and certificate paths in the report are shown in **host perspective** (any `/host` prefix from the Pod view is stripped). Node Certificate Status and TLS Certificate Expiry tables include Level and Issue Code (e.g. CERT-002, CERT-003); the TLS table also has an Expired (Yes/No) column and "Days to Expiry". Time semantics (header/filename vs. TLS vs. node cert) are described in §6.

---

## 6. Time Semantics in Reports

Kubeowler uses different time conventions depending on context:

| Location | Time convention | Notes |
|----------|-----------------|-------|
| Report header ("Generated At") | Cluster host local time | Taken from the first node's `timestamp_local` when node inspection data is available; falls back to UTC otherwise. Example: `2026-02-09 18:38:22 +08:00`. |
| Report filename | Cluster host local time | Same as header; the timestamp portion (e.g. `2026-02-09-183822`) reflects cluster local time when node inspection ran. |
| TLS Certificate Expiry table | UTC | Expiration dates come from x509 validity parsed in Rust; labeled "Expiry (UTC)". |
| Node Certificate Status table | Node local time | Expiration dates come from the node script (`openssl x509 -noout -enddate`), which uses the node/container local timezone. Labeled "Expiration Date (node local)". |

Node certificate fix workflows differ from TLS Secret certificate workflows (node renewal vs. Secret update), but both use the same issue codes (CERT-002, CERT-003) for "expiring soon" and "expired".

---

## 7. References

- Node inspection JSON schema: [node-inspection-schema.md](node-inspection-schema.md)
- Node inspector build and deploy: [node-inspector-build-deploy.md](node-inspector-build-deploy.md)
- Installation and usage: [installation.md](installation.md)
