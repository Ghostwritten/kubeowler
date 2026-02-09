# Node Inspector: Limitations

This document describes limitations of the Node Inspector DaemonSet and how host visibility affects the JSON output. When the DaemonSet is configured with **hostPID: true** and the host root mounted read-only at **/host**, the script can read host os_version, host root disk (`df -P /host`), and discover node_certificates from host processes. **OS Version / Kernel Version in the report:** when the Kubernetes API provides `Node.status.nodeInfo.osImage` and `kernelVersion`, the report uses those; otherwise it uses the DaemonSet script values (os_version, kernel_version).

---

## 1. Field behavior in the Pod-generated JSON

| Field | Description |
|-------|-------------|
| **os_version** | When `/host` is mounted: read from `/host/etc/os-release`, `/host/usr/lib/os-release`. Otherwise fallback: `/etc/os-release`, `/etc/redhat-release`, `/proc/1/root/etc/os-release`, `/proc/1/root/usr/lib/os-release`. With hostPID, `/proc/1` is the host init, so host OS can be read. |
| **runtime** | Depends on systemctl or runtime socket; inside the DaemonSet container it is often `unknown` (image has no systemctl, no socket mounted). |
| **services.journald_active / crontab_present / ntp_synced** | Often false or container view when running inside the container (container’s own state or commands unavailable). |
| **security.selinux / firewalld_active** | Depend on getenforce/systemctl; in the container often unknown/false. |
| **node_certificates** | With **hostPID: true**, certificate paths are parsed from host process cmdlines (e.g. kube-apiserver, kubelet, etcd); expiration is checked via `/proc/<pid>/root/<path>` and openssl. |
| **resources.root_disk_pct / disk_total_g / disk_used_g** | When `/host` is mounted, `df -P /host` gives the **host root disk**; otherwise the container root disk. |

**Always from host /proc, /sys (when hostPID/host mounts are used):** kernel_version, uptime, cpu_cores, memory_*, load_*, swap_*, kernel sysctls, ipvs_loaded, zombie_count.

---

## 2. Remaining limitations (default or generic)

| Category | Description |
|----------|-------------|
| **hostNetwork** | Not enabled by default; host IPs are filled in the report by Kubeowler from Node.status.addresses. |
| **System interfaces** | If the image has no systemctl/getenforce, etc., runtime, journald, crontab, ntp, selinux, and firewalld may be untestable or reflect the container view. |
| **Runtime socket** | Without containerd/docker socket mounted, runtime detection is usually unknown. |

---

## 3. Configurations used in deploy/node-inspector/daemonset.yaml

- **hostPID: true** — Container shares the host PID namespace; host processes can be discovered and `/proc/<pid>/root` used to read certificates and host os-release.
- **Host root mount** — Host root is mounted read-only at `/host`; when present, the script uses `df -P /host` for host root disk and reads os_version from `/host/etc/os-release`, `/host/usr/lib/os-release`.

The script (`scripts/node-check-universal.sh`) implements detection and fallback for `/host`; report output is in English.

---

## 4. Node inspector: supported host operating systems

The node inspection script runs **inside the DaemonSet container** (Alpine-based). It does not execute commands on the host; it only **reads** host paths via the read-only mount (`/host`) and, when `hostPID: true`, via `$HOST_PROC` (host `/proc`).

**Requirements for the host (node):**

| Requirement | Description |
|-------------|-------------|
| **Kernel** | Linux; the host must expose **/proc** and **/sys** so that the container can read them (via `/host/proc`, `/host/sys` or the script’s fallback when `/host` is not used). |
| **OS version** | The script detects OS from **os-release** or **redhat-release**: `/host/etc/os-release`, `/host/usr/lib/os-release`, or `$HOST_ETC/os-release`, `$HOST_ETC/redhat-release`. Any distribution providing one of these works. |
| **No host commands** | The script does **not** rely on host binaries (e.g. systemctl, getenforce). Service and security state are inferred from host **process names** (via `/proc` cmdline) and from files under `/proc` and `/sys`. |

**Alignment with README / installation:** The same operating systems listed in [README.md](../README.md) and [installation.md](installation.md) are supported for the node inspector: **RHEL 7+, CentOS 7.x, Rocky Linux 8+, AlmaLinux 8+, Ubuntu 18.04+, SUSE / openSUSE, OpenAnolis, Kylin**, and other compatible Linux distributions that provide `/proc`, `/sys`, and os-release or redhat-release.

---

For the JSON schema and collection vs. report usage, see [node-inspection-schema.md](node-inspection-schema.md) and [node-inspector-collection-gaps.md](node-inspector-collection-gaps.md).
