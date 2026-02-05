# AUTO-004 HPA behavior limits scaling

## Summary

HPA scaling behavior (scaleUp/scaleDown) is configured in a way that limits how quickly or how much the HPA can scale. This may be intentional but can also cause slow response to load.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: HPA behavior limits scaling
- spec.behavior has restrictive policies (e.g. very low percent or pod count per period)

## Resolution

1. Review spec.behavior.scaleUp and scaleDown; increase stabilizationWindow or percent if scaling is too slow
2. Use behavior to prevent flapping while keeping response time acceptable
3. Test under load to validate scaling speed and stability
4. Remove or relax behavior if the default behavior is sufficient

## References

- [HPA behavior](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/#support-for-configurable-scaling-behavior)
- [HPAScaleBehavior](https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/horizontal-pod-autoscaler-v2/#horizontalpodautoscalerbehavior)
