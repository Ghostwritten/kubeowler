# NODE-003 Zombie processes on node

## Summary

A node has zombie processes when one or more processes are in the zombie (defunct) state (state Z in `/proc`). Zombie processes have exited but their parent has not reaped them; they consume a PID slot and, in large numbers, can contribute to PID exhaustion (PIDPressure) or indicate buggy or stuck parent processes.

## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Node &lt;name&gt; has N zombie process(es)
- Node Inspection "Node 进程健康" table shows a non-zero zombie count and issue code [NODE-003](NODE-003.md)
- On the node, `ps` or inspection of `/proc/*/stat` shows processes in state Z

## Resolution

1. Identify zombie processes on the node: `ps aux | grep Z` or inspect `/proc/<pid>/stat` (second field is state).
2. Find the parent process (PPID) of each zombie; the parent is responsible for reaping. If the parent is a system component (e.g. kubelet, container runtime), check its logs and consider restarting it under controlled maintenance.
3. If many zombies accumulate, check for PID limits (`podPidsLimit`, node capacity) and consider tuning or fixing the parent process to reap children correctly.
4. Restarting the parent process will reap zombies; avoid killing zombie processes directly (they cannot be signalled).

## References

- [Node status and conditions](https://kubernetes.io/docs/concepts/architecture/nodes/#node-status)
- [Node pressure eviction (PIDPressure)](https://kubernetes.io/docs/concepts/scheduling-eviction/node-pressure-eviction/)
