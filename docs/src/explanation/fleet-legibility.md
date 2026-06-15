# Fleet Legibility

> **Scope:** internal architecture — for operators, and for consumers or
> integrators building against the fleet API. This page explains
> *why* the broker exposes a fleet view the way it does, and the design
> stance behind it. For the machinery (endpoints, fields, frame shapes)
> see the [Fleet reference](../reference/fleet.md); for the operational
> task see [Monitoring Your Fleet](../how-to/fleet-monitoring.md).

Brokkr is [pull-based](./reconciliation.md): agents poll the broker for
their target state and own the reconciliation of their own clusters. The
broker is not a controller standing over the fleet pushing work down — it
is a place agents come to. That architecture is deliberate and it scales,
but it creates an awkward question for whoever is running the system:
**if the broker doesn't drive the agents, how do you know the fleet is
healthy?** Nothing in the control loop forces an answer. An agent can
stop polling, lose its cluster connection, or fall hopelessly behind on a
backlog of deployment objects, and the pull model alone gives the
operator no obvious signal that any of it happened.

Fleet legibility is the broker's answer. The work landed as two
initiatives: **Agent Fleet Legibility**, which makes the fleet's state
observable on demand, and **Fleet Live Push**, which turns that observable
state into a change-driven stream. This page is about the philosophy those
two share.

## Legibility, not alerting

The word *legibility* is doing real work here, and it is chosen over
*alerting* on purpose.

An alerting system decides. It holds thresholds, evaluates them, and
emits verdicts: this agent is *unhealthy*, that backlog is *too deep*,
this heartbeat is *stale*. Brokkr deliberately does **not** do this. The
broker computes and reports *measured values* — how many seconds since
the last heartbeat, how many deployment objects are pending for an agent,
how many work orders are claimed, whether the agent last reported its
Kubernetes API as reachable, how long since its last event. It attaches
no judgement to any of them. There is no `healthy` boolean anywhere in a
fleet record, and that absence is the whole design.

The reason is that **thresholds are deployment-specific and the broker
has no business owning them.** Consider heartbeat age. A 30-second-stale
heartbeat is completely unremarkable for an operator running a sleepy
fleet of agents polling on a long interval; for an operator who tuned
their agents to a tight loop and expects sub-second liveness, 30 seconds
of silence is a fire. Both operators are correct *for their deployment*.
If the broker baked in a "stale after N seconds" rule, it would be wrong
for at least one of them, and probably wrong for most of them most of the
time. The same reasoning applies to every signal: what counts as "too
much" pending-object backpressure depends on the workload's normal churn;
what counts as "too quiet" depends on how often that agent is expected to
do anything at all.

So the broker draws the line at *measurement* and hands *severity* to the
consumer. **The broker computes; the consumer decides.** A consumer — an
operator's script, a Datadog monitor fed from the API, or the `ui-slim`
demo that ships in `examples/ui-slim` — is where the thresholds live,
because that is where the deployment-specific knowledge lives. This keeps
the broker honest (it never claims to know something it can't) and keeps
it stable (operators tune their own alerting without asking the broker to
grow another config knob). It is the same restraint the telemetry track
shows in [the internal WS channel](./internal-ws-channel.md): surface the
signal, point at the right tool for the verdict, refuse to grow into a
role that belongs elsewhere.

## Why most signals are computed but one is reported

Almost every fleet signal is something the broker can derive from data it
already holds. It knows when each agent last heartbeated, how many pending
deployment objects are targeted at an agent, how many work orders are
pending or claimed, how many of an agent's deployment-health records are
failing or degraded, and when the agent last produced an event. None of
that requires asking the agent anything — it's a set of grouped queries
over the broker's own database, assembled into a per-agent record. (That
the rollup is computed with bounded, whole-fleet grouped queries rather
than one query per agent is a scaling detail, not a philosophical one;
the reference covers it.)

There is exactly one signal the broker *cannot* compute: **whether an
agent can reach its own Kubernetes API.** The broker lives outside the
agent's cluster. It has no route into that cluster's control plane and no
way to probe it — the agent could be cut off from its own API server
while its connection to the broker stays perfectly healthy. The only
component that can answer "is kube reachable from where the agent sits" is
the agent itself.

So that one signal is **agent-self-reported**, carried on the heartbeat
the agent already sends. The broker takes the agent's word for it and
stores the latest value verbatim; it is the one fleet field the broker
trusts rather than derives. This is a clean illustration of the legibility
stance at the data-source level: the broker reports what it can measure
directly, and for the one thing it structurally *cannot* see, it relays
the report of the only party who can — without second-guessing it.

Because that report may simply not exist yet — a freshly-registered agent
that hasn't completed a heartbeat, an older agent that predates the probe
— the reachability fields are **nullable**, and null means "unknown," not
"unreachable." This is graceful degradation by design: the absence of a
report is itself a distinguishable state, and (consistent with the whole
philosophy) the broker leaves it to the consumer to decide whether an
agent that has never reported its kube reachability is worth worrying
about. A boolean would have collapsed "I don't know" into a false
verdict; a nullable value preserves the honest third state.

## Pull and push: state-on-demand vs change-driven

Legibility comes in two shapes because operators need it in two shapes.

The **pull** surface — `GET /fleet` — answers "what is the state of the
whole fleet *right now*." It is state-on-demand: a consumer asks, the
broker computes the current records and returns them, and the exchange is
over. This is the right model for an operator opening a dashboard, a
script doing a periodic scrape, or anything that wants a coherent snapshot
at a moment of its choosing. It needs nothing standing and costs nothing
between requests.

The **push** surface — `/fleet/live` — answers a different question:
"tell me *when something changes*." It is a WebSocket a consumer holds
open; the broker streams a per-agent update each time that agent's record
moves. This is the right model for a live view that should reflect the
fleet without the consumer polling in a tight loop — exactly what a
consumer like the `ui-slim` demo uses to keep its fleet panel current.

The two are not redundant; they answer genuinely different questions
(*what is true now* vs *what just changed*), and live push is built as an
optimisation over the pull model rather than a replacement for it. A
record that arrives over `/fleet/live` is assembled from the same code
path as the pull surface, so a pushed record is identical to what a `GET`
would have returned — the live stream is the pull view kept warm.

### The hybrid trigger

The interesting design question is *what makes a live update fire*, and
the answer is a hybrid of two mechanisms, because the signals themselves
divide into two kinds.

Some changes are **events**: an agent connects, an agent disconnects, an
agent heartbeats. These have a discrete moment, and the broker pushes a
fresh record for the affected agent at that moment — event-driven,
immediate, no waiting. This is the natural complement to the internal
broker↔agent channel — the persistent broker↔agent WebSocket over which
agents connect and heartbeat: the same connect/disconnect/heartbeat
moments that the WS channel already surfaces become the triggers for a
fleet update.

But other signals have **no event to hang off of.** Pending-object
backpressure rising, work-order counts shifting, deployment-health records
flipping to degraded — these are facts about the database that become true
without any single frame arriving to announce them. Nothing "happens" when
a backlog grows; it just grows. An event-driven trigger would never fire
for these, and the live view would silently drift out of date.

So the broker also runs a **periodic sweep** (a 20-second cadence in the
current version). Each pass it recomputes the fleet and broadcasts an
update *only* for agents whose computed signals actually changed since the
last pass — a backlog that's been flat for an hour generates no traffic.
The first pass only seeds a baseline so a fresh broker doesn't blast the
whole fleet to subscribers on startup. Both halves are necessary: the
event-driven half gives immediacy for the things that have a moment, and
the periodic half gives eventual freshness for the things that don't. One
without the other leaves a real gap.

## Why fleet push tolerates lag by dropping

A single in-process `FleetBroadcaster` fans every per-agent update out to
every `/fleet/live` subscriber. Its capacity is bounded, which raises the
classic streaming question: what happens to a subscriber that falls
behind? Brokkr's answer for the fleet stream is the bluntest one
available — **a lagging fleet subscriber just drops the frames it missed
and keeps going.** No gap marker, no backfill, no slowing the producer.

This is only acceptable because of a specific property of fleet records:
**each record fully supersedes the previous one for that agent.** A
`FleetUpdate` is the agent's complete current state, not a delta against a
prior frame. A consumer holds the latest record per `agent_id`; if it
misses three updates for an agent and then receives a fourth, the fourth
*is* the truth and the three it missed are irrelevant — they were
snapshots of intermediate states that no longer exist. Dropping a
superseded frame loses nothing that matters.

Contrast this deliberately with the **stack log-tail** described in [the
internal WS channel](./internal-ws-channel.md). There, every line matters:
a log line is not superseded by the next line, it stands on its own, and a
silently dropped line is lost information. So that path takes the opposite
stance — when a log subscriber lags, the broker emits a visible `log_gap`
marker so the consumer can *see* that data went missing rather than being
quietly misled into thinking it saw everything. The two streams make
opposite trade-offs because the data has opposite semantics: cumulative,
order-sensitive log lines demand gap-marking; self-superseding state
snapshots can be dropped without ceremony. Both are the same underlying
rule from [ADR-0008](https://github.com/colliery-io/brokkr/blob/main/.metis/adrs/BROKKR-A-0008.md)
— *a slow subscriber must never slow ingestion* — applied honestly to
data with different meaning.

There is a second, related guarantee: a fleet broadcast **never blocks the
operation that triggered it.** The push is fired from producer hot paths —
a heartbeat being recorded, a connection being registered — and a failure
to push (no subscribers, a DB hiccup, an agent that vanished mid-compute)
must never affect that triggering operation. The producer is best-effort
and swallows its own failures by design. Legibility is a layer *over* the
control plane, never a thing the control plane waits on; the fleet view
going dark must never be able to wedge a heartbeat.

## Admin-only scoping

The whole fleet surface — pull and push alike — is **admin-only.** This is
a sharper scoping rule than some of Brokkr's other endpoints, which allow
an owner to see their own resources, and the reason is structural rather
than cautious.

A fleet record is inherently **cross-cutting.** It spans every agent, and
agents span tenants; the rollup is a view of the entire deployment at
once. There is no coherent way to scope a fleet record "to its owner,"
because it has no single owner — it is, by construction, everyone's. A
per-agent record also folds together backpressure, health, and
connectivity that cross tenant boundaries. The only principal for whom the
fleet view is meaningful and appropriate is one with deployment-wide
authority, so the broker gates both the `GET /fleet` rollup and the
`/fleet/live` subscription on the admin check, with no per-owner path. The
live subscription's auth is *simpler* than the stack live-tail's
admin-or-owner rule precisely because there is no owner dimension to
reason about — for fleet, it's admin or nothing.

## Related

- [Fleet reference](../reference/fleet.md) — the endpoints, record fields,
  and frame shapes.
- [Monitoring Your Fleet](../how-to/fleet-monitoring.md) — the operational
  task of building thresholds and watching the fleet.
- [Internal Broker ↔ Agent WebSocket Channel](./internal-ws-channel.md) —
  the channel whose connect/disconnect/heartbeat moments drive the
  event half of the live trigger, and the log-tail whose gap-marking this
  page contrasts against.
- [Reconciliation](./reconciliation.md) — the pull-based model that makes
  fleet legibility necessary in the first place.
- [ADR-0008](https://github.com/colliery-io/brokkr/blob/main/.metis/adrs/BROKKR-A-0008.md)
  — the WS-channel decision record, including the slow-subscriber rule.
