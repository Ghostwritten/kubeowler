# AUTO-001 HPA replica range too narrow

## Summary

An HPA has a min/max replica range that is too narrow (e.g. min equals max or very small range). This limits the benefit of horizontal scaling and can lead to overload or underutilization.

## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: HPA replica range too narrow
- spec.minReplicas and spec.maxReplicas are equal or very close

## Resolution

1. Set maxReplicas high enough to handle peak load; set minReplicas for baseline
2. Use metrics and load tests to choose a reasonable range
3. Ensure the target workload (Deployment/StatefulSet) can scale and has sufficient quota

## References

- [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
- [HPA walkthrough](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale-walkthrough/)
