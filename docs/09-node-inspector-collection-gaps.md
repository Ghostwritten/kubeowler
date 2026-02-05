# Node Inspector: Collection vs. Report Usage

This document maps the JSON output of the DaemonSet script `node-check-universal.sh` to Kubeowler's parsing (e.g. `types.rs`) and to the inspection report. It identifies fields that **are collected and parsed but are not currently displayed** in the report.

---

## 1. Root-Level Output (emit_node_inspection_json)

| Level   | Field                   | Type   | Parsed (types.rs) | Shown in report |
|---------|-------------------------|--------|-------------------|-----------------|
| Root    | node_name               | string | Yes               | Yes — all node tables |
| Root    | hostname                | string | Yes               | No — used by collector only to backfill node_name when empty |
| Root    | timestamp               | string | Yes               | No — node-level collection time |
| Root    | runtime                 | string | Yes               | Yes — Node services table |
| Root    | os_version              | string | Yes               | Yes — Node General Information |
| Root    | kernel_version          | string | Yes               | Yes — Node General Information |
| Root    | uptime                  | string | Yes               | Yes — Node General Information |
| Root    | resources               | object | Yes               | See §2 |
| Root    | services                | object | Yes               | See §3 |
| Root    | security                | object | Yes               | See §4 |
| Root    | kernel                  | object | Yes               | See §5 |
| Root    | container_state_counts  | object | Yes               | Yes — populated from K8s API; Node container state counts table |
| Root    | zombie_count            | number | Yes               | Yes — Node process health table |
| Root    | issue_count             | number | Yes               | Yes — used only for node_inspection_status (warning) |
| Root    | node_certificates       | array  | Yes               | Yes — Node Certificate Status table |
| Root    | node_disks              | array  | Yes               | Yes — Node disk usage table |

---

## 2. resources Sub-Object

| Field            | Parsed | Shown in report |
|------------------|--------|-----------------|
| cpu_cores        | Yes    | Yes — Node resources table |
| memory_total_mib | Yes    | Yes — Node resources table |
| memory_used_mib  | Yes    | Yes — Node resources table |
| memory_used_pct  | Yes    | Yes — Node resources table |
| root_disk_pct    | Yes    | No — disk data is shown via node_disks table instead |
| disk_total_g     | Yes    | No — same as above |
| disk_used_g      | Yes    | No — same as above |
| disk_used_pct    | Yes    | No — same as above |
| load_1m / load_5m / load_15m | Yes | Yes — Node resources table |
| swap_enabled     | Yes    | No — only swap totals/used/percentage are shown |
| swap_total_g / swap_used_g / swap_used_pct | Yes | Yes — Node resources table |
| status           | Yes    | Yes — contributes to node status (ok/warning/error) |
| detail           | Yes    | No — only status is used for status derivation |

---

## 3. services Sub-Object

| Field             | Parsed | Shown in report |
|-------------------|--------|-----------------|
| runtime           | Yes    | Yes — Node services table (duplicates root runtime) |
| journald_active   | Yes    | No |
| crontab_present   | Yes    | No |
| ntp_synced        | Yes    | Yes — Node services table |
| status            | Yes    | Yes — contributes to node status |
| detail            | Yes    | No |

---

## 4. security Sub-Object

| Field            | Parsed | Shown in report |
|------------------|--------|-----------------|
| selinux          | Yes    | Yes — Node security table |
| firewalld_active | Yes    | Yes — Node security table |
| ipvs_loaded      | Yes    | Yes — Node security table |
| status           | Yes    | Yes — contributes to node status |
| detail           | Yes    | No |

---

## 5. kernel Sub-Object

| Field                  | Parsed | Shown in report |
|------------------------|--------|-----------------|
| net_ipv4_ip_forward    | Yes    | Yes — Node kernel table |
| vm_swappiness         | Yes    | Yes — Node kernel table |
| net_core_somaxconn    | Yes    | Yes — Node kernel table |
| status                 | Yes    | Yes — contributes to node status |
| detail                 | Yes    | No |

---

## 6. node_certificates (per item)

| Field            | Parsed | Shown in report |
|------------------|--------|-----------------|
| path             | Yes    | Yes — Node Certificate Status table |
| expiration_date  | Yes    | Yes — Node Certificate Status table |
| days_remaining   | Yes    | Yes — Node Certificate Status table |
| status           | Yes    | Partially — only `status == "Expired"` is used to derive an Expired Yes/No column; "Valid" and "Expiring soon" are not shown as a dedicated Status column |

---

## 7. node_disks (per item, gather_disk_mounts)

| Field       | Parsed | Shown in report |
|-------------|--------|-----------------|
| device      | Yes    | Yes — Node disk usage table |
| mount_point | Yes    | Yes — Node disk usage table |
| fstype      | Yes    | Yes — column exists, but script often emits empty string (fstype not parsed from `df -P`) |
| total_g / used_g / used_pct | Yes | Yes — Node disk usage table and NODE-004/NODE-005 checks |

---

## 8. Summary: Not Shown or Indirectly Used

### 8.1 Parsed but not displayed in the report

- **Root:** `hostname`, `timestamp`
- **resources:** `root_disk_pct`, `disk_total_g`, `disk_used_g`, `disk_used_pct`, `swap_enabled`, `detail`
- **services:** `journald_active`, `crontab_present`, `detail`
- **security:** `detail`
- **kernel:** `detail`

### 8.2 Used only for derived state (not as report columns)

- **Sub-object status fields** (`resources.status`, `services.status`, `security.status`, `kernel.status`): used only to compute `node_inspection_status` (ok/warning/error); not shown as literal columns in the report.
- **issue_count:** used only to mark a node as warning; not shown as its own column.
- **Certificate status:** used only to derive the Expired Yes/No column; "Valid" and "Expiring soon" are not shown as a Status column.

### 8.3 Data-source gap

- **node_disks.fstype:** The `gather_disk_mounts` logic in the script does not currently parse filesystem type; the report has an FSType column but it is often empty.

---

## 9. Recommendations

1. **Fuller report coverage:** Consider exposing node-level `timestamp` in Node General Information or a dedicated column; `swap_enabled` in Node resources; `journald_active` and `crontab_present` in Node services; and the various `detail` fields (resources, services, security, kernel) in the relevant sections or summary tables.
2. **Certificate status:** Add a dedicated Status column in the Node Certificate Status table with values "Valid", "Expiring soon", and "Expired".
3. **node_disks.fstype:** In `node-check-universal.sh`, extend `gather_disk_mounts` to parse and emit filesystem type (e.g. from `df -P` or `/proc/mounts`) so the report FSType column is populated.
