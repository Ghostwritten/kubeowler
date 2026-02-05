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

For per-node host-level data (CPU, memory, root disk, load, runtime, journald, SELinux, sysctl), Kubeowler relies on an optional DaemonSet. One Pod per node runs a script that writes one JSON object to stdout; that stdout is the Pod log. Kubeowler does not read files from PVC or node; it only reads Pod logs via the Kubernetes API. When the user runs `kubeowler check` with type all or nodes, the code lists Pods in kube-system with label app=kubeowler-node-inspector, fetches each Pod log, parses JSON into NodeInspectionResult, and stores in ClusterReport.node_inspection_results. If no DaemonSet Pods exist, node_inspection_results is empty and the report omits the Node Inspection section.

---

## 4. In-Memory Report Structure

ClusterReport holds: cluster_name, report_id, timestamp, overall_score, inspections (list of InspectionResult), executive_summary, cluster_overview (optional), node_inspection_results (optional). No database or external storage is used.

---

## 5. Report Generation

The report generator takes ClusterReport and produces Markdown. It does not re-query the cluster. It renders: Cluster Overview, Node Inspection (if node_inspection_results present), Executive Summary, Detailed Results (check results, issues grouped by resource), Key Findings and Recommendations. Filters are applied at generation time.

---

## 6. References

- Node inspection JSON schema: [08-node-inspection-schema.md](08-node-inspection-schema.md)
- Node inspector build and deploy: [06-node-inspector-build-deploy.md](06-node-inspector-build-deploy.md)
- Installation and usage: [01-installation-guide.md](01-installation-guide.md)
