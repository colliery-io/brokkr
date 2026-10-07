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
        assert!(stdout.contains(needle), "apply help missing {needle}: {stdout}");
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
