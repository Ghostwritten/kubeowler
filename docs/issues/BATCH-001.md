# BATCH-001 CronJob suspended

## Summary

A CronJob with spec.suspend true does not create Jobs on schedule. Keep suspended only if intended; otherwise resume.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows CronJob is suspended
- spec.suspend is true

## Resolution

1. To resume scheduled runs: set spec.suspend to false
2. If no longer needed, delete the CronJob or keep suspended and document the reason

## References

- [CronJobs](https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/)
- [Automated tasks with CronJobs](https://kubernetes.io/docs/tasks/job/automated-tasks-with-cron-jobs/)
