# BATCH-003 CronJob schedule or controller issue

## Summary

The CronJob has never been scheduled (lastScheduleTime empty), possibly due to invalid schedule format, timezone, or controller not running.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows CronJob never executed
- status.lastScheduleTime is empty

## Resolution

1. Check spec.schedule format (cron expression) and timezone
2. Ensure CronJob controller and kube-controller-manager are running; check controller logs
3. Ensure spec.suspend is false; create a Job manually if needed to verify the task runs

## References

- [CronJob schedule syntax](https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/#schedule-syntax)
- [Cron expression](https://en.wikipedia.org/wiki/Cron)
