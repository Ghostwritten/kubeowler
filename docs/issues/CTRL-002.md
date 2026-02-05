# CTRL-002 Static Pod not ready

## Summary

A static Pod (e.g. control plane component run by kubelet from a manifest on the node) is not ready. Static pods are critical for control plane nodes.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Static Pod not ready
- Static pod (e.g. in /etc/kubernetes/manifests) is not in Ready state

## Resolution

1. On the node, check the static pod manifest and kubelet logs
2. Verify certificates, volumes, and image pull for the static pod
3. Ensure kubelet is running and has read access to the manifest directory
4. For HA, check other control plane nodes; recover or replace the node if needed

## References

- [Static pods](https://kubernetes.io/docs/tasks/configure-pod-container/static-pod/)
- [Control plane](https://kubernetes.io/docs/concepts/overview/components/#control-plane-components)
