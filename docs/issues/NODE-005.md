# NODE-005 Node disk usage critical

## Summary

A mount point on the node has reached 90% or more disk usage. This is a critical condition that can lead to node eviction (DiskPressure), pod failures, and inability to write logs or temporary files.

## Severity

Critical

## Symptoms

- Node disk usage table shows Used % >= 90% for a mount point
- Report links this finding to NODE-005
- Node condition DiskPressure may become True if kubelet detects low disk space

## Resolution

1. Act immediately to free space: identify largest consumers with `du`, remove unnecessary files, prune container images and build cache
2. If the node is critical (e.g. control-plane), consider cordoning and evacuating workloads before maintenance
3. Expand the volume or add storage; plan capacity for logs and ephemeral usage
4. Configure appropriate resource limits and log rotation to prevent recurrence

## Example

Report row: `| worker01 | /var | /dev/sdb1 | xfs | 50.0 | 46.0 | 92.0% | Critical [NODE-005](NODE-005.md) |`

## References

- [Node pressure eviction (DiskPressure)](https://kubernetes.io/docs/concepts/scheduling-eviction/node-pressure-eviction/)
- [Reserve resources for system daemons](https://kubernetes.io/docs/tasks/administer-cluster/reserve-compute-resources/)
