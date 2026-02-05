# POLICY-002 No LimitRange configured

## Summary

A namespace has no LimitRange. Pods can be created without default requests/limits, leading to uneven scheduling and potential resource exhaustion.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Namespace has no LimitRange
- No LimitRange resource in the namespace

## Resolution

1. Create a LimitRange to set default and max min/max for CPU and memory per container or pod
2. Enforce default requests and limits so all pods have resource constraints
3. Use with ResourceQuota for full namespace resource governance

## References

- [LimitRange](https://kubernetes.io/docs/concepts/policy/limit-range/)
- [Configure default memory requests and limits](https://kubernetes.io/docs/tasks/administer-cluster/manage-resources/memory-default-namespace/)
