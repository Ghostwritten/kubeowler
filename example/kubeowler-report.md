# cluster-1 Kubernetes Cluster Check Report

**Report ID**: `11ff9489-f63f-48be-a4a1-a5bf4dbf0c6c`

**Cluster**: cluster-1

**Generated At**: 2026-02-09 07:00:19 +00:00

## 🖥️ Cluster Overview

| Metric | Value |
|--------|-------|
| Cluster Version | v1.33.7 |
| Node Count | 4 |
| Ready Nodes | 4 |
| Pod Count | 83 |
| Namespace Count | 13 |
| Cluster Age (days) | 14 |
| Container Runtime | containerd://2.1.5 |
| Overall Health | 🟡 Good (Score: 82.4) |

### Node conditions

| Node | Ready | MemoryPressure | DiskPressure | PIDPressure |
|------|-------|----------------|--------------|-------------|
| master01 | True | False | False | False |
| worker01 | True | False | False | False |
| worker02 | True | False | False | False |
| worker03 | True | False | False | False |

### Workload summary

| Controller | Total | Ready |
|------------|-------|-------|
| Deployment | 34 | 32 |
| StatefulSet | 3 | 3 |
| DaemonSet | 8 | 8 |

### Storage summary

| Metric | Value |
|--------|-------|
| PV total | 6 |
| PVC total | 6 |
| PVC Bound | 6 |
| StorageClass count | 1 |
| Default StorageClass | No |

### Container resource usage (top 20 high usage)

Top 20 containers by usage vs limit (CPU or memory ≥ 80% of limit). Data from **metrics-server** (Pod metrics API) and **Pod spec** (limits). This section is **omitted when metrics-server is unavailable**.

| Namespace | Pod | Container | CPU used (m) | CPU request (m) | CPU limit (m) | Mem used (Mi) | Mem request (Mi) | Mem limit (Mi) | Note |
|-----------|-----|-----------|--------------|-----------------|---------------|---------------|------------------|----------------|------|
| upm-system | consul-webhook-cert-manager-7667c64c6f-gvndk | webhook-cert-manager | 0 | 100 | 100 | 40 | 50 | 50 | High usage |

## Node Inspection

Per-node checks from kubeowler-node-inspector DaemonSet.

### Node General Information

| Node | OS Version | IP Address | Kernel Version | Uptime | Collection time |
|------|-------------|------------|----------------|--------|------------------|
| master01 | Red Hat Enterprise Linux 9.7 (Plow) | 192.168.22.151 | 5.14.0-611.24.1.el9_7.x86_64 | 5 day(s) 23 hour(s) | 2026-02-09T07:00:19Z |
| worker01 | Red Hat Enterprise Linux 9.7 (Plow) | 192.168.22.152 | 5.14.0-611.24.1.el9_7.x86_64 | 5 day(s) 23 hour(s) | 2026-02-09T07:00:00Z |
| worker02 | Red Hat Enterprise Linux 9.7 (Plow) | 192.168.22.153 | 5.14.0-611.24.1.el9_7.x86_64 | 5 day(s) 23 hour(s) | 2026-02-09T07:00:28Z |
| worker03 | Red Hat Enterprise Linux 9.7 (Plow) | 192.168.22.154 | 5.14.0-611.24.1.el9_7.x86_64 | 5 day(s) 23 hour(s) | 2026-02-09T07:00:10Z |

### Node resources

| Node | CPU (cores) | CPU Used | CPU % | Mem Total (Gi) | Mem Used (Gi) | Mem % | Swap Total (Gi) | Swap Used (Gi) | Swap % | Load (1m, 5m, 15m) |
|------|-------------|----------|-------|----------------|---------------|-------|----------------|---------------|-------|---------------------|
| master01 | 4 | 0.59 | 14.7% | 7.5 | 2.9 | 38.9% | 0.00 | 0.00 | 0.0% | 0.47, 0.39, 0.41 |
| worker01 | 32 | 0.67 | 2.1% | 31.1 | 13.5 | 43.6% | 0.00 | 0.00 | 0.0% | 0.14, 0.16, 0.11 |
| worker02 | 12 | 0.07 | 0.6% | 23.2 | 2.8 | 12.1% | 0.00 | 0.00 | 0.0% | 0.09, 0.06, 0.01 |
| worker03 | 16 | 0.14 | 0.9% | 23.2 | 4.6 | 19.9% | 0.00 | 0.00 | 0.0% | 0.41, 0.23, 0.20 |

### Node disk usage

Per-node filesystem usage by mount. Status: Info (<60% used), Warning (60–90%), Critical (≥90%).

| Node | Mount Point | Device | FSType | Total (Gi) | Used (Gi) | Used % | Status |
|------|-------------|--------|--------|------------|------------|--------|--------|
| master01 | boot | /dev/sda2 | - | 0.9 | 0.4 | 46.7% | Info |
| master01 | / | /dev/mapper/rhel-root | - | 58.4 | 9.4 | 16.1% | Info |
| master01 | / | overlay | - | 58.4 | 9.4 | 16.1% | Info |
| worker01 | boot | /dev/sda2 | - | 0.9 | 0.4 | 46.7% | Info |
| worker01 | / | /dev/mapper/rhel-root | - | 58.4 | 21.2 | 36.4% | Info |
| worker01 | / | overlay | - | 58.4 | 21.2 | 36.4% | Info |
| worker02 | boot | /dev/sda2 | - | 0.9 | 0.4 | 46.7% | Info |
| worker02 | / | overlay | - | 58.4 | 12.5 | 21.4% | Info |
| worker02 | / | /dev/mapper/rhel-root | - | 58.4 | 12.5 | 21.4% | Info |
| worker03 | boot | /dev/sda2 | - | 0.9 | 0.4 | 46.7% | Info |
| worker03 | / | /dev/mapper/rhel-root | - | 58.4 | 13.6 | 23.2% | Info |
| worker03 | / | overlay | - | 58.4 | 13.6 | 23.2% | Info |

### Node container state counts

| Node | Running | Waiting | Exited |
|------|---------|---------|--------|
| master01 | 9 | 0 | 3 |
| worker01 | 40 | 0 | 12 |
| worker02 | 18 | 1 | 7 |
| worker03 | 24 | 1 | 8 |

## Node component and service status

| Node/Service | Kubelet | Container runtime | NTP synced | Journald | Crontab |
|------|--------|-------------------|------------|----------|----------|
| master01 | enabled | enabled | enabled | enabled | enabled |
| worker01 | enabled | enabled | enabled | enabled | disabled |
| worker02 | enabled | enabled | enabled | enabled | enabled |
| worker03 | enabled | enabled | disabled | enabled | enabled |

### Node security and kernel modules

SELinux, firewalld, IPVS, br_netfilter, overlay, and nf_conntrack status; helps troubleshoot network and security policy.

| Node | SELinux | Firewalld | IPVS | br_netfilter | overlay | nf_conntrack |
|------|---------|------------|------|--------------|---------|---------------|
| master01 | Permissive | Inactive | Yes | Yes | Yes | Yes |
| worker01 | Permissive | Inactive | Yes | Yes | Yes | Yes |
| worker02 | Permissive | Inactive | Yes | Yes | Yes | Yes |
| worker03 | Permissive | Inactive | Yes | Yes | Yes | Yes |

### Node network and stability

Conntrack usage, inode usage, OOM count, open FDs, and zombie count; used to assess node stability and resource pressure.

| Node | Conntrack usage % | Inode usage % | OOM count | FD (open/max) | Zombie count |
|------|-------------------|---------------|-----------|---------------|---------------|
| master01 | 0.0% | 1.0% | 0 | 5344/9223372036854775807 | 0 |
| worker01 | 0.0% | 2.0% | 0 | 8000/9223372036854775807 | 0 |
| worker02 | 0.0% | 1.0% | 0 | 5472/9223372036854775807 | 0 |
| worker03 | 0.0% | 1.0% | 0 | 6368/9223372036854775807 | 0 |

### Node kernel parameters

ip_forward, swappiness, and somaxconn; affects network forwarding, memory swapping, and connection queue.

| Node | net.ipv4.ip_forward | vm.swappiness | net.core.somaxconn |
|------|---------------------|--------------|--------------------|
| master01 | 0 | 60 | 4096 |
| worker01 | 0 | 60 | 4096 |
| worker02 | 0 | 60 | 4096 |
| worker03 | 0 | 60 | 4096 |

### Node Certificate Status

| Node | Path | Expired | Expiration Date (node local) | Days to Expiry | Level | Issue Code |
|------|------|---------|------------------------------|----------------|-------|------------|
| master01 | etc/kubernetes/ssl/apiserver-kubelet-client.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| master01 | etc/kubernetes/ssl/apiserver.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| master01 | etc/kubernetes/ssl/ca.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| master01 | etc/kubernetes/ssl/front-proxy-ca.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| master01 | etc/kubernetes/ssl/front-proxy-client.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| worker01 | etc/kubernetes/ssl/ca.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| worker02 | etc/kubernetes/ssl/ca.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |
| worker03 | etc/kubernetes/ssl/ca.crt | No | Jan  2 04:05:42 2126 GMT | 36485 | Info | CERT-002 |

## Recent cluster events (Warning / Error)

| Namespace | Object | Level | Reason | Message | Last seen |
|-----------|--------|-------|--------|---------|----------|
| kubeowler | Pod/kubeowler-node-inspector-7qvxp | Warning | Failed | Error: ImagePullBackOff | 2026-02-09 06:59:42 |
| kubeowler | Pod/kubeowler-node-inspector-7qvxp | Warning | Failed | Failed to pull image "docker.io/ghostwritten/kubeowler-no... | 2026-02-09 06:58:32 |
| kubeowler | Pod/kubeowler-node-inspector-7qvxp | Warning | Failed | Error: ErrImagePull | 2026-02-09 06:58:32 |
| upm-system | Pod/consul-connect-injector-6b4b944cd5-jwhpg | Warning | BackOff | Back-off restarting failed container sidecar-injector in ... | 2026-02-09 06:58:23 |
| default | Pod/nginx-deployment-fail-5cff74b894-nwdvc | Warning | Failed | Error: ImagePullBackOff | 2026-02-09 06:58:11 |
| default | Pod/nginx-deployment-fail-5cff74b894-jsls4 | Warning | Failed | Failed to pull image "nginx:non-existent-tag": rpc error:... | 2026-02-09 06:58:05 |
| upm-system | Pod/consul-connect-injector-69b8cc47c5-ssqhv | Warning | BackOff | Back-off restarting failed container sidecar-injector in ... | 2026-02-09 06:57:50 |
| kubeowler | Pod/kubeowler-node-inspector-2djmh | Warning | Failed | Failed to pull image "docker.io/ghostwritten/kubeowler-no... | 2026-02-09 06:56:21 |
| kubeowler | Pod/kubeowler-node-inspector-2djmh | Warning | Failed | Error: ErrImagePull | 2026-02-09 06:56:21 |
| default | Pod/nginx-deployment-fail-5cff74b894-jsls4 | Warning | Failed | Error: ImagePullBackOff | 2026-02-09 06:53:10 |
| upm-system | Pod/consul-connect-injector-6b4b944cd5-jwhpg | Warning | Unhealthy | Startup probe failed: Get "http://10.233.69.134:9445/read... | 2026-02-09 06:48:25 |
| upm-system | Pod/consul-connect-injector-69b8cc47c5-ssqhv | Warning | Unhealthy | Startup probe failed: Get "http://10.233.94.112:9445/read... | 2026-02-09 06:47:38 |

## 📋 Detailed Results

### Check Results

| Resource | Check Item | Status | Score | Details |
|----------|------------|--------|-------|----------|
| Service | Network Policy Coverage | ⚠️ Warning | 0.0/100.0 | 0/13 namespaces with network policies |
| PersistentVolume | Storage Class Configuration | ⚠️ Warning | 70.0/100.0 | 1 storage classes, 0 default |
| Pod | Resource Requests | ⚠️ Warning | 65.0/100.0 | 65/100 containers with resource requests |
| Pod | Resource Limits | ⚠️ Warning | 38.0/100.0 | 38/100 containers with resource limits |
| Pod | Complete Resource Configuration | ⚠️ Warning | 38.0/100.0 | 38/100 containers with complete resource configuration |
| Pod | Pod Health | ⚠️ Warning | 91.6/100.0 | Running: 76, Failed: 0, Pending: 2, Total: 83. Container ... |
| Pod | Pod Stability | ⚠️ Warning | 81.9/100.0 | 15/83 pods with excessive restarts |
| HorizontalPodAutoscaler | Horizontal Pod Autoscalers | ⚠️ Warning | 70.0/100.0 | No HPAs detected in the target scope |
| Job | CronJobs | ⚠️ Warning | 70.0/100.0 | No CronJobs detected |
| NetworkPolicy | RBAC Configuration | ⚠️ Warning | 63.5/100.0 | Risky roles: 10, Risky bindings: 2 |
| NetworkPolicy | Pod Security Standards | ⚠️ Warning | 78.3/100.0 | Secure pods: 65/83, Running as root: 7, Privileged: 11 |
| NetworkPolicy | Network Policy Coverage | ⚠️ Warning | 0.0/100.0 | 0/13 namespaces with network policies |
| ResourceQuota | Resource Quotas | ⚠️ Warning | 60.0/100.0 | No ResourceQuota objects found |
| ResourceQuota | Limit Ranges | ⚠️ Warning | 65.0/100.0 | No LimitRange objects found |
| Observability | Logging Stack | ⚠️ Warning | 70.0/100.0 | No logging stack found |
| Certificate | TLS certificate expiry | ⚠️ Warning | 70.0/100.0 | 5 certificate(s); 2 expiring in 90 days, 1 in 30 days, 0 ... |

### Namespace summary

| Namespace | Pods | Deployments | NetworkPolicy | ResourceQuota | LimitRange |
|-----------|------|-------------|---------------|---------------|------------|
| cert-manager | 3 | 3 | No | No | No |
| default | 2 | 1 | No | No | No |
| demo | 0 | 0 | No | No | No |
| kube-node-lease | 0 | 0 | No | No | No |
| kube-public | 0 | 0 | No | No | No |
| kube-system | 23 | 4 | No | No | No |
| kubeowler | 4 | 0 | No | No | No |
| kubeowler-inspector | 0 | 0 | No | No | No |
| metallb-system | 5 | 1 | No | No | No |
| openebs | 4 | 1 | No | No | No |
| prometheus | 11 | 3 | No | No | No |
| upm-system | 24 | 20 | No | No | No |
| velero | 7 | 1 | No | No | No |

<a id="pod"></a>

### Pod

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `default/nginx-deployment-fail-5cff74b894-jsls4` | Critical | [POD-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-005.md) | ImagePullBackOff |
| `default/nginx-deployment-fail-5cff74b894-nwdvc` | Critical | [POD-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-005.md) | ImagePullBackOff |
| `kube-system/nginx-proxy-worker01` | Critical | [POD-012](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-012.md) | Pod Running but not Ready |
| `kube-system/nginx-proxy-worker03` | Critical | [POD-012](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-012.md) | Pod Running but not Ready |
| `upm-system/consul-connect-injector-69b8cc47c5-ssqhv` | Critical | [POD-012](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-012.md) | Pod Running but not Ready |
| `upm-system/consul-connect-injector-6b4b944cd5-jwhpg` | Critical | [POD-012](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-012.md) | Pod Running but not Ready |
| `upm-system/consul-connect-injector-69b8cc47c5-ssqhv` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/consul-connect-injector-6b4b944cd5-jwhpg` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-auth-7cb945f67d-hvl9m` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-elasticsearch-ms-6bdff4b79b-2jwm9` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-gateway-bb4cc77d4-9rl6d` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-innodb-cluster-ms-5fb56d898b-sh7xq` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-kafka-ms-77c4ccd4d6-66kjt` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-milvus-ms-68f994794f-thjjm` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-mongodb-ms-6c5c6ffc4-hbr2c` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-mysql-ms-598d6cf7db-9w42l` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-postgresql-ms-65676ffccc-sz27w` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-redis-ms-5d4bf984f-pr2rs` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-resource-78d8494fdb-kmlvs` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `upm-system/upm-platform-zookeeper-ms-668877c9f8-zprtm` | Critical | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |
| `cert-manager/cert-manager-7b8b89f89d-dvmjv` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `cert-manager/cert-manager-cainjector-7f9fdd5dd5-zkd9g` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `cert-manager/cert-manager-webhook-769f6b94cb-d4pc7` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `kube-system/kube-proxy-4sc5z` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `kube-system/kube-proxy-6vskm` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `kube-system/kube-proxy-jv8kb` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `kube-system/kube-proxy-sk4cn` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `metallb-system/controller-9c6cff498-6qhm5` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `metallb-system/speaker-bjsz8` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `metallb-system/speaker-kgprs` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `metallb-system/speaker-ps8b8` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `metallb-system/speaker-tstgb` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/alertmanager-prometheus-kube-prometheus-alertmanager-0` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-kube-prometheus-operator-76694676bc-6dpx6` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-kube-prometheus-operator-76694676bc-h76s6` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-kube-state-metrics-7547645674-rmh26` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-kube-prometheus-prometheus-0` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-kube-prometheus-prometheus-0` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-node-exporter-5jllf` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-node-exporter-98djg` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-node-exporter-jtndm` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `prometheus/prometheus-prometheus-node-exporter-x7xfw` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/node-agent-57pwr` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/node-agent-kjplw` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/node-agent-twwbk` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770610478287-nxndw` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770614378295-js7rs` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770618278304-97tkq` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `velero/velero-7b88f944d5-tdznx` | Warning | [RES-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-001.md) | Container has no resource requests |
| `cert-manager/cert-manager-7b8b89f89d-dvmjv` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `cert-manager/cert-manager-cainjector-7f9fdd5dd5-zkd9g` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `cert-manager/cert-manager-webhook-769f6b94cb-d4pc7` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/dns-autoscaler-56cb45595c-8jsd8` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-apiserver-master01` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-controller-manager-master01` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-proxy-4sc5z` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-proxy-6vskm` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-proxy-jv8kb` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-proxy-sk4cn` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/kube-scheduler-master01` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/metrics-server-56ff78d5b7-xc7sk` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/nginx-proxy-worker01` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/nginx-proxy-worker02` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `kube-system/nginx-proxy-worker03` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `metallb-system/controller-9c6cff498-6qhm5` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `metallb-system/speaker-bjsz8` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `metallb-system/speaker-kgprs` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `metallb-system/speaker-ps8b8` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `metallb-system/speaker-tstgb` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/alertmanager-prometheus-kube-prometheus-alertmanager-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/alertmanager-prometheus-kube-prometheus-alertmanager-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-f8zn7` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-grafana-75f54c4cdd-n8rht` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-kube-prometheus-operator-76694676bc-6dpx6` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-kube-prometheus-operator-76694676bc-h76s6` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-kube-state-metrics-7547645674-rmh26` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-kube-prometheus-prometheus-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-kube-prometheus-prometheus-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-node-exporter-5jllf` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-node-exporter-98djg` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-node-exporter-jtndm` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `prometheus/prometheus-prometheus-node-exporter-x7xfw` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-auth-7cb945f67d-hvl9m` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-db-mysql-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-db-redis-0` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-elasticsearch-ms-6bdff4b79b-2jwm9` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-gateway-bb4cc77d4-9rl6d` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-innodb-cluster-ms-5fb56d898b-sh7xq` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-kafka-ms-77c4ccd4d6-66kjt` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-milvus-ms-68f994794f-thjjm` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-mongodb-ms-6c5c6ffc4-hbr2c` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-mysql-ms-598d6cf7db-9w42l` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-nginx-6b9b4df87d-lzhbl` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-postgresql-ms-65676ffccc-sz27w` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-redis-cluster-ms-6c4f9db698-qdfr6` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-redis-ms-5d4bf984f-pr2rs` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-resource-78d8494fdb-kmlvs` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-user-5494c99b96-dqmp5` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-view-7c4476d5cb-qpc2t` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-zookeeper-ms-668877c9f8-zprtm` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/node-agent-57pwr` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/node-agent-kjplw` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/node-agent-twwbk` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770610478287-nxndw` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770614378295-js7rs` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/upm-system-minio-backups-kopia-maintain-job-1770618278304-97tkq` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `velero/velero-7b88f944d5-tdznx` | Warning | [RES-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-002.md) | Container has no resource limits |
| `upm-system/upm-platform-redis-cluster-ms-6c4f9db698-qdfr6` | Warning | [POD-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POD-003.md) | Container restart count too high |

---

<a id="storageclass"></a>

### StorageClass

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| StorageClass | Warning | [STO-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/STO-009.md) | No default StorageClass |

---

<a id="clusterrole"></a>

### ClusterRole

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `metallb-system:speaker` | Warning | [SEC-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-001.md) | ClusterRole has excessive permissions |
| `openebs-lvm-provisioner-role` | Warning | [SEC-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-001.md) | ClusterRole has excessive permissions |
| `prometheus-kube-prometheus-operator` | Warning | [SEC-001](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-001.md) | ClusterRole has excessive permissions |

---

<a id="clusterrolebinding"></a>

### ClusterRoleBinding

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `velero` | Critical | [SEC-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-003.md) | ServiceAccount has cluster-admin |
| `velero-server` | Critical | [SEC-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-003.md) | ServiceAccount has cluster-admin |

---

<a id="serviceaccount"></a>

### ServiceAccount

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `default/nginx-deployment-fail-5cff74b894-jsls4` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `default/nginx-deployment-fail-5cff74b894-nwdvc` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/kube-apiserver-master01` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/kube-controller-manager-master01` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/kube-scheduler-master01` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/nginx-proxy-worker01` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/nginx-proxy-worker02` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kube-system/nginx-proxy-worker03` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kubeowler/kubeowler-node-inspector-5w9xs` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kubeowler/kubeowler-node-inspector-7qvxp` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kubeowler/kubeowler-node-inspector-sjwvr` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |
| `kubeowler/kubeowler-node-inspector-xj7tg` | Warning | [SEC-009](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-009.md) | Uses default ServiceAccount |

---

<a id="networkpolicy"></a>

### NetworkPolicy

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `cluster` | Warning | [SEC-008](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-008.md) | Insufficient network policy coverage |

---

<a id="certificate"></a>

### Certificate

#### TLS Certificate Expiry

| Secret (namespace/name) | Expired | Expiry (UTC) | Days to Expiry | Level | Issue Code |
|--------------------------|---------|--------------|----------------|-------|------------|
| demo/demo-cm-ss-root-ca | No | Feb  3 09:06:59 2027 +00:00 | 359 | Info | [CERT-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/CERT-002.md) |
| upm-system/consul-connect-inject-webhook-cert | No | Feb  9 19:18:35 2026 +00:00 | 0 | Warning | [CERT-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/CERT-002.md) |
| upm-system/consul-server-cert | No | Feb  2 09:34:47 2028 +00:00 | 723 | Info | [CERT-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/CERT-002.md) |
| upm-system/upm-engine-compose-operator-webhook-server-certs | No | May  3 08:51:37 2026 +00:00 | 83 | Info | [CERT-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/CERT-002.md) |
| upm-system/upm-engine-unit-operator-webhook-server-certs | No | May  3 08:51:16 2026 +00:00 | 83 | Info | [CERT-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/CERT-002.md) |

---

<a id="policy"></a>

### Policy

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `cluster` | Warning | [POLICY-002](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/POLICY-002.md) | No LimitRange configured |

---

<a id="observability"></a>

### Observability

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `kube-system` | Warning | [OBS-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/OBS-003.md) | Log aggregation not deployed |

---

<a id="security"></a>

### Security

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `kubeowler/kubeowler-node-inspector-5w9xs` | Warning | [SEC-006](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-006.md) | Container runs as root |
| `kubeowler/kubeowler-node-inspector-7qvxp` | Warning | [SEC-006](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-006.md) | Container runs as root |
| `kubeowler/kubeowler-node-inspector-sjwvr` | Warning | [SEC-006](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-006.md) | Container runs as root |
| `kubeowler/kubeowler-node-inspector-xj7tg` | Warning | [SEC-006](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-006.md) | Container runs as root |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-9t7xq` | Warning | [SEC-007](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-007.md) | Container allows privilege escalation |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-gtw6r` | Warning | [SEC-007](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-007.md) | Container allows privilege escalation |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-gw6kd` | Warning | [SEC-007](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-007.md) | Container allows privilege escalation |
| `kube-system/calico-node-dlmbk` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/calico-node-fjv6b` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/calico-node-s2zwk` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/calico-node-xndg7` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/kube-proxy-4sc5z` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/kube-proxy-6vskm` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/kube-proxy-jv8kb` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `kube-system/kube-proxy-sk4cn` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-9t7xq` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-gtw6r` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `openebs/openebs-lvmlocalpv-lvm-localpv-node-gw6kd` | Warning | [SEC-005](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-005.md) | Container runs privileged |
| `velero/node-agent-57pwr` | Warning | [SEC-004](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-004.md) | Pod runs as root |
| `velero/node-agent-kjplw` | Warning | [SEC-004](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-004.md) | Pod runs as root |
| `velero/node-agent-twwbk` | Warning | [SEC-004](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/SEC-004.md) | Pod runs as root |

---

<a id="resource-management"></a>

### Resource Management

| Resource | Level | Issue Code | Short Title |
|----------|-------|------------|-------------|
| `cert-manager` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |
| `default` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |
| `kubeowler-inspector` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |
| `openebs` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |
| `prometheus` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |
| `velero` | Warning | [RES-003](https://github.com/Ghostwritten/kubeowler/blob/main/docs/issues/RES-003.md) | Namespace has no resource quota |

---

---

*Report generated by [kubeowler](https://github.com/username/kubeowler).*
