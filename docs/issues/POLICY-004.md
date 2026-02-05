# POLICY-004 Replica count does not satisfy PDB

## Summary

The workload's replica count is lower than the PDB's minAvailable (or does not satisfy maxUnavailable). Voluntary disruptions may violate the PDB or block eviction.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Replica count does not satisfy PDB
- PDB minAvailable is greater than current ready replicas, or maxUnavailable cannot be satisfied

## Resolution

1. Increase the workload replica count so that at least minAvailable replicas are always available (or maxUnavailable can be met)
2. Or relax the PDB (lower minAvailable or increase maxUnavailable) to match actual replica count
3. Reconcile PDB and HPA maxReplicas so scaling does not conflict with PDB

## References

- [Pod Disruption Budgets](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/)
- [Configure PDB](https://kubernetes.io/docs/tasks/run-application/configure-pdb/)
