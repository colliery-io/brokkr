---
title: Running and Writing the BDD Features
---

The BDD suite describes Brokkr's flows as a person runs them, in Gherkin, and runs each line against a real broker. The features live in `tests/bdd/features`; the steps that give them meaning are Rust, in `tests/bdd/src/steps`, built on the [`cucumber`](https://crates.io/crates/cucumber) crate. The steps run the real `brokkr` CLI and the real `brokkr-agent` binary, and call the broker API only where no command exists.

There is one feature per flow:

| Feature | What it covers |
|---------|----------------|
| `generator-onboarding` | An admin creates a generator, gets its PAK once, registers an agent with it; a repeat name is refused; the generator's PAK reads its own record and nothing else. |
| `agent-onboarding` | An admin creates an agent with its generators and gets its PAK and the Helm command once; the agent is inactive until activated; the agent finds its record by its PAK, and a wrong configured name or cluster stops it with a precise error. |
| `generator-after-apply` | With its own PAK, a generator applies a folder, sees its stack's status, events and logs, sees the labels of its agents and which agents a stack label reaches, rotates its PAK, deletes its stack, and sees where its stack went. |
| `generators-apart` | A generator cannot create generators or agents, cannot register agents, cannot read another generator's stack; a target for an unregistered agent is refused, even for the admin. For work orders: a generator cannot read the status, the logs, the events or the result of another generator's work order, cannot delete it, and does not see it in its list or its history; a work order for an agent that is not registered with the generator is refused, and the error names the command that registers it. |
| `agent-lifecycle` | After onboarding: an agent's heartbeat in the fleet record, a silent agent, a paused agent, an agent that loses a stack and no longer counts its health, a deleted agent. The scenario plays the agent with the agent's PAK. |
| `stack-lifecycle` | After the first apply: a changed folder gives a new revision that the agent serves, a failing revision makes the stack read failing, a new revision that fixes it makes it read healthy, a deleted stack leaves no failing and no degraded record. The scenario plays the agent with the agent's PAK. |
| `work-order-lifecycle` | A work order from create to its end: pending, claimed, completed and in the log; a failure that waits and comes back until its retries are spent; a claim that passes its timeout; a work order for a label that one of two agents claims; the pending and claimed counts of the fleet record. The scenario plays the agent with the agent's PAK. With its own PAK and the `brokkr work-order` commands, a generator creates a work order and reads its status, its result and its history; its work order for a label reaches only the agents registered with it; it lists its own work orders and not those of an admin; it deletes a pending one. With the agent in the cluster of the test stack: an admin's job and a generator's job run, and their log line, events and result read back. |

Each scenario creates its own generators, agents and stacks under unique names and removes them when it ends, so scenarios run in any order, side by side, and the suite runs twice against the same broker.

## Run the suite

The suite needs the development compose stack: a broker, a database and the stack's agent in its k3s cluster. `angreal tests bdd` brings the stack up, builds the suite, the CLI and the agent, runs every feature and takes the stack down:

```bash
angreal tests bdd
```

To run one feature, name its file without the extension. With `--skip-docker` the task uses a stack that is up already (for example after `angreal local up`):

```bash
angreal tests bdd --skip-docker --feature generator-onboarding
```

The suites share one compose project, so run one at a time.

The binary reads its targets from the environment: `BROKER_URL` (default `http://localhost:3000`), `ADMIN_PAK` (default: the development PAK), `BROKKR_CLI` (the `brokkr` binary, required), `BROKKR_AGENT` (the `brokkr-agent` binary, for the scenarios that start an agent) and `BROKKR_CLUSTER_AGENT` (the name of the stack's agent, for the scenario that waits for a report). `angreal tests bdd` sets all of them. To run the binary yourself, from the repository root:

```bash
cargo build -p brokkr-cli --bin brokkr -p brokkr-agent --bin brokkr-agent
cargo build --release --manifest-path tests/bdd/Cargo.toml
BROKKR_CLI=target/debug/brokkr BROKKR_AGENT=target/debug/brokkr-agent \
BROKKR_CLUSTER_AGENT=brokkr-integration-test-agent \
  tests/bdd/target/release/brokkr-bdd --input tests/bdd/features/generators-apart.feature
```

`--input` takes a glob; without it the binary runs every feature in `tests/bdd/features`. `--name <regex>` runs the scenarios whose name matches; `--concurrency <n>` sets how many run at once (the default is 4); `-vv` prints the scenario's state after a failed step. Run with `--help` for the rest.

A failed step prints the step, the command that ran, and the command's whole output (stdout and stderr), so the report shows what the CLI said.

A scenario tagged `@known-bug` states a behaviour the product does not have yet; a comment above it names the bug. The runner leaves these scenarios out, so the suite stays a gate. `BDD_KNOWN_BUGS=run` includes them; run them when the fix lands, then remove the tag and the comment.

## In CI

The workflow `bdd.yml` runs one feature per runner, each with its own compose stack. `main.yml` calls it on a pull request that changes the broker, the agent, the CLI, the client, the features or the workflow, and the `CI gate` check requires it.

## Add a scenario

A scenario is one behaviour. Write it in the words of the console: generator, agent, stack, PAK. Say what a person does and sees, never how the broker does it: "the generator applies the folder", not "POST /stacks". Keep each line short and in the present tense.

```gherkin
  Scenario: A repeat name is refused
    Given the admin has created the generator "team-robin"
    When the admin creates the generator "team-robin" again
    Then the command is refused
    And there is one generator "team-robin"
```

Names in a feature are plain (`team-robin`, `prod-1`, `payments`). The runner gives each a unique real name for the scenario, so two scenarios that both say `team-robin` never collide. Use the steps that exist where you can: read the `#[given]`, `#[when]` and `#[then]` attributes in `tests/bdd/src/steps`, or run a feature with a line that has no step yet; the suite fails on it and names the line.

## Add a step

A step is an `async fn` on the `World` with a `#[given]`, `#[when]` or `#[then]` attribute and a [Cucumber expression](https://github.com/cucumber/cucumber-expressions#readme); `{string}` takes a quoted value, `{word}` one word. The modules are by who acts: `steps/admin.rs` and `steps/generator.rs` and `steps/agent.rs` hold what each does (Given and When), `steps/checks.rs` holds what they see (Then).

```rust
#[when(expr = "the generator {string} deletes the stack {string}")]
async fn deletes_stack(w: &mut World, generator: String, stack: String) {
    let pak = w.generator(&generator).pak.clone();
    let stack = w.real(&w.stack(&stack).name);
    w.brokkr(&pak, &["stack", "delete", &stack]).await;
}
```

The `World` (`tests/bdd/src/world.rs`) holds the scenario's generators, agents and stacks by their feature names, the last command that ran and the last answer of the broker. Its helpers do the common work: `w.real(name)` gives the unique real name; `w.brokkr(pak, args)` runs the CLI and keeps the run; `w.brokkr_agent(pak, name, cluster)` starts the agent binary; `w.expect_output(text)` and `w.expect_error(text)` check the last run and, when the check fails, show the whole command and its output; `w.api` reaches the broker for a read no command makes. Keep a Then step to one check, so a failure names the one thing that went wrong.

A step that creates something records it on the `World` (`w.generators`, `w.agents`, `w.stacks`), and the `after` hook in `main.rs` deletes everything recorded there as the admin. A step that creates a new kind of thing adds it to that hook too.

Before you commit, run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` in `tests/bdd`, and run the feature you changed against the stack.

## Add a feature

A new feature is a new file in `tests/bdd/features`, named for its flow, with one line of context under the `Feature:` heading. Add its name to the `feature` matrix in `.github/workflows/bdd.yml` and to the table above.
