# BATCH-002 CronJob job failed

## Summary

The CronJob's last scheduled run failed (lastSuccessfulTime is older than lastScheduleTime or unset). Investigate Job/Pod failure and fix root cause.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows CronJob last run failed
- status.lastSuccessfulTime is older than lastScheduleTime or missing

## Resolution

1. List Jobs for the CronJob: `kubectl get jobs -l job-name=...`; describe the failed Job
2. Check pod logs and events for image, resource, command, or dependency issues
3. Adjust backoffLimit or fix task logic; wait for next run or create a Job manually to verify

## References

- [CronJob limitations](https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/#cron-job-limitations)
- [Job patterns](https://kubernetes.io/docs/concepts/workloads/controllers/job/#job-patterns)
