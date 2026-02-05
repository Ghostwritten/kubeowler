# NODE-001 Node not ready

## Summary

A node is reported as not ready when its kubelet has not reported Ready status to the API server, or when the node condition `Ready` is False. Pods on not-ready nodes may not be scheduled or may not run correctly, affecting cluster availability.

## Severity

Critical

## Example

N/A

## Symptoms

- Report shows: Node &lt;name&gt; is not ready
- `kubectl get nodes` shows the node STATUS other than Ready
- Node Conditions show `type=Ready, status=False` or unknown

## Resolution

1. Log onto the node and check the kubelet service: `systemctl status kubelet`
2. View kubelet logs: `journalctl -u kubelet -f`
3. Verify node resources (CPU, memory, disk) are sufficient for kubelet and the container runtime
4. Check networking: node must reach the API server, DNS, and required ports
5. On cloud instances, ensure provider metadata/credentials and network policies do not block kubelet

## References

- [Node status and conditions](https://kubernetes.io/docs/concepts/architecture/nodes/#node-status)
- [Debug worker nodes](https://kubernetes.io/docs/tasks/debug/debug-cluster/debug-worker-nodes/)
