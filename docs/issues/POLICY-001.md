# POLICY-001 No ResourceQuota configured

## Summary

A namespace has no ResourceQuota. In multi-tenant or shared clusters this can allow one tenant to consume excessive resources and impact others.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Namespace has no ResourceQuota
- No ResourceQuota resource in the namespace

## Resolution

1. Create a ResourceQuota for the namespace limiting total CPU, memory, PVCs, etc.
2. Align limits with cluster capacity and tenant expectations
3. Use with LimitRange for predictable defaults per pod/container

## References

- [Resource quotas](https://kubernetes.io/docs/concepts/policy/resource-quotas/)
- [Quota per namespace](https://kubernetes.io/docs/tasks/administer-cluster/manage-resources/quota-memory-cpu-namespace/)
