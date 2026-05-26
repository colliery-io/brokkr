# ws-loadtest

One-off load-test harness for the internal broker↔agent WebSocket channel
(BROKKR-T-0177 / I-0020 B4). It drives a synthetic agent fleet against a live
broker to establish — and later re-check — a throughput / footprint baseline.

This is **not** part of the workspace build or CI. It's a standalone crate
(note the empty `[workspace]` table in `Cargo.toml`) that you run by hand
against a local stack.

## What it does

1. Provisions a generator, a pool of stacks, and N agents (each with a unique
   PAK) over the REST API using an admin PAK.
2. Opens N agent WS connections to `/internal/ws/agent`. Each sends a
   `heartbeat` every 5s and telemetry (`k8s_event` / `pod_log_line`,
   alternating) at a target per-agent rate. Telemetry is what exercises the
   I-0019 Postgres write path under the 6h retention ceiling.
3. Opens K live-subscriber connections (`/api/v1/stacks/{id}/live`, admin PAK)
   spread across the stack pool to exercise per-stack broadcast fan-out.
4. Every few seconds samples the broker `/metrics` gauge, `docker stats`
   (CPU% / RSS), and the telemetry-table row counts (`docker exec … psql`),
   then prints a summary with achieved rates and peak footprint.

The wire JSON is hand-rolled but mirrors `brokkr_wire::WsMessage` exactly
(external-tagged, snake_case). Correctness is validated empirically: if the
format were wrong the broker would drop the frames and the row counts would
never grow.

> Note: telemetry body `agent_id` must equal the authenticated agent (the
> broker drops mismatches before persist/broadcast), so the tool sends each
> agent's provisioned id, not a synthetic one.

## Running

Bring up the local stack first, then run:

```sh
angreal local up
cargo run --release --manifest-path tools/ws-loadtest/Cargo.toml
```

### Config (env vars, all optional)

| var | default | meaning |
|-----|---------|---------|
| `BROKER_URL` | `http://localhost:3000` | broker base URL |
| `ADMIN_PAK` | dev admin PAK | admin PAK for provisioning + subscribers |
| `LT_AGENTS` | `500` | synthetic agents |
| `LT_STACKS` | `50` | stack pool size |
| `LT_SUBSCRIBERS` | `50` | live subscribers |
| `LT_MSG_RATE` | `10` | telemetry msgs/sec/agent |
| `LT_DURATION_SECS` | `300` | run length |
| `LT_SAMPLE_SECS` | `10` | sampling interval |
| `LT_BROKER_CONTAINER` | `brokkr-dev-broker-1` | container for `docker stats` |
| `LT_PG_CONTAINER` | `brokkr-dev-postgres-1` | container for row-count queries |

Smoke test (quick correctness check):

```sh
LT_AGENTS=20 LT_STACKS=5 LT_SUBSCRIBERS=5 LT_DURATION_SECS=20 LT_SAMPLE_SECS=5 \
  cargo run --release --manifest-path tools/ws-loadtest/Cargo.toml
```

The recorded baseline numbers live in the task doc
(`.metis/initiatives/BROKKR-I-0020/tasks/BROKKR-T-0177.md`).
