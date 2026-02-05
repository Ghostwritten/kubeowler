# AUTO-003 HPA target workload or metrics issue

## Summary

An HPA targets a workload (scaleTargetRef) that does not exist or is invalid, or the metrics it uses are unavailable. The HPA cannot scale correctly.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: HPA target workload or metrics issue
- HPA status shows no scale target or metric errors

## Resolution

1. Verify scaleTargetRef (apiVersion, kind, name) points to a valid Deployment, StatefulSet, or other scalable resource
2. For resource metrics ensure metrics-server is running and pods have requests set
3. For custom/external metrics ensure the metrics API and adapters are installed and returning data
4. Check HPA events: `kubectl describe hpa <name> -n <ns>`

## References

- [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
- [Resource metrics pipeline](https://kubernetes.io/docs/tasks/debug/debug-cluster/resource-metrics-pipeline/)
