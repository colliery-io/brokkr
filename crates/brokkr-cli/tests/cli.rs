/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Functional tests that exercise the built `brokkr` binary as a black box:
//! argument parsing, the config-resolution UX, and error handling. These do
//! not need a running broker — the live `apply` path (binary → broker) is
//! covered by the Rust SDK contract suite, since the command is a thin shell
//! over the contract-tested `BrokkrClient::apply`.

use std::path::Path;
use std::process::Command;

/// Path to the compiled binary under test (Cargo sets `CARGO_BIN_EXE_<name>`).
fn brokkr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_brokkr"))
}

/// Run with a deliberately empty environment so a developer's real
/// `~/.brokkr/config` or `BROKKR_*` vars can't leak into the assertions.
fn sandboxed(mut cmd: Command) -> Command {
    cmd.env_remove("BROKKR_BROKER_URL")
        .env_remove("BROKKR_PAK")
        // Point HOME at a path with no config so default_config_path misses.
        .env("HOME", "/nonexistent-brokkr-home");
    cmd
}

fn run(mut cmd: Command) -> (std::process::Output, String, String) {
    let output = cmd.output().expect("failed to spawn brokkr");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output, stdout, stderr)
}

#[test]
fn help_lists_apply() {
    let mut cmd = brokkr();
    cmd.arg("--help");
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("apply"), "top-level help: {stdout}");
}

#[test]
fn apply_help_documents_flags() {
    let mut cmd = brokkr();
    cmd.args(["apply", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for needle in ["--filename", "--stack", "--target-label", "--generator"] {
        assert!(
            stdout.contains(needle),
            "apply help missing {needle}: {stdout}"
        );
    }
}

#[test]
fn version_prints() {
    let mut cmd = brokkr();
    cmd.arg("--version");
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("brokkr"), "version: {stdout}");
}

#[test]
fn apply_requires_stack_and_filename() {
    // Missing required args -> clap usage error on stderr, non-zero exit.
    let mut cmd = brokkr();
    cmd.arg("apply");
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("--filename") || stderr.contains("--stack"),
        "expected a usage error, got: {stderr}"
    );
}

#[test]
fn apply_without_connection_config_errors_clearly() {
    // Args are valid but there is no broker URL / PAK anywhere -> our resolver
    // should fail before any network call with an actionable message.
    let cmd = sandboxed({
        let mut c = brokkr();
        c.args(["apply", "-f", ".", "--stack", "demo"]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("broker URL") || stderr.contains("PAK"),
        "expected a connection-config error, got: {stderr}"
    );
}

#[test]
fn malformed_config_file_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    std::fs::write(&config, "broker_url = \"missing closing quote\npak =").unwrap();

    let cmd = sandboxed({
        let mut c = brokkr();
        c.args([
            "apply",
            "-f",
            ".",
            "--stack",
            "demo",
            "--config",
            config.to_str().unwrap(),
        ]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("parse config file"),
        "expected a parse error, got: {stderr}"
    );
}

#[test]
fn config_file_supplies_connection_then_bundle_read_runs() {
    // A valid config file satisfies the resolver, so the command proceeds far
    // enough to read the (empty) manifest bundle and fail there instead — i.e.
    // the connection layer was accepted. The broker URL points nowhere, so we
    // assert only that the failure is NOT the connection-config error.
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    std::fs::write(
        &config,
        "broker_url = \"http://127.0.0.1:1\"\npak = \"brokkr_test\"\n",
    )
    .unwrap();
    // Empty manifest dir -> read_manifests should reject before any network use.
    let empty = dir.path().join("manifests");
    std::fs::create_dir(&empty).unwrap();

    let cmd = sandboxed({
        let mut c = brokkr();
        c.args([
            "apply",
            "-f",
            empty.to_str().unwrap(),
            "--stack",
            "demo",
            "--config",
            config.to_str().unwrap(),
        ]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        !stderr.contains("no broker URL") && !stderr.contains("no PAK"),
        "connection config should have been accepted, got: {stderr}"
    );
    // Sanity: the temp config path is what we wrote.
    assert!(Path::new(&config).exists());
}

// --- BROKKR-T-0333: day-zero commands. The live paths run in the Rust SDK
// contract suite (`scenario_day_zero_cli`), which runs this binary against a
// broker. ---

#[test]
fn agent_and_stack_help_list_the_day_zero_commands() {
    for (group, commands) in [
        ("agent", &["activate", "pause", "label", "list"][..]),
        ("stack", &["label", "target", "list"][..]),
    ] {
        let mut cmd = brokkr();
        cmd.args([group, "--help"]);
        let (output, stdout, _) = run(cmd);
        assert!(output.status.success());
        for c in commands {
            assert!(stdout.contains(c), "{group} help is missing {c}: {stdout}");
        }
    }
}

#[test]
fn day_zero_commands_take_a_name_or_an_id() {
    for args in [
        &["agent", "activate", "--help"][..],
        &["agent", "pause", "--help"][..],
        &["agent", "label", "--help"][..],
        &["stack", "label", "--help"][..],
        &["stack", "target", "--help"][..],
    ] {
        let mut cmd = brokkr();
        cmd.args(args);
        let (output, stdout, _) = run(cmd);
        assert!(output.status.success());
        assert!(stdout.contains("name or the id"), "{args:?}: {stdout}");
    }
}

#[test]
fn label_must_have_the_key_value_shape() {
    // The label check runs before any connection, so no broker is needed.
    for (group, target) in [("agent", "my-agent"), ("stack", "my-stack")] {
        let cmd = sandboxed({
            let mut c = brokkr();
            c.args([group, "label", target, "env=prod"]);
            c
        });
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success());
        assert!(stderr.contains("key:value"), "{group}: {stderr}");
    }
}

#[test]
fn stack_target_needs_a_stack_and_an_agent() {
    let mut cmd = brokkr();
    cmd.args(["stack", "target", "my-stack"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("<AGENT>"),
        "expected a usage error, got: {stderr}"
    );
}

#[test]
fn agent_list_without_connection_config_errors_clearly() {
    let cmd = sandboxed({
        let mut c = brokkr();
        c.args(["agent", "list"]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("broker URL") || stderr.contains("PAK"),
        "expected a connection-config error, got: {stderr}"
    );
}

// --- BROKKR-T-0344: register, deregister and registrations take a name or
// an id. The live path is in the Rust SDK contract suite
// (`scenario_registration_by_name_cli`). ---

#[test]
fn registration_commands_take_a_name_or_an_id() {
    for command in ["register", "deregister", "registrations"] {
        let mut cmd = brokkr();
        cmd.args([command, "--help"]);
        let (output, stdout, _) = run(cmd);
        assert!(output.status.success());
        assert!(stdout.contains("NAME_OR_ID"), "{command}: {stdout}");
        assert!(stdout.contains("name or the id"), "{command}: {stdout}");
    }
}

// BROKKR-T-0358: a repeat register succeeds, like `agent label`. The live
// path is in `scenario_registration_by_name_cli`.
#[test]
fn register_help_says_a_repeat_is_unchanged() {
    let mut cmd = brokkr();
    cmd.args(["register", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("unchanged:"), "{stdout}");
    assert!(stdout.contains("status 0"), "{stdout}");
}

#[test]
fn register_with_names_parses_then_needs_a_connection() {
    // A name is not a parse error: the command gets as far as the
    // connection config before it fails.
    let cmd = sandboxed({
        let mut c = brokkr();
        c.args(["register", "--agent", "edge-1", "--generator", "acme"]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("broker URL") || stderr.contains("PAK"),
        "expected a connection-config error, got: {stderr}"
    );
}

// BROKKR-T-0370: `brokkr generator create|list`, so the admin's one-time steps
// (create the generator, activate the agent, register it) are one tool. The
// live path is in `scenario_generator_cli` of the SDK contract suite.
#[test]
fn generator_help_lists_create_and_list() {
    let mut cmd = brokkr();
    cmd.args(["generator", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for c in ["create", "list"] {
        assert!(
            stdout.contains(c),
            "generator help is missing {c}: {stdout}"
        );
    }
    let mut cmd = brokkr();
    cmd.arg("--help");
    let (_, stdout, _) = run(cmd);
    assert!(stdout.contains("generator"), "top-level help: {stdout}");
}

#[test]
fn generator_create_help_says_the_pak_is_shown_once() {
    let mut cmd = brokkr();
    cmd.args(["generator", "create", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("--description"), "{stdout}");
    assert!(stdout.contains("shown once"), "{stdout}");
    assert!(stdout.contains("generator"), "{stdout}");
}

#[test]
fn generator_create_needs_a_name() {
    let mut cmd = brokkr();
    cmd.args(["generator", "create"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("NAME"),
        "expected a usage error, got: {stderr}"
    );
}

#[test]
fn generator_commands_parse_then_need_a_connection() {
    for args in [
        &["generator", "create", "team-acme", "--description", "Acme"][..],
        &["generator", "list"][..],
    ] {
        let cmd = sandboxed({
            let mut c = brokkr();
            c.args(args);
            c
        });
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success());
        assert!(
            stderr.contains("broker URL") || stderr.contains("PAK"),
            "{args:?}: expected a connection-config error, got: {stderr}"
        );
    }
}

// BROKKR-T-0373: `brokkr agent create`, the last of the admin's one-time
// steps that needed curl. The live path is in `scenario_agent_create_cli`.
#[test]
fn agent_create_help_says_the_pak_is_shown_once_and_names_activate() {
    let mut cmd = brokkr();
    cmd.args(["agent", "create", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for needle in [
        "--cluster",
        "--generator",
        "shown once",
        "activate",
        "NAME_OR_ID",
    ] {
        assert!(
            stdout.contains(needle),
            "agent create help missing {needle}: {stdout}"
        );
    }
}

#[test]
fn agent_create_needs_a_name_and_a_cluster() {
    let mut cmd = brokkr();
    cmd.args(["agent", "create", "prod-1"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("--cluster"),
        "expected a usage error, got: {stderr}"
    );
}

#[test]
fn agent_create_parses_then_needs_a_connection() {
    let cmd = sandboxed({
        let mut c = brokkr();
        c.args([
            "agent",
            "create",
            "prod-1",
            "--cluster",
            "us-east-1",
            "--generator",
            "acme",
        ]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("broker URL") || stderr.contains("PAK"),
        "expected a connection-config error, got: {stderr}"
    );
}

// BROKKR-T-0379, 0380, 0381: what a generator does after `apply`: see where the
// stack went, read its events and logs, delete it, rotate its PAK, and say
// what its PAK is. The live path is in `scenario_generator_flow_cli`.
#[test]
fn stack_help_lists_status_events_logs_and_delete() {
    let mut cmd = brokkr();
    cmd.args(["stack", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for c in ["status", "events", "logs", "delete"] {
        assert!(stdout.contains(c), "stack help is missing {c}: {stdout}");
    }
    for args in [
        &["stack", "status", "--help"][..],
        &["stack", "events", "--help"][..],
        &["stack", "logs", "--help"][..],
        &["stack", "delete", "--help"][..],
    ] {
        let mut cmd = brokkr();
        cmd.args(args);
        let (output, stdout, _) = run(cmd);
        assert!(output.status.success());
        assert!(stdout.contains("name or the id"), "{args:?}: {stdout}");
    }
    let mut cmd = brokkr();
    cmd.args(["stack", "events", "--help"]);
    let (_, stdout, _) = run(cmd);
    assert!(stdout.contains("--limit"), "{stdout}");
}

#[test]
fn generator_rotate_and_whoami_have_help() {
    let mut cmd = brokkr();
    cmd.args(["generator", "rotate", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("once"), "{stdout}");
    assert!(stdout.contains("NAME_OR_ID"), "{stdout}");
    let mut cmd = brokkr();
    cmd.args(["whoami", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("generator"), "{stdout}");
    let mut cmd = brokkr();
    cmd.arg("--help");
    let (_, stdout, _) = run(cmd);
    assert!(stdout.contains("whoami"), "top-level help: {stdout}");
}

#[test]
fn generator_flow_commands_parse_then_need_a_connection() {
    for args in [
        &["stack", "status", "payments"][..],
        &["stack", "events", "payments", "--limit", "20"][..],
        &["stack", "logs", "payments"][..],
        &["stack", "delete", "payments"][..],
        &["generator", "rotate"][..],
        &["generator", "rotate", "team-acme"][..],
        &["whoami"][..],
    ] {
        let cmd = sandboxed({
            let mut c = brokkr();
            c.args(args);
            c
        });
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success());
        assert!(
            stderr.contains("broker URL") || stderr.contains("PAK"),
            "{args:?}: expected a connection-config error, got: {stderr}"
        );
    }
}

// BROKKR-T-0371: the word is generator. The old names stay as hidden aliases
// for one release so pipelines do not break.
#[test]
fn the_old_tenant_names_still_parse() {
    let mut cmd = brokkr();
    cmd.args(["tenant", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("create"), "{stdout}");
    let mut cmd = brokkr();
    cmd.args(["agent", "create", "--help"]);
    let (_, stdout, _) = run(cmd);
    assert!(stdout.contains("--generator"), "{stdout}");
    assert!(
        !stdout.contains("--tenant"),
        "the alias is hidden: {stdout}"
    );
    let cmd = sandboxed({
        let mut c = brokkr();
        c.args([
            "agent",
            "create",
            "prod-1",
            "--cluster",
            "us-east-1",
            "--tenant",
            "acme",
        ]);
        c
    });
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("broker URL") || stderr.contains("PAK"),
        "the old flag parses then needs a connection: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Work order commands (BROKKR-T-0407). The calls to the broker are tested in
// the broker's integration suite (`api::sdk_work_orders`); here: the help, the
// arguments, and the need for a connection.
// ---------------------------------------------------------------------------

const WORK_ORDER_ID: &str = "00000000-0000-0000-0000-000000000001";

#[test]
fn help_lists_the_work_order_commands() {
    let mut cmd = brokkr();
    cmd.arg("--help");
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    assert!(stdout.contains("work-order"), "top-level help: {stdout}");

    let mut cmd = brokkr();
    cmd.args(["work-order", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for needle in [
        "create", "list", "history", "status", "delete", "logs", "events",
    ] {
        assert!(
            stdout.contains(needle),
            "work-order help missing {needle}: {stdout}"
        );
    }
}

#[test]
fn work_order_create_help_documents_the_targets_and_the_timing() {
    let mut cmd = brokkr();
    cmd.args(["work-order", "create", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for needle in [
        "--filename",
        "--type",
        "--agent",
        "--label",
        "--annotation",
        "--claim-timeout",
        "--max-retries",
        "--backoff",
        "owns it",
    ] {
        assert!(
            stdout.contains(needle),
            "work-order create help missing {needle}: {stdout}"
        );
    }
}

#[test]
fn work_order_create_needs_a_file_and_one_target() {
    // No file.
    let mut cmd = brokkr();
    cmd.args(["work-order", "create", "--agent", "prod-1"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(stderr.contains("--filename"), "{stderr}");

    // A file and no target.
    let mut cmd = brokkr();
    cmd.args(["work-order", "create", "-f", "job.yaml"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    for needle in ["--agent", "--label", "--annotation"] {
        assert!(stderr.contains(needle), "missing {needle}: {stderr}");
    }
}

#[test]
fn work_order_targets_have_their_shapes() {
    for (flag, bad, shape) in [
        ("--label", "env=prod", "key:value"),
        ("--annotation", "tier:gold", "key=value"),
    ] {
        let mut cmd = brokkr();
        cmd.args(["work-order", "create", "-f", "job.yaml", flag, bad]);
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success());
        assert!(stderr.contains(shape), "{flag} {bad}: {stderr}");
    }
}

#[test]
fn work_order_commands_take_an_id() {
    for command in ["status", "delete", "logs", "events"] {
        // No id.
        let mut cmd = brokkr();
        cmd.args(["work-order", command]);
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success(), "{command}");
        assert!(stderr.contains("<ID>"), "{command}: {stderr}");

        // A name is not an id: a work order has no name.
        let mut cmd = brokkr();
        cmd.args(["work-order", command, "nightly-backup"]);
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success(), "{command}");
        assert!(stderr.contains("nightly-backup"), "{command}: {stderr}");
    }
}

#[test]
fn work_order_commands_parse_then_need_a_connection() {
    let job = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(job.path(), "apiVersion: batch/v1\nkind: Job\n").unwrap();
    let job_path = job.path().to_str().unwrap().to_string();

    let cases: Vec<Vec<&str>> = vec![
        vec![
            "work-order",
            "create",
            "-f",
            &job_path,
            "--agent",
            "prod-1",
            "--label",
            "env:prod",
            "--annotation",
            "tier=gold",
            "--claim-timeout",
            "600",
            "--max-retries",
            "1",
            "--backoff",
            "30",
        ],
        vec![
            "work-order",
            "create",
            "-f",
            &job_path,
            "--type",
            "build",
            "--label",
            "env:prod",
        ],
        vec!["work-order", "list"],
        vec!["work-order", "history"],
        vec![
            "work-order",
            "history",
            "--type",
            "custom",
            "--result",
            "failure",
            "--agent",
            WORK_ORDER_ID,
            "--limit",
            "20",
        ],
        vec!["work-order", "status", WORK_ORDER_ID],
        vec!["work-order", "delete", WORK_ORDER_ID],
        vec!["work-order", "logs", WORK_ORDER_ID, "--limit", "50"],
        vec!["work-order", "events", WORK_ORDER_ID],
    ];
    for args in cases {
        let mut cmd = sandboxed(brokkr());
        cmd.args(&args);
        let (output, _, stderr) = run(cmd);
        assert!(!output.status.success(), "{args:?}");
        // The arguments parsed: the failure is the missing connection.
        assert!(
            stderr.contains("broker URL") || stderr.contains("BROKKR_BROKER_URL"),
            "{args:?}: {stderr}"
        );
    }
}

#[test]
fn work_order_history_help_documents_the_filters() {
    let mut cmd = brokkr();
    cmd.args(["work-order", "history", "--help"]);
    let (output, stdout, _) = run(cmd);
    assert!(output.status.success());
    for needle in [
        "--type", "--result", "--agent", "--limit", "success", "failure", "ended",
    ] {
        assert!(
            stdout.contains(needle),
            "work-order history help missing {needle}: {stdout}"
        );
    }
}

#[test]
fn work_order_history_result_is_success_or_failure() {
    let mut cmd = brokkr();
    cmd.args(["work-order", "history", "--result", "ok"]);
    let (output, _, stderr) = run(cmd);
    assert!(!output.status.success());
    assert!(
        stderr.contains("success") && stderr.contains("failure"),
        "{stderr}"
    );
}
