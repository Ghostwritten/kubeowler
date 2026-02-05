# NODE-002 Node has resource pressure

## Summary

A node has resource pressure when the kubelet reports MemoryPressure, DiskPressure, or PIDPressure as True. Under pressure the node may stop scheduling new pods or evict existing ones, affecting workload stability.

## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Node &lt;name&gt; has MemoryPressure / DiskPressure / PIDPressure
- Node Conditions show the corresponding type with status True

## Resolution

1. **MemoryPressure**: Add node memory or reduce pod resource requests/limits on the node; investigate memory leaks or high-memory processes
2. **DiskPressure**: Clean images, logs, and temporary files; expand disk or add storage; tune kubelet `imageGCHighThresholdPercent` etc.
3. **PIDPressure**: Limit PIDs per pod on the node (e.g. `podPidsLimit`); investigate process leaks
4. Use `kubectl describe node <name>` to inspect Allocatable and Capacity; use metrics to identify the source of pressure

## References

- [Node pressure eviction](https://kubernetes.io/docs/concepts/scheduling-eviction/node-pressure-eviction/)
- [Node status and conditions](https://kubernetes.io/docs/concepts/architecture/nodes/#node-status)
