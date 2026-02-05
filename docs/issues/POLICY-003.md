# POLICY-003 Critical workload has no PDB

## Summary

A critical workload (e.g. production Deployment) has no PodDisruptionBudget (PDB). During voluntary disruptions (node drain, cluster upgrade) pods may be evicted without a minimum availability guarantee.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Critical workload has no PDB
- No PDB targets the workload's pods

## Resolution

1. Create a PodDisruptionBudget that selects the workload's pods (e.g. by label)
2. Set minAvailable or maxUnavailable based on desired availability (e.g. minAvailable: 1 for single-replica critical app)
3. Ensure replica count is high enough to satisfy the PDB during drains

## References

- [Pod Disruption Budgets](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/)
- [Specifying a disruption budget](https://kubernetes.io/docs/tasks/run-application/configure-pdb/)
