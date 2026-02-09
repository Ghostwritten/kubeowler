#!/usr/bin/env bash
# Universal node inspection script for DaemonSet.
# Output convention (for exec or logs):
#   stdout: exactly one JSON object (node_name, resources, services, security, kernel, certificates, etc.).
#   stderr: diagnostics, warnings, errors only. Do not write to stdout except the final JSON.
# When kubeowler runs this via exec, it parses stdout as JSON; use stderr for debugging (e.g. kubectl exec ... /node-check-universal.sh 2>&1).
# Requires host root mounted at /host (no overlay of /proc, /sys, /etc so container can start on older kernels/SELinux).
# Compatible with any Linux host that provides /proc, /sys, and os-release or redhat-release (e.g. RHEL, CentOS, Rocky, Alma, Ubuntu, SUSE).

set -e

# ------------------------------------------------------------------------------
# Log helper: write diagnostics to stderr (never to stdout; stdout is JSON only).
# ------------------------------------------------------------------------------
log_err() { echo "[node-check] $*" >&2; }

# ------------------------------------------------------------------------------
# Environment (injected or default)
# ------------------------------------------------------------------------------
NODE_NAME="${NODE_NAME:-$(hostname 2>/dev/null)}"
TIMESTAMP="${TIMESTAMP:-$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)}"
# Node local time (cluster host time) for report header/filename; use host TZ when /host is mounted
if [ -r /host/etc/localtime ]; then
  TIMESTAMP_LOCAL="${TIMESTAMP_LOCAL:-$(TZ=:/host/etc/localtime date "+%Y-%m-%dT%H:%M:%S%z" 2>/dev/null)}"
else
  TIMESTAMP_LOCAL="${TIMESTAMP_LOCAL:-$(date "+%Y-%m-%dT%H:%M:%S%z" 2>/dev/null)}"
fi

# Host paths for node inspection (when host root mounted at /host; no overlay of /proc,/sys,/etc)
HOST_PROC="${HOST_PROC:-/host/proc}"
HOST_SYS="${HOST_SYS:-/host/sys}"
HOST_ETC="${HOST_ETC:-/host/etc}"
[ ! -d "$HOST_PROC" ] && HOST_PROC="/proc" && HOST_SYS="/sys" && HOST_ETC="/etc"

# ------------------------------------------------------------------------------
# Utility: escape string for JSON
# ------------------------------------------------------------------------------
escape_json() {
  echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\n/ /g'
}

# ------------------------------------------------------------------------------
# Get OS version (host when /host or hostPID; fallbacks for /etc symlink)
# ------------------------------------------------------------------------------
get_os_version() {
  local v=""
  if [ -d /host ] && [ -r /host ]; then
    [ -r /host/etc/os-release ] && v=$(grep -E '^PRETTY_NAME=' /host/etc/os-release 2>/dev/null | cut -d= -f2- | tr -d '"' | head -c 200)
    [ -z "$v" ] && [ -r /host/usr/lib/os-release ] && v=$(grep -E '^PRETTY_NAME=' /host/usr/lib/os-release 2>/dev/null | cut -d= -f2- | tr -d '"' | head -c 200)
  fi
  [ -z "$v" ] && [ -r "$HOST_ETC/os-release" ] && v=$(grep -E '^PRETTY_NAME=' "$HOST_ETC/os-release" 2>/dev/null | cut -d= -f2- | tr -d '"' | head -c 200)
  [ -z "$v" ] && [ -r "$HOST_ETC/redhat-release" ] && v=$(cat "$HOST_ETC/redhat-release" 2>/dev/null | head -c 200)
  [ -z "$v" ] && [ -r "$HOST_PROC/1/root/etc/os-release" ] && v=$(grep -E '^PRETTY_NAME=' "$HOST_PROC/1/root/etc/os-release" 2>/dev/null | cut -d= -f2- | tr -d '"' | head -c 200)
  [ -z "$v" ] && [ -r "$HOST_PROC/1/root/usr/lib/os-release" ] && v=$(grep -E '^PRETTY_NAME=' "$HOST_PROC/1/root/usr/lib/os-release" 2>/dev/null | cut -d= -f2- | tr -d '"' | head -c 200)
  echo "$v"
}

# ------------------------------------------------------------------------------
# Get kernel release from /proc
# ------------------------------------------------------------------------------
get_kernel_version() {
  if [ -r "$HOST_PROC/sys/kernel/osrelease" ]; then
    cat "$HOST_PROC/sys/kernel/osrelease" 2>/dev/null | tr -d '\n'
  fi
}

# ------------------------------------------------------------------------------
# Format uptime seconds as human-readable string
# ------------------------------------------------------------------------------
get_uptime_string() {
  local uptime_sec=0
  [ -r "$HOST_PROC/uptime" ] && uptime_sec=$(awk '{print int($1)}' "$HOST_PROC/uptime" 2>/dev/null)
  [ -z "$uptime_sec" ] || [ "$uptime_sec" -lt 0 ] && return
  if [ "$uptime_sec" -ge 86400 ]; then
    local days=$((uptime_sec / 86400))
    local rest=$((uptime_sec % 86400))
    local hours=$((rest / 3600))
    [ "$hours" -gt 0 ] && echo "${days} day(s) ${hours} hour(s)" || echo "${days} day(s)"
  elif [ "$uptime_sec" -ge 3600 ]; then
    local hours=$((uptime_sec / 3600))
    local rest=$((uptime_sec % 3600))
    local mins=$((rest / 60))
    [ "$mins" -gt 0 ] && echo "${hours} hour(s) ${mins} min" || echo "${hours} hour(s)"
  elif [ "$uptime_sec" -ge 60 ]; then
    echo "$((uptime_sec / 60)) min"
  else
    echo "${uptime_sec} sec"
  fi
}

# ------------------------------------------------------------------------------
# CPU usage % from /proc/stat (sample over ~1s). Sets: cpu_used_pct, cpu_used (used cores).
# ------------------------------------------------------------------------------
gather_cpu_usage() {
  cpu_used_pct=""
  cpu_used=""
  [ ! -r "$HOST_PROC/stat" ] && return
  local line1 line2 id1 id2 t1 t2 total_delta idle_delta pct cores
  line1=$(grep '^cpu ' "$HOST_PROC/stat" 2>/dev/null)
  [ -z "$line1" ] && return
  sleep 1
  line2=$(grep '^cpu ' "$HOST_PROC/stat" 2>/dev/null)
  [ -z "$line2" ] && return
  # cpu user nice system idle iowait irq softirq steal (fields 2-9)
  id1=$(echo "$line1" | awk '{print $5}')
  t1=$(echo "$line1" | awk '{print $2+$3+$4+$5+$6+$7+$8+$9}')
  id2=$(echo "$line2" | awk '{print $5}')
  t2=$(echo "$line2" | awk '{print $2+$3+$4+$5+$6+$7+$8+$9}')
  total_delta=$((t2 - t1))
  idle_delta=$((id2 - id1))
  [ "${total_delta:-0}" -le 0 ] && return
  pct=$(awk "BEGIN {printf \"%.1f\", (1 - ($idle_delta/($total_delta+0.0)))*100}")
  cpu_used_pct="$pct"
  [ -n "$cpu_cores" ] && [ "$cpu_cores" -gt 0 ] && cpu_used=$(awk "BEGIN {printf \"%.2f\", $cpu_cores * ($pct/100)}") || true
}

# ------------------------------------------------------------------------------
# Gather resources (CPU, memory, disk, load, swap). Use /host for disk when mounted.
# Sets: cpu_cores, mem_*, disk_*, load_*, swap_*, res_status, res_detail, cpu_used_pct, cpu_used
# ------------------------------------------------------------------------------
gather_resources() {
  cpu_cores=$(grep -c ^processor "$HOST_PROC/cpuinfo" 2>/dev/null || echo "0")
  gather_cpu_usage
  mem_total_kb=$(awk '/MemTotal:/{print $2}' "$HOST_PROC/meminfo" 2>/dev/null || echo "0")
  mem_avail_kb=$(awk '/MemAvailable:/{print $2}' "$HOST_PROC/meminfo" 2>/dev/null || echo "0")
  mem_used_kb=$((mem_total_kb - mem_avail_kb))
  mem_total_mib=$((mem_total_kb / 1024))
  mem_used_mib=$((mem_used_kb / 1024))
  mem_used_pct="0"
  [ "$mem_total_kb" -gt 0 ] && mem_used_pct=$(awk "BEGIN {printf \"%.1f\", ($mem_used_kb/$mem_total_kb)*100}")

  if [ -d /host ] && [ -r /host ]; then
    root_disk_pct=$(df -P /host 2>/dev/null | awk 'NR==2 {gsub(/%/,""); print $5}' || echo "0")
    eval $(df -P /host 2>/dev/null | awk 'NR>1 {t+=$2; u+=$3} END {printf "disk_total_kb=%s\ndisk_used_kb=%s\n", t+0, u+0}')
  else
    root_disk_pct=$(df -P / 2>/dev/null | awk 'NR==2 {gsub(/%/,""); print $5}' || echo "0")
    eval $(df -P 2>/dev/null | awk 'NR>1 {t+=$2; u+=$3} END {printf "disk_total_kb=%s\ndisk_used_kb=%s\n", t+0, u+0}')
  fi
  disk_total_g="0"
  disk_used_g="0"
  disk_used_pct_num="0"
  [ "${disk_total_kb:-0}" -gt 0 ] && disk_total_g=$(awk "BEGIN {printf \"%.1f\", ${disk_total_kb:-0}/1024/1024}") && disk_used_g=$(awk "BEGIN {printf \"%.1f\", ${disk_used_kb:-0}/1024/1024}") && disk_used_pct_num=$(awk "BEGIN {printf \"%.1f\", (${disk_used_kb:-0}/${disk_total_kb:-0})*100}")

  load_1m=$(awk '{print $1}' "$HOST_PROC/loadavg" 2>/dev/null || echo "")
  load_5m=$(awk '{print $2}' "$HOST_PROC/loadavg" 2>/dev/null || echo "")
  load_15m=$(awk '{print $3}' "$HOST_PROC/loadavg" 2>/dev/null || echo "")

  swap_enabled="false"
  grep -q '^/dev/' "$HOST_PROC/swaps" 2>/dev/null && swap_enabled="true"
  swap_total_kb=$(awk '/SwapTotal:/{print $2}' "$HOST_PROC/meminfo" 2>/dev/null || echo "0")
  swap_free_kb=$(awk '/SwapFree:/{print $2}' "$HOST_PROC/meminfo" 2>/dev/null || echo "0")
  swap_used_kb=$((swap_total_kb - swap_free_kb))
  [ "${swap_used_kb:-0}" -lt 0 ] && swap_used_kb=0
  swap_total_g="0"
  swap_used_g="0"
  swap_used_pct_num="0"
  [ "${swap_total_kb:-0}" -gt 0 ] && swap_total_g=$(awk "BEGIN {printf \"%.2f\", ${swap_total_kb:-0}/1024/1024}") && swap_used_g=$(awk "BEGIN {printf \"%.2f\", ${swap_used_kb:-0}/1024/1024}") && swap_used_pct_num=$(awk "BEGIN {printf \"%.1f\", (${swap_used_kb:-0}/${swap_total_kb:-0})*100}")

  res_status="ok"
  res_detail=""
  [ "$cpu_cores" -eq 0 ] && { res_status="error"; res_detail="cpu_cores unknown"; } || true
  [ "$mem_total_kb" -eq 0 ] && { res_status="error"; res_detail="${res_detail:+$res_detail; }memory unknown"; } || true
  [ -n "$cpu_used_pct" ] && cpu_used_pct_json="$cpu_used_pct" || cpu_used_pct_json="null"
  [ -n "$cpu_used" ] && cpu_used_json="$cpu_used" || cpu_used_json="null"
}

# ------------------------------------------------------------------------------
# Gather per-mount disk usage (df -P). Dedupe by device; keep /dev/* and one root overlay, skip tmpfs/shm/containerd.
# ------------------------------------------------------------------------------
gather_disk_mounts() {
  node_disks_json=""
  local tmpf
  tmpf=$(mktemp 2>/dev/null || echo "/tmp/disk_mounts_$$")
  : > "$tmpf"
  df -P 2>/dev/null | awk 'NR>1 {
    total_kb=$2; used_kb=$3;
    total_g=(total_kb+0)/1024/1024; used_g=(used_kb+0)/1024/1024;
    if (total_kb+0>0) pct=(used_kb/total_kb)*100; else pct=0;
    device=$1; mount=$6; for(i=7;i<=NF;i++) mount=mount" "$i;
    skip=0;
    if (device ~ /^\/dev\//) key=device;
    else if (device == "overlay" && (mount == "/" || mount == "/host" || mount !~ /containerd|rootfs/)) key="overlay_root";
    else if (device == "tmpfs" || device == "shm" || device == "devtmpfs") skip=1;
    else if (device == "overlay") skip=1;
    else key=device;
    if (skip) next;
    if (key in seen) next;
    seen[key]=1;
    gsub(/\\/,"\\\\",device); gsub(/"/,"\\\"",device);
    gsub(/\\/,"\\\\",mount); gsub(/"/,"\\\"",mount);
    printf "{\"device\":\"%s\",\"mount_point\":\"%s\",\"fstype\":\"\",\"total_g\":%.2f,\"used_g\":%.2f,\"used_pct\":%.1f}\n", device, mount, total_g, used_g, pct
  }' >> "$tmpf" 2>/dev/null || true
  if [ -s "$tmpf" ]; then
    node_disks_json=$(paste -sd',' "$tmpf" 2>/dev/null || true)
  fi
  rm -f "$tmpf" 2>/dev/null || true
}

# ------------------------------------------------------------------------------
# Check if a process name exists in host /proc (host-aware, no systemctl).
# Usage: host_proc_running "firewalld" && var="true"
# ------------------------------------------------------------------------------
host_proc_running() {
  local name="$1"
  for f in "$HOST_PROC"/[0-9]*/cmdline; do
    [ -r "$f" ] || continue
    tr '\0' '\n' < "$f" 2>/dev/null | head -1 | grep -qF "$name" && return 0
  done
  return 1
}

# ------------------------------------------------------------------------------
# Gather services (ntp, status). Sets: ntp_synced, svc_status, svc_detail. Runtime comes from Kubernetes API.
# ------------------------------------------------------------------------------
gather_services() {
  ntp_synced="false"
  # Host-aware: check NTP daemon process on host
  if host_proc_running "chronyd" || host_proc_running "ntpd" || host_proc_running "systemd-timesyncd"; then
    ntp_synced="true"
  elif command -v timedatectl &>/dev/null; then
    timedatectl status 2>/dev/null | grep -qi 'NTP synchronized: yes' && ntp_synced="true" || true
  elif command -v chronyc &>/dev/null; then
    chronyc tracking 2>/dev/null | grep -q 'Leap status.*Normal' && ntp_synced="true" || true
  fi

  journald_active="false"
  if host_proc_running "systemd-journald"; then
    journald_active="true"
  fi

  crontab_present="false"
  if host_proc_running "crond"; then
    crontab_present="true"
  fi

  kubelet_running="false"
  if host_proc_running "kubelet"; then
    kubelet_running="true"
  fi

  container_runtime_running="false"
  if host_proc_running "containerd" || host_proc_running "dockerd" || host_proc_running "crio"; then
    container_runtime_running="true"
  fi

  svc_status="ok"
  svc_detail=""
}

# ------------------------------------------------------------------------------
# Gather security (SELinux, firewalld, IPVS). SELinux: getenforce or /sys/fs/selinux/enforce (host when /sys hostPath).
# ------------------------------------------------------------------------------
gather_security() {
  selinux_val="unknown"
  # Prefer host's SELinux state from $HOST_SYS (container getenforce would report container context)
  if [ -r "$HOST_SYS/fs/selinux/enforce" ]; then
    local e
    e=$(cat "$HOST_SYS/fs/selinux/enforce" 2>/dev/null | tr -d ' \n')
    case "$e" in
      1) selinux_val="Enforcing" ;;
      0) selinux_val="Permissive" ;;
      *) ;;
    esac
  fi
  if [ "$selinux_val" = "unknown" ] && [ -r "$HOST_SYS/fs/selinux/status" ]; then
    grep -q '^SELinux status:.*disabled' "$HOST_SYS/fs/selinux/status" 2>/dev/null && selinux_val="Disabled" || true
  fi
  # When selinuxfs not mounted (common when SELinux disabled on RHEL/CentOS), infer Disabled
  if [ "$selinux_val" = "unknown" ] && [ ! -d "$HOST_SYS/fs/selinux" ]; then
    selinux_val="Disabled"
  fi
  # Fallback: check /etc/selinux/config for SELINUX=disabled
  if [ "$selinux_val" = "unknown" ] && [ -r "$HOST_ETC/selinux/config" ]; then
    grep -qE '^SELINUX=disabled' "$HOST_ETC/selinux/config" 2>/dev/null && selinux_val="Disabled" || true
  fi
  if [ "$selinux_val" = "unknown" ] && command -v getenforce &>/dev/null; then
    selinux_val=$(getenforce 2>/dev/null || echo "unknown")
  fi
  firewalld_active="false"
  if host_proc_running "firewalld"; then
    firewalld_active="true"
  fi
  ipvs_loaded="false"
  cat "$HOST_PROC/modules" 2>/dev/null | grep -q ip_vs && ipvs_loaded="true" || true
  br_netfilter_loaded="false"
  cat "$HOST_PROC/modules" 2>/dev/null | grep -q br_netfilter && br_netfilter_loaded="true" || true
  overlay_loaded="false"
  cat "$HOST_PROC/modules" 2>/dev/null | grep -qE '^overlay\s|^overlayfs\s' && overlay_loaded="true" || true
  nf_conntrack_loaded="false"
  nf_conntrack_count=""
  nf_conntrack_max=""
  if cat "$HOST_PROC/modules" 2>/dev/null | grep -q nf_conntrack; then
    nf_conntrack_loaded="true"
    [ -r "$HOST_PROC/sys/net/netfilter/nf_conntrack_count" ] && nf_conntrack_count=$(cat "$HOST_PROC/sys/net/netfilter/nf_conntrack_count" 2>/dev/null | tr -d '\n')
    [ -r "$HOST_PROC/sys/net/netfilter/nf_conntrack_max" ] && nf_conntrack_max=$(cat "$HOST_PROC/sys/net/netfilter/nf_conntrack_max" 2>/dev/null | tr -d '\n')
  fi
  sec_status="ok"
  sec_detail=""
}

# ------------------------------------------------------------------------------
# Gather stability: inode usage (root or /host), OOM count (vmstat), file-nr (open/max).
# ------------------------------------------------------------------------------
gather_stability() {
  inode_used_pct=""
  if [ -d /host ] && [ -r /host ]; then
    inode_used_pct=$(df -i /host 2>/dev/null | awk 'NR==2 {gsub(/%/,""); print $5}' || true)
  else
    inode_used_pct=$(df -i / 2>/dev/null | awk 'NR==2 {gsub(/%/,""); print $5}' || true)
  fi
  [ -z "$inode_used_pct" ] && inode_used_pct="null" || true

  oom_kill_count="null"
  if [ -r "$HOST_PROC/vmstat" ]; then
    oom_kill_count=$(grep '^oom_kill ' "$HOST_PROC/vmstat" 2>/dev/null | awk '{print $2}' || true)
  fi
  [ -z "$oom_kill_count" ] && oom_kill_count="null" || true

  file_nr_open=""
  file_nr_max=""
  if [ -r "$HOST_PROC/sys/fs/file-nr" ]; then
    file_nr_open=$(awk '{print $1}' "$HOST_PROC/sys/fs/file-nr" 2>/dev/null || true)
    file_nr_max=$(awk '{print $3}' "$HOST_PROC/sys/fs/file-nr" 2>/dev/null || true)
  fi
  [ -z "$file_nr_open" ] && file_nr_open="null" || true
  [ -z "$file_nr_max" ] && file_nr_max="null" || true
}

# ------------------------------------------------------------------------------
# Gather kernel sysctl. Sets: sysctl_forward, sysctl_swappiness, sysctl_somaxconn, ker_status, ker_detail
# ------------------------------------------------------------------------------
gather_kernel_sysctl() {
  sysctl_forward=""
  sysctl_swappiness=""
  sysctl_somaxconn=""
  # Read host kernel sysctl from $HOST_PROC/sys (container sysctl would report container view)
  [ -r "$HOST_PROC/sys/net/ipv4/ip_forward" ] && sysctl_forward=$(cat "$HOST_PROC/sys/net/ipv4/ip_forward" 2>/dev/null | tr -d '\n') || true
  [ -r "$HOST_PROC/sys/vm/swappiness" ] && sysctl_swappiness=$(cat "$HOST_PROC/sys/vm/swappiness" 2>/dev/null | tr -d '\n') || true
  [ -r "$HOST_PROC/sys/net/core/somaxconn" ] && sysctl_somaxconn=$(cat "$HOST_PROC/sys/net/core/somaxconn" 2>/dev/null | tr -d '\n') || true
  ker_status="ok"
  ker_detail=""
}

# ------------------------------------------------------------------------------
# Count zombie processes (state Z) in /proc
# ------------------------------------------------------------------------------
count_zombie_processes() {
  local count=0
  for stat in "$HOST_PROC"/[0-9]*/stat; do
    [ -r "$stat" ] || continue
    [ "$(awk '{print $3}' "$stat" 2>/dev/null)" = "Z" ] && count=$((count + 1))
  done 2>/dev/null
  echo "$count"
}

# ------------------------------------------------------------------------------
# Compute issue_count from res/svc/sec/ker status and zombie_count
# ------------------------------------------------------------------------------
compute_issue_count() {
  issue_count=0
  [ "$res_status" != "ok" ] && issue_count=$((issue_count + 1)) || true
  [ "$svc_status" != "ok" ] && issue_count=$((issue_count + 1)) || true
  [ "$sec_status" != "ok" ] && issue_count=$((issue_count + 1)) || true
  [ "$ker_status" != "ok" ] && issue_count=$((issue_count + 1)) || true
  [ "$zombie_count" -gt 0 ] && issue_count=$((issue_count + 1)) || true
}

# ------------------------------------------------------------------------------
# Collect certificate paths from kube/etcd process cmdlines; write path|pid to temp file
# ------------------------------------------------------------------------------
collect_cert_paths_from_proc() {
  local cert_flags="client-ca-file tls-cert-file etcd-certfile etcd-cafile cert-file trusted-ca-file peer-cert-file peer-trusted-ca-file"
  local key_flags="key-file keyfile private-key etcd-keyfile peer-key-file"
  for proc in "$HOST_PROC"/[0-9]*; do
    [ -d "$proc" ] || continue
    local pid="${proc##*/}"
    [ ! -r "$proc/cmdline" ] && continue
    [ ! -d "$proc/root" ] && continue
    [ ! -r "$proc/root" ] && continue
    local cmdline
    cmdline=$(tr '\0' '\n' < "$proc/cmdline" 2>/dev/null)
    local first_arg
    first_arg=$(echo "$cmdline" | head -1)
    case "$first_arg" in
      *kube-apiserver*|*kubelet*|*etcd*|*kube-controller*|*kube-scheduler*) ;;
      *) continue ;;
    esac
    local take_next=0
    local flag_name=""
    while IFS= read -r arg; do
      if [ "$take_next" -eq 1 ]; then
        take_next=0
        local path="$arg"
        for k in $key_flags; do [ "$flag_name" = "$k" ] && path="" && break; done
        [ -z "$path" ] && continue
        case "$path" in
          /*) ;;
          *) path="/$path"; local resolved=$(readlink -f "$proc/root/$path" 2>/dev/null); [ -n "$resolved" ] && path="$resolved";;
        esac
        printf '%s|%s\n' "$path" "$pid" >> "$cert_paths_file"
        continue
      fi
      case "$arg" in
        --*=*)
          local flag="${arg%%=*}"
          flag="${flag#--}"
          local path="${arg#*=}"
          for k in $key_flags; do echo "$flag" | grep -q "$k" && path="" && break; done
          [ -z "$path" ] && continue
          local found=0
          for f in $cert_flags; do [ "$flag" = "$f" ] && found=1 && break; done
          [ "$found" -eq 0 ] && continue
          case "$path" in
            /*) ;;
            *) path="/$path"; resolved=$(readlink -f "$proc/root/$path" 2>/dev/null); [ -n "$resolved" ] && path="$resolved";;
          esac
          printf '%s|%s\n' "$path" "$pid" >> "$cert_paths_file"
          ;;
        --*)
          flag="${arg#--}"
          found=0
          for f in $cert_flags; do [ "$flag" = "$f" ] && found=1 && break; done
          [ "$found" -eq 1 ] && take_next=1 && flag_name="$flag"
          ;;
      esac
    done <<< "$cmdline"
  done 2>/dev/null
}

# ------------------------------------------------------------------------------
# Fallback: add fixed certificate paths (when hostPath exposes /etc). pid=0 => read path directly.
# ------------------------------------------------------------------------------
collect_cert_paths_fixed() {
  local dir
  for dir in "$HOST_ETC/kubernetes/ssl" "$HOST_ETC/ssl/etcd/ssl"; do
    [ ! -d "$dir" ] && continue
    local f
    for f in "$dir"/*.crt "$dir"/*.pem; do
      [ -e "$f" ] || [ -L "$f" ] || continue
      [ -r "$f" ] || continue
      printf '%s|0\n' "$f" >> "$cert_paths_file"
    done
  done 2>/dev/null
}

# ------------------------------------------------------------------------------
# Build node_certificates JSON array from path|pid file (openssl expiry check)
# When pid=0 or /proc/pid/root not readable, try reading path directly (fixed-path fallback).
# ------------------------------------------------------------------------------
build_certificates_json() {
  node_certificates_json=""
  if ! command -v openssl &>/dev/null; then
    return
  fi
  sort -u -t'|' -k1,1 "$cert_paths_file" 2>/dev/null | while IFS='|' read -r path pid; do
    [ -z "$path" ] && continue
    local path_noslash="${path#/}"
    local read_path=""
    if [ -n "$pid" ] && [ "$pid" != "0" ]; then
      read_path="$HOST_PROC/${pid}/root/${path_noslash}"
      [ -r "$read_path" ] || read_path=""
    fi
    [ -z "$read_path" ] && read_path="$path"
    [ ! -r "$read_path" ] && continue
    local enddate_raw
    enddate_raw=$(openssl x509 -noout -enddate -in "$read_path" 2>/dev/null) || continue
    [ -z "$enddate_raw" ] && continue
    local enddate_str="${enddate_raw#notAfter=}"
    local end_ts cur_ts days_remaining=0 status="Valid"
    end_ts=$(date -d "$enddate_str" +%s 2>/dev/null) || true
    cur_ts=$(date +%s 2>/dev/null) || true
    [ -n "$end_ts" ] && [ -n "$cur_ts" ] && days_remaining=$(( (end_ts - cur_ts) / 86400 )) || true
    [ "$days_remaining" -lt 0 ] && status="Expired" || true
    [ "$days_remaining" -ge 0 ] && [ "$days_remaining" -lt 30 ] && status="Expiring soon" || true
    local path_escaped end_escaped
    path_escaped=$(echo "$path" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\n/ /g')
    end_escaped=$(echo "$enddate_str" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\n/ /g')
    printf '{"path":"%s","expiration_date":"%s","days_remaining":%s,"status":"%s"}\n' "$path_escaped" "$end_escaped" "$days_remaining" "$status" >> "$cert_json_file"
  done
  [ -s "$cert_json_file" ] && node_certificates_json=$(paste -sd',' "$cert_json_file") || true
}

# ------------------------------------------------------------------------------
# Emit single JSON object to stdout
# ------------------------------------------------------------------------------
emit_node_inspection_json() {
  local res_d svc_d sec_d ker_d
  res_d=$(escape_json "$res_detail")
  svc_d=$(escape_json "$svc_detail")
  sec_d=$(escape_json "$sec_detail")
  ker_d=$(escape_json "$ker_detail")
  cat <<EOF
{
  "node_name": "$(escape_json "$NODE_NAME")",
  "hostname": "$(escape_json "$NODE_NAME")",
  "timestamp": "$TIMESTAMP",
  "timestamp_local": "$(escape_json "${TIMESTAMP_LOCAL:-}")",
  "runtime": "",
  "os_version": "$(escape_json "$os_version")",
  "kernel_version": "$(escape_json "$kernel_version")",
  "uptime": "$(escape_json "$uptime_str")",
  "resources": {
    "cpu_cores": $cpu_cores,
    "cpu_used": ${cpu_used_json:-null},
    "cpu_used_pct": ${cpu_used_pct_json:-null},
    "memory_total_mib": $mem_total_mib,
    "memory_used_mib": $mem_used_mib,
    "memory_used_pct": $mem_used_pct,
    "root_disk_pct": ${root_disk_pct:-0},
    "disk_total_g": ${disk_total_g:-0},
    "disk_used_g": ${disk_used_g:-0},
    "disk_used_pct": ${disk_used_pct_num:-0},
    "load_1m": "$(escape_json "$load_1m")",
    "load_5m": "$(escape_json "$load_5m")",
    "load_15m": "$(escape_json "$load_15m")",
    "swap_enabled": $swap_enabled,
    "swap_total_g": ${swap_total_g:-0},
    "swap_used_g": ${swap_used_g:-0},
    "swap_used_pct": ${swap_used_pct_num:-0},
    "status": "$res_status",
    "detail": "$res_d"
  },
  "services": {
    "runtime": "",
    "ntp_synced": $ntp_synced,
    "journald_active": $journald_active,
    "crontab_present": $crontab_present,
    "kubelet_running": $kubelet_running,
    "container_runtime_running": $container_runtime_running,
    "status": "$svc_status",
    "detail": "$svc_d"
  },
  "security": {
    "selinux": "$(escape_json "$selinux_val")",
    "firewalld_active": $firewalld_active,
    "ipvs_loaded": $ipvs_loaded,
    "br_netfilter_loaded": $br_netfilter_loaded,
    "overlay_loaded": $overlay_loaded,
    "nf_conntrack_loaded": $nf_conntrack_loaded,
    "nf_conntrack_count": ${nf_conntrack_count:-null},
    "nf_conntrack_max": ${nf_conntrack_max:-null},
    "status": "$sec_status",
    "detail": "$sec_d"
  },
  "kernel": {
    "net_ipv4_ip_forward": "$(escape_json "$sysctl_forward")",
    "vm_swappiness": "$(escape_json "$sysctl_swappiness")",
    "net_core_somaxconn": "$(escape_json "$sysctl_somaxconn")",
    "status": "$ker_status",
    "detail": "$ker_d"
  },
  "stability": {
    "inode_used_pct": ${inode_used_pct:-null},
    "oom_kill_count": ${oom_kill_count:-null},
    "file_nr_open": ${file_nr_open:-null},
    "file_nr_max": ${file_nr_max:-null}
  },
  "container_state_counts": ${container_states_json},
  "zombie_count": $zombie_count,
  "issue_count": $issue_count,
  "node_certificates": [${node_certificates_json}],
  "node_disks": [${node_disks_json:-}]
}
EOF
}

# ------------------------------------------------------------------------------
# Main (runtime is reported via Kubernetes API Node.status.nodeInfo.containerRuntimeVersion)
# ------------------------------------------------------------------------------
os_version=$(get_os_version)
kernel_version=$(get_kernel_version)
uptime_str=$(get_uptime_string)
gather_resources
gather_disk_mounts
gather_services
gather_security
gather_stability
gather_kernel_sysctl
zombie_count=$(count_zombie_processes)
compute_issue_count

# container_state_counts: left empty here; kubeowler fills it from Kubernetes API (list pods, aggregate by node).
container_states_json="{}"
cert_paths_file=$(mktemp 2>/dev/null || echo "/tmp/cert_paths_$$")
cert_json_file=$(mktemp 2>/dev/null || echo "/tmp/cert_json_$$")
: > "$cert_paths_file"
: > "$cert_json_file"
collect_cert_paths_from_proc
collect_cert_paths_fixed
build_certificates_json
rm -f "$cert_paths_file" "$cert_json_file" 2>/dev/null

emit_node_inspection_json
