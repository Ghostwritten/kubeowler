# AUTO-002 HPA has no metrics configured

## Summary

An HPA has no metrics defined under spec.metrics. Without metrics the HPA cannot compute desired replicas and will not scale.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: HPA has no metrics configured
- spec.metrics is empty or missing

## Resolution

1. Add at least one metric: resource (e.g. cpu, memory), pods, or object
2. For resource metrics ensure metrics-server (or equivalent) is installed
3. Set target type and value (e.g. averageUtilization for CPU)
4. Verify the HPA can read the metric: check HPA status and events

## References

- [HPA metrics](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/#support-for-metrics-apis)
- [Metrics server](https://kubernetes.io/docs/tasks/debug/debug-cluster/resource-metrics-pipeline/)
