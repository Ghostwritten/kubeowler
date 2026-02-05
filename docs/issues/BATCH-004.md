# BATCH-004 Job needs backoffLimit or resource check

## Summary

A Job has failed or is stuck; backoffLimit may be too low, or resource limits/requests may be causing OOM or scheduling failures. Adjust backoffLimit or resources and fix the underlying failure.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Job needs backoffLimit or resource check
- Job has failed or not completed; pods may be OOMKilled or Pending

## Resolution

1. Describe the Job and its pods; check events and container exit codes
2. Increase backoffLimit if transient failures are expected; fix the cause if possible
3. Set or increase resource requests/limits to avoid OOM and ensure schedulability
4. Use activeDeadlineSeconds to cap total Job duration if needed

## References

- [Jobs](https://kubernetes.io/docs/concepts/workloads/controllers/job/)
- [Job patterns](https://kubernetes.io/docs/concepts/workloads/controllers/job/#job-patterns)
