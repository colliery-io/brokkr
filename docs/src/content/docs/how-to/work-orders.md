---
title: Launch a Job with a Work Order
---

A stack is state that an agent keeps. A work order is a job that runs one time: a database migration, a backup, a test run. This guide shows how a generator launches a Job on its own agent with the `brokkr` CLI and its own PAK, follows the Job, and reads its result and its log lines. No admin is necessary after the agent is registered with the generator.

For the rules behind the commands, see [Work Orders & Build System](../../explanation/work-orders/#work-orders-for-a-generator). For each route and each number, see the [Work Orders reference](../../reference/work-orders/).

## Before you start

- The `brokkr` binary, version 0.12.0 or later, and a broker of the same version.
- A **generator PAK**. The examples use the generator `team-robin`.
- One agent that is **registered with your generator** and **active**. The examples use the agent `prod-1`. An admin registers an agent with `brokkr register --agent <agent> --generator <generator>` (see [Register Agents with Generators](../agent-registration/)).

Give the CLI the broker and the PAK one time:

```bash
export BROKKR_BROKER_URL=https://broker.example.com
export BROKKR_PAK=<your PAK>
```

The outputs on this page are from one run against the development stack of the repository, on 2026-11-14. The ids are replaced with placeholders such as `<work-order-1>`, and the agent of that stack is shown as `prod-1`. The broker of that run was at `http://localhost:3000`. The two errors under "The agent is not registered with your generator" are from a second run on the same day, against a broker of the repository with no cluster: they need no agent that runs.

## Step 1: Write the Job

A work order of the type `custom` holds Kubernetes YAML. The agent applies it. When the YAML holds a `batch/v1` `Job`, the agent watches the Job to its end, and the result of the Job is the result of the work order.

Save this as `migrate-job.yaml`:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: migrate-2026-11-14
  namespace: default
spec:
  backoffLimit: 0
  ttlSecondsAfterFinished: 600
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: migrate
          image: busybox:1.36
          command:
            - sh
            - -c
            - |
              echo "migration 0042: start"
              sleep 20
              echo "migration 0042: 3 tables changed"
              echo "migration 0042: done"
```

Three rules for the Job:

- Give the Job a `metadata.name`. The agent does not support `generateName`.
- Set `restartPolicy: Never` and a `backoffLimit`. A Job that fails ends the work order with a failure. Brokkr does not run it again.
- Use a new name for each run, or let `ttlSecondsAfterFinished` remove the old Job. The agent applies the YAML; it does not delete a Job of the same name first.

## Step 2: Create the work order

```bash
brokkr work-order create -f migrate-job.yaml --agent prod-1
```

```
created work order <work-order-1> (type custom)
targets: agent "prod-1"
An agent that matches a target claims the work order on its next poll.
To follow it, run: brokkr work-order status <work-order-1>
```

A work order has no start time: it runs as soon as an agent claims it. The agent claims it on its next poll, which is 10 seconds by default. The broker also tells a connected agent that a work order is there. In this run the agent claimed the work order 10 seconds after the creation.

## Step 3: Follow the work order

```bash
brokkr work-order status <work-order-1>
```

While the Job runs:

```
work order <work-order-1>
type: custom
owner: team-robin
created: 2026-11-14T16:17:40.642061+00:00
status: CLAIMED. agent "prod-1" runs it, since 2026-11-14T16:17:50.653204+00:00.
The agent can hold it for 3600 seconds.
To see the lines of its pods, run: brokkr work-order logs <work-order-1>
```

When the Job has ended:

```
work order <work-order-1>
type: custom
owner: team-robin
created: 2026-11-14T16:17:40.642061+00:00
status: COMPLETED at 2026-11-14T16:18:15.749493+00:00. agent "prod-1" ran it. Tries after the first: 0.
result: success
message: Applied 1 resource(s). Job(s) default/migrate-2026-11-14 completed successfully.
To see the lines of its pods, run: brokkr work-order logs <work-order-1>
```

| Status | Meaning |
|--------|---------|
| `PENDING` | No agent has claimed the work order yet. |
| `CLAIMED` | An agent runs it. |
| `RETRY_PENDING` | A try failed, and the broker waits before it puts the work order in the queue again. |
| `COMPLETED` | It ended with success. |
| `FAILED` | It ended with a failure. The `result` line gives the reason. |

`brokkr work-order list` shows the work orders that wait or run. A work order that ended is not in this list.

## Step 4: Read the logs and the events of the Job

The agent sends the log lines of the pods of the Job, and the Kubernetes events about the Job and its pods. You do not need an annotation on the pod.

```bash
brokkr work-order logs <work-order-1>
```

The newest line is first:

```
2026-11-14T16:18:12.534191+00:00 default/migrate-2026-11-14-wv84n/migrate: migration 0042: done
2026-11-14T16:18:12.534180+00:00 default/migrate-2026-11-14-wv84n/migrate: migration 0042: 3 tables changed
2026-11-14T16:17:54.713336+00:00 default/migrate-2026-11-14-wv84n/migrate: migration 0042: start
```

```bash
brokkr work-order events <work-order-1>
```

```
OBSERVED                          TYPE    REASON            OBJECT                        MESSAGE
2026-11-14T16:18:15+00:00         Normal  Completed         Job/migrate-2026-11-14        Job completed
2026-11-14T16:17:52+00:00         Normal  Started           Pod/migrate-2026-11-14-wv84n  Started container migrate
2026-11-14T16:17:52+00:00         Normal  Created           Pod/migrate-2026-11-14-wv84n  Created container: migrate
2026-11-14T16:17:52+00:00         Normal  Pulled            Pod/migrate-2026-11-14-wv84n  Successfully pulled image "busybox:1.36" in 1.385s (1.385s including waiting). Image size: 1906887 bytes.
2026-11-14T16:17:51+00:00         Normal  Pulling           Pod/migrate-2026-11-14-wv84n  Pulling image "busybox:1.36"
2026-11-14T16:17:50.697712+00:00  Normal  Scheduled         Pod/migrate-2026-11-14-wv84n  Successfully assigned default/migrate-2026-11-14-wv84n to brokkr-dev-k3s
2026-11-14T16:17:50+00:00         Normal  SuccessfulCreate  Job/migrate-2026-11-14        Created pod: migrate-2026-11-14-wv84n
```

The lines and the events stay readable after the work order ends, for at most 6 hours. Brokkr is not a log store: send the logs that you must keep to your own log system.

## Step 5: Read the results of earlier work orders

This is the history after the three Jobs of this page:

```bash
brokkr work-order history
```

```
ID                                    TYPE    RESULT   AGENT                          OWNER       ENDED                             MESSAGE
<work-order-3>                        custom  success  prod-1                         team-robin  2026-11-14T16:18:50.042227+00:00  Applied 1 resource(s). Job(s) default/report-2026-11-14 c...
<work-order-2>                        custom  failure  prod-1                         team-robin  2026-11-14T16:18:30.835460+00:00  Job default/backup-2026-11-14 ran and failed: BackoffLimi...
<work-order-1>                        custom  success  prod-1                         team-robin  2026-11-14T16:18:15.749493+00:00  Applied 1 resource(s). Job(s) default/migrate-2026-11-14 ...
```

Use `--result failure` to see only the failures, and `--limit` for fewer rows. For the full message of one work order, use `brokkr work-order status` with its id.

## When a Job fails

This Job prints one line and then fails:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: backup-2026-11-14
  namespace: default
spec:
  backoffLimit: 0
  ttlSecondsAfterFinished: 600
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: backup
          image: busybox:1.36
          command:
            - sh
            - -c
            - |
              echo "backup: the bucket does not answer"
              exit 1
```

```bash
brokkr work-order create -f failing-job.yaml --agent prod-1
brokkr work-order status <work-order-2>
```

The status after the Job ended:

```
work order <work-order-2>
type: custom
owner: team-robin
created: 2026-11-14T16:18:25.790727+00:00
status: FAILED at 2026-11-14T16:18:30.835460+00:00. agent "prod-1" ran it. Tries after the first: 0.
result: failure
message: Job default/backup-2026-11-14 ran and failed: BackoffLimitExceeded: Job has reached the specified backoff limit
To see the lines of its pods, run: brokkr work-order logs <work-order-2>
```

The log lines of the failed Job are still there:

```bash
brokkr work-order logs <work-order-2>
```

```
2026-11-14T16:18:27.852040+00:00 default/backup-2026-11-14-lnqdr/backup: backup: the bucket does not answer
```

## Send a work order to a label

A label reaches each agent that has the label **and** is registered with your generator. Here `label-job.yaml` is a Job like the one of step 1, and the agent `prod-1` has the label `env:prod`. One of those agents claims the work order. An agent of another generator that has the same label does not get it.

```bash
brokkr work-order create -f label-job.yaml --label env:prod
```

```
created work order <work-order-3> (type custom)
targets: label env:prod
An agent that matches a target claims the work order on its next poll.
To follow it, run: brokkr work-order status <work-order-3>
```

```bash
brokkr work-order status <work-order-3>
```

```
work order <work-order-3>
type: custom
owner: team-robin
created: 2026-11-14T16:18:39.994911+00:00
status: COMPLETED at 2026-11-14T16:18:50.042227+00:00. agent "prod-1" ran it. Tries after the first: 0.
result: success
message: Applied 1 resource(s). Job(s) default/report-2026-11-14 completed successfully.
To see the lines of its pods, run: brokkr work-order logs <work-order-3>
```

## Refusals

### The agent is not registered with your generator

A generator sees only the agents that are registered with it. So your PAK cannot find an agent that is not registered by its name. The CLI cannot tell this from a name that no agent has, and the error says both:

```bash
brokkr work-order create -f migrate-job.yaml --agent prod-2
```

```
error: invalid request: no agent that is registered with your generator has the name or id "prod-2". Either no agent has this name or id, or the agent is not registered with your generator. An admin registers an agent with a generator: brokkr register --agent prod-2 --generator <generator-id>. With the id of the agent in --agent, the broker says which of the two it is.
```

With the id of the agent, the broker gives the reason:

```bash
brokkr work-order create -f migrate-job.yaml --agent <agent-id>
```

```
error: agent <agent-id> is not registered with your generator, so the generator cannot send it a work order. The broker created nothing. An admin registers it: brokkr register --agent <agent-id> --generator <generator-id>
```

In both cases the broker created nothing. Ask an admin to register the agent with your generator: each error gives the command. Then create the work order again.

### The work order belongs to another owner

A generator reads and deletes only its own work orders. This is the answer when the generator `team-sam` asks for a work order of `team-robin`:

```bash
brokkr work-order status <work-order-1>
```

```
error: work order <work-order-1> belongs to another owner. Use the PAK of the generator that owns it, or an admin PAK. To see which PAK you use, run: brokkr whoami
```

The answer is the same for a work order that an admin created, also when it runs on your agent. Use the PAK of the generator that created the work order.

## Delete a work order that waits

A work order that no agent claims waits with no time limit. Here the agent `prod-3` is registered with the generator, but it does not run. `pending-job.yaml` is the Job of step 1 with a new name:

```bash
brokkr work-order create -f pending-job.yaml --agent prod-3
brokkr work-order list
```

The list shows the work order that waits:

```
ID                                    TYPE    STATUS   OWNER       AGE
<work-order-4>                        custom  PENDING  team-robin  3s
```

```bash
brokkr work-order delete <work-order-4>
```

```
deleted work order <work-order-4>
```

You can also delete a work order that an agent has claimed. Brokkr does not stop the Job that the agent started: the Job runs to its end in the cluster. A work order that ended stays in the history and cannot be deleted:

```bash
brokkr work-order delete <work-order-1>
```

```
error: work order <work-order-1> has ended (COMPLETED). A work order that ended stays in the log and cannot be deleted.
```

## Set the time limits

| Flag | Default | Meaning |
|------|---------|---------|
| `--claim-timeout <SECONDS>` | 3600 | How long an agent can hold the work order. The agent watches the Job for this time, less 10% (and never less than 60 seconds less). A Job that still runs at the end of that window ends the work order with a failure, and the Job continues in the cluster. |
| `--max-retries <N>` | 3 | How many times the broker tries again after a failure that can be tried again, for example an apply that failed. A Job that ran and failed is not tried again. |
| `--backoff <SECONDS>` | 60 | The base of the wait before a new try. The first new try waits 2 × this value, and the wait doubles for each new try. |

For a Job that runs for 2 hours, give it a longer claim timeout:

```bash
brokkr work-order create -f long-job.yaml --agent prod-1 --claim-timeout 9000
```

See [Timing](../../reference/work-orders/#timing) for each number.

## What a work order does not do

- It has no schedule. For a schedule, put a `CronJob` in a stack.
- It does not expire while it waits.
- It does not put a Job back that someone deleted from the cluster.
- It watches only a `batch/v1` `Job` (and the build of a `build` work order) to its end. The agent applies any other kind and does not watch it, and it sends no pod logs for it.

## Related

- [Work Orders reference](../../reference/work-orders/): the routes, who can call each one, and the timing.
- [CLI reference](../../reference/cli/#work-orders-brokkr-work-order): each flag of `brokkr work-order`.
- [Using the Operator Console](../operator-console/): the Work orders view shows the owner of each work order, and the pod logs and events of its job.
- [Container Builds with Shipwright](../shipwright-builds/): the `build` work type.
