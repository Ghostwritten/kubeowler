# BATCH-005 Job Pod stuck or timeout adjustment needed

## Summary

A Job's pods are stuck (e.g. Pending, CrashLoopBackOff) or the Job is taking longer than expected. Timeout (activeDeadlineSeconds) or resource/scheduling issues may need adjustment.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows: Job Pod stuck or timeout adjustment needed
- Job pods are not completing or Job exceeds expected duration

## Resolution

1. Check pod status and events: `kubectl describe pod` for each Job pod
2. If pods are Pending, address scheduling (resources, affinity, PVCs)
3. If failing, fix image/command/resources and consider higher backoffLimit
4. Set or increase activeDeadlineSeconds to fail the Job after a max duration; tune for your workload

## References

- [Jobs](https://kubernetes.io/docs/concepts/workloads/controllers/job/)
- [Debug pods](https://kubernetes.io/docs/tasks/debug/debug-application/debug-pods/)
