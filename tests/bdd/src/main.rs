/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Brokkr BDD suite (BROKKR-T-0385).
//!
//! The Gherkin features in `features/` describe the onboarding flows as a
//! person runs them: generator onboarding, agent onboarding, the generator after
//! apply, and the rules that keep generators apart. The steps drive the real
//! `brokkr` CLI (`BROKKR_CLI`), the real `brokkr-agent` binary
//! (`BROKKR_AGENT`) and, where no command exists, the broker API, against a
//! running broker (`BROKER_URL`, `ADMIN_PAK`).
//!
//! Each scenario creates its own generators, agents and stacks under unique
//! names and removes them in the `after` hook, so scenarios run in any order
//! and the suite runs twice against the same broker.
//!
//! Run with: `angreal tests bdd [--skip-docker] [--feature NAME]`.

mod api;
mod process;
mod steps;
mod world;

use std::time::Duration;

use cucumber::World as _;
use futures::FutureExt as _;

use world::{Env, World};

/// The features next to this crate. `--input <glob>` overrides it.
const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/features");

/// A scenario with this tag states a behaviour the product does not have
/// yet; a comment above it names the bug. The runner leaves these out, so
/// the suite stays a gate, unless `BDD_KNOWN_BUGS=run` asks for them.
const KNOWN_BUG_TAG: &str = "known-bug";

/// Whether a scenario with these tags runs in `pairing` (BROKKR-T-0414).
///
/// `pairing-old-agent` and `pairing-old-broker` mark a scenario of that one
/// pairing. `needs-new-agent` and `needs-new-broker` mark a scenario that the
/// last release of that side cannot run.
fn runs_in_pairing<'a>(pairing: &str, tags: impl Iterator<Item = &'a String>) -> bool {
    for tag in tags {
        if let Some(only) = tag.strip_prefix("pairing-")
            && only != pairing
        {
            return false;
        }
        if tag == "needs-new-agent" && pairing == "old-agent" {
            return false;
        }
        if tag == "needs-new-broker" && pairing == "old-broker" {
            return false;
        }
    }
    true
}

#[tokio::main]
async fn main() {
    let env = Env::from_process_env();
    println!("Brokkr BDD suite");
    println!("  broker: {}", env.broker_url);
    println!("  brokkr: {}", env.cli.display());
    match &env.agent_bin {
        Some(path) => println!("  brokkr-agent: {}", path.display()),
        None => println!("  brokkr-agent: not set (BROKKR_AGENT); agent start scenarios fail"),
    }
    match &env.cluster_agent {
        Some(name) => println!("  cluster agent: {name}"),
        None => println!("  cluster agent: not set (BROKKR_CLUSTER_AGENT); cluster scenarios fail"),
    }

    println!("  pairing: {} (BDD_PAIRING)", env.pairing);
    if !["source", "old-agent", "old-broker"].contains(&env.pairing.as_str()) {
        eprintln!(
            "no pairing {:?}. The pairings are: source, old-agent, old-broker",
            env.pairing
        );
        std::process::exit(2);
    }
    let pairing = env.pairing.clone();

    if let Err(e) = api::wait_for_ready(&env.broker_url, Duration::from_secs(60)).await {
        eprintln!("the broker is not ready: {e}");
        std::process::exit(1);
    }

    let run_known_bugs = std::env::var("BDD_KNOWN_BUGS").is_ok_and(|v| v == "run");
    if !run_known_bugs {
        println!("  @{KNOWN_BUG_TAG} scenarios: left out (BDD_KNOWN_BUGS=run includes them)");
    }

    World::cucumber()
        .max_concurrent_scenarios(Some(4))
        .fail_on_skipped()
        .after(|_feature, _rule, _scenario, _finished, world| {
            async move {
                if let Some(world) = world {
                    world.cleanup().await;
                }
            }
            .boxed_local()
        })
        .filter_run_and_exit(FEATURES, move |feature, rule, scenario| {
            let tags = || {
                feature
                    .tags
                    .iter()
                    .chain(rule.iter().flat_map(|r| r.tags.iter()))
                    .chain(scenario.tags.iter())
            };
            let known_bug = tags().any(|t| t == KNOWN_BUG_TAG);
            (run_known_bugs || !known_bug) && runs_in_pairing(&pairing, tags())
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::runs_in_pairing;

    fn runs(pairing: &str, tags: &[&str]) -> bool {
        let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        runs_in_pairing(pairing, tags.iter())
    }

    #[test]
    fn a_scenario_with_no_pairing_tag_runs_in_each_pairing() {
        for pairing in ["source", "old-agent", "old-broker"] {
            assert!(runs(pairing, &[]));
            assert!(runs(pairing, &["known-bug"]));
        }
    }

    #[test]
    fn a_pairing_scenario_runs_in_its_pairing_only() {
        assert!(runs("old-agent", &["pairing-old-agent"]));
        assert!(!runs("old-broker", &["pairing-old-agent"]));
        assert!(!runs("source", &["pairing-old-agent"]));
        assert!(runs("old-broker", &["pairing-old-broker"]));
        assert!(!runs("source", &["pairing-old-broker"]));
    }

    #[test]
    fn a_scenario_that_needs_a_new_side_is_left_out_when_that_side_is_old() {
        assert!(runs("source", &["needs-new-agent", "needs-new-broker"]));
        assert!(!runs("old-agent", &["needs-new-agent"]));
        assert!(runs("old-broker", &["needs-new-agent"]));
        assert!(!runs("old-broker", &["needs-new-broker"]));
        assert!(!runs("old-agent", &["needs-new-agent", "needs-new-broker"]));
    }
}
