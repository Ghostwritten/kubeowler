# CTRL-001 Control plane component not ready

## Summary

A control plane component is reported unhealthy (e.g. via ComponentStatus or equivalent). This can affect scheduling, replication, or API availability.


## Severity

Warning

## Example

N/A

## Symptoms

- Report indicates a control plane component is not ready or reports a condition not True
- Control plane component status is abnormal

## Resolution

1. Check component status and logs; control plane components often run as static pods in kube-system
2. Inspect etcd, kube-apiserver, kube-controller-manager, kube-scheduler logs and resources
3. Ensure network and dependencies (e.g. etcd reachability) are OK; restart the failing component if needed (e.g. via static pod manifest)

## References

- [Control plane components](https://kubernetes.io/docs/concepts/overview/components/)
- [Debug control plane](https://kubernetes.io/docs/tasks/debug/debug-cluster/)
