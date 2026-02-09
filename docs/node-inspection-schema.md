# Node Inspection JSON Schema

Single JSON object per node, stdout from the DaemonSet script (`node-check-universal.sh`). Consumed by Kubeowler for the resources, services, security, kernel, disk, and certificate sections of the report.

---

## Root

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| node_name | string | yes | Node name (hostname or K8s node name) |
| hostname | string | no | Same as node_name if omitted |
| timestamp | string | no | ISO8601 or simple date-time (UTC) |
| timestamp_local | string | no | Node local time for report header/filename, e.g. 2026-02-09T18:38:22+0800 |
| runtime | string | no | "containerd" \| "docker" \| "cri-o" \| "unknown" |
| os_version | string | no | OS version (e.g. from /etc/os-release PRETTY_NAME) |
| kernel_version | string | no | Kernel release (e.g. from /proc/sys/kernel/osrelease) |
| uptime | string | no | System uptime (e.g. "2 days", "5 hours") |
| resources | object | no | See NodeResources |
| services | object | no | See NodeServices |
| security | object | no | See NodeSecurity |
| kernel | object | no | See NodeKernel |
| container_state_counts | object | no | Per-state counts from Kubernetes API: "running", "exited", "waiting" (filled by Kubeowler, not by script) |
| zombie_count | number | no | Number of zombie processes on the node (state Z in /proc) |
| issue_count | number | no | Count of warning/error checks for summary |
| node_certificates | array | no | See NodeCertificate; certs discovered from process cmdlines |
| node_disks | array | no | See NodeDisk; per-mount disk usage from gather_disk_mounts |

---

## NodeResources

| Field | Type | Description |
|-------|------|-------------|
| cpu_cores | number | CPU core count |
| memory_total_mib | number | Total memory MiB |
| memory_used_mib | number | Used memory MiB |
| memory_used_pct | number | Memory use percentage |
| root_disk_pct | number | Root filesystem use percentage |
| disk_total_g | number | Aggregated disk total (all mounts) in GB |
| disk_used_g | number | Aggregated disk used in GB |
| disk_used_pct | number | Aggregated disk use percentage |
| load_1m | string | 1-min load average (e.g. from /proc/loadavg) |
| load_5m | string | 5-min load average |
| load_15m | string | 15-min load average |
| swap_enabled | boolean | Whether swap is on |
| swap_total_g | number | Swap total in GB (from /proc/meminfo) |
| swap_used_g | number | Swap used in GB |
| swap_used_pct | number | Swap use percentage |
| status | string | "ok" \| "warning" \| "error" |
| detail | string | Optional message |

---

## NodeCertificate

| Field | Type | Description |
|-------|------|-------------|
| path | string | Certificate file path (from process cmdline) |
| expiration_date | string | NotAfter date string from openssl |
| days_remaining | number | Days until expiry (negative if expired) |
| status | string | "Valid" \| "Expiring soon" \| "Expired" |

---

## NodeServices

| Field | Type | Description |
|-------|------|-------------|
| runtime | string | "containerd" \| "docker" \| "cri-o" \| "unknown" |
| journald_active | boolean | systemd-journald active |
| crontab_present | boolean | Whether crontab exists |
| ntp_synced | boolean | NTP/time synchronized (timedatectl or chronyc) |
| status | string | "ok" \| "warning" \| "error" |
| detail | string | Optional message |

---

## NodeSecurity

| Field | Type | Description |
|-------|------|-------------|
| selinux | string | e.g. "Enforcing" \| "Permissive" \| "Disabled" |
| firewalld_active | boolean | firewalld active |
| ipvs_loaded | boolean | IPVS kernel module loaded |
| status | string | "ok" \| "warning" \| "error" |
| detail | string | Optional message |

---

## NodeKernel

| Field | Type | Description |
|-------|------|-------------|
| net_ipv4_ip_forward | string | sysctl value |
| vm_swappiness | string | sysctl value |
| net_core_somaxconn | string | sysctl value |
| status | string | "ok" \| "warning" \| "error" |
| detail | string | Optional message |

---

## NodeDisk

Per-mount entry from `gather_disk_mounts`; used in the Node disk usage table and NODE-004/NODE-005 checks.

| Field | Type | Description |
|-------|------|-------------|
| device | string | Block device or source |
| mount_point | string | Mount path |
| fstype | string | Filesystem type (may be empty if not parsed by script) |
| total_g | number | Total size in GB |
| used_g | number | Used size in GB |
| used_pct | number | Used percentage |

---

For which fields are collected but not shown in the report, see [node-inspector-collection-gaps.md](node-inspector-collection-gaps.md).
