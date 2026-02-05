# AUTO-005 HPA metric target not configured

## Summary

An HPA metric entry has no target (averageUtilization, averageValue, or value). Without a target the metric cannot be used to compute desired replicas.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: HPA metric missing scaling target
- spec.metrics[].resource or .pods or .object target is not set

## Resolution

1. Set a target for each metric: Resource type typically uses averageUtilization; Pods/Object use averageValue or value
2. Choose sensible values (e.g. CPU 80%) and confirm the HPA can read the metric
3. Re-apply the HPA and check status.conditions and currentMetrics

## References

- [HPA metrics](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/#support-for-metrics-apis)
- [MetricTarget](https://kubernetes.io/docs/reference/kubernetes-api/common-definitions/quantity/)
