# NODE-004 Node disk usage high (Warning)

## Summary

A mount point on the node has reached 60% or more disk usage (below the 90% critical threshold). This is a warning level condition; if usage continues to grow, the node may hit critical levels (90%+, NODE-005) and trigger evictions or failures. The report uses: **Info** for &lt;60%, **Warning** (this code) for 60%–&lt;90%, **Critical** (NODE-005) for ≥90%.

## Severity

Warning

## Symptoms

- Node disk usage table shows Used % in the 60%–&lt;90% range for a mount point (e.g. `/`, `/var`)
- Report links this finding to NODE-004

## Resolution

1. Identify large directories on the node (e.g. `du -h --max-depth=1 /` for the reported mount)
2. Clean up logs, temporary files, or unused images (e.g. crictl rmi --prune, clear journal)
3. Extend the volume or add additional storage if the workload is expected to grow
4. Consider log rotation and retention for application and system logs

## Example

N/A (report shows mount point, device, total/used Gi, used %, and status link to this doc)

## References

- [Node pressure eviction (DiskPressure)](https://kubernetes.io/docs/concepts/scheduling-eviction/node-pressure-eviction/)
- [Kubelet garbage collection](https://kubernetes.io/docs/concepts/architecture/garbage-collection/)
