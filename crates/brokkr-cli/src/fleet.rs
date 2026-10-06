/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Day-zero commands for agents and stacks (BROKKR-T-0333).
//!
//! `brokkr agent activate|pause|label|list` and `brokkr stack label|target|list`.
//! Each command takes an agent or a stack by name or by id; the SDK looks the
//! id up, so a tutorial needs no `jq` step. Each command is a thin shell over
//! one [`BrokkrClient`] method.

use brokkr_client::{AGENT_ACTIVE, AGENT_INACTIVE, BrokkrClient};
use clap::{Args, Subcommand};

/// The longest label the broker accepts.
const LABEL_MAX: usize = 64;

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    /// Let an agent apply its stacks.
    ///
    /// A new agent is INACTIVE and applies nothing. This command sets its
    /// status to ACTIVE. The agent starts to apply its stacks on its next
    /// poll. Requires an admin PAK.
    Activate(AgentRef),

    /// Stop an agent from applying its stacks.
    ///
    /// This command sets the status of the agent to INACTIVE. The agent keeps
    /// the resources that it applied, but it applies no new changes until you
    /// activate it again. Requires an admin PAK.
    Pause(AgentRef),

    /// Add a label to an agent.
    ///
    /// A stack that has the same label goes to the agent. The label has the
    /// form key:value, for example env:prod. If the agent has the label
    /// already, the command changes nothing. Requires an admin PAK.
    Label(AgentLabelArgs),

    /// List the agents with their name, id, status, cluster and labels.
    ///
    /// An admin PAK lists all agents. A tenant PAK lists the agents that are
    /// registered with its tenant.
    List,
}

#[derive(Debug, Subcommand)]
pub enum StackCommand {
    /// Add a label to a stack.
    ///
    /// The stack goes to each agent that has the same label. The label has
    /// the form key:value, for example env:prod. If the stack has the label
    /// already, the command changes nothing. Requires an admin PAK or the PAK
    /// of the tenant that owns the stack.
    Label(StackLabelArgs),

    /// Send a stack to one agent, whatever the labels of the agent are.
    ///
    /// The agent must be registered with the tenant that owns the stack. If
    /// the target exists already, the command changes nothing. Requires an
    /// admin PAK or the PAK of the tenant that owns the stack.
    Target(StackTargetArgs),

    /// List the stacks with their name, id and labels.
    ///
    /// An admin PAK lists all stacks. A tenant PAK lists the stacks of its
    /// tenant.
    List,
}

#[derive(Debug, Args)]
pub struct AgentRef {
    /// The name or the id of the agent.
    #[arg(value_name = "AGENT")]
    agent: String,
}

#[derive(Debug, Args)]
pub struct AgentLabelArgs {
    /// The name or the id of the agent.
    #[arg(value_name = "AGENT")]
    agent: String,

    /// The label to add, in the form key:value (for example env:prod).
    #[arg(value_name = "LABEL", value_parser = parse_label)]
    label: String,
}

#[derive(Debug, Args)]
pub struct StackLabelArgs {
    /// The name or the id of the stack.
    #[arg(value_name = "STACK")]
    stack: String,

    /// The label to add, in the form key:value (for example env:prod).
    #[arg(value_name = "LABEL", value_parser = parse_label)]
    label: String,
}

#[derive(Debug, Args)]
pub struct StackTargetArgs {
    /// The name or the id of the stack.
    #[arg(value_name = "STACK")]
    stack: String,

    /// The name or the id of the agent that gets the stack.
    #[arg(value_name = "AGENT")]
    agent: String,
}

/// Accept a label only in the one label shape, `key:value`. The broker
/// compares labels as exact strings, so `env=prod` would never match
/// `env:prod`; refusing it here stops that mistake before it is stored.
pub fn parse_label(raw: &str) -> Result<String, String> {
    let shape =
        || format!("the label \"{raw}\" must have the form key:value, for example env:prod");
    if raw.chars().any(char::is_whitespace) {
        return Err(format!("{}. A label cannot contain spaces", shape()));
    }
    if raw.len() > LABEL_MAX {
        return Err(format!(
            "the label \"{raw}\" is too long. The maximum is {LABEL_MAX} characters"
        ));
    }
    match raw.split_once(':') {
        Some((key, value)) if !key.is_empty() && !value.is_empty() => Ok(raw.to_string()),
        _ => Err(shape()),
    }
}

pub async fn agent(client: &BrokkrClient, command: AgentCommand) -> Result<(), String> {
    match command {
        AgentCommand::Activate(args) => {
            let agent = client
                .set_agent_status(&args.agent, AGENT_ACTIVE)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "agent \"{}\" ({}) is {}. It applies its stacks on its next poll.",
                agent.name, agent.id, agent.status
            );
        }
        AgentCommand::Pause(args) => {
            let agent = client
                .set_agent_status(&args.agent, AGENT_INACTIVE)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "agent \"{}\" ({}) is {}. It applies nothing until you run: brokkr agent activate {}",
                agent.name, agent.id, agent.status, agent.name
            );
        }
        AgentCommand::Label(args) => {
            let (agent, added) = client
                .add_agent_label(&args.agent, &args.label)
                .await
                .map_err(|e| e.to_string())?;
            if added {
                println!("added label \"{}\" to agent \"{}\"", args.label, agent.name);
            } else {
                println!(
                    "unchanged: agent \"{}\" has label \"{}\" already",
                    agent.name, args.label
                );
            }
        }
        AgentCommand::List => {
            let agents = client.list_agents().await.map_err(|e| e.to_string())?;
            if agents.is_empty() {
                println!("no agents");
                return Ok(());
            }
            // Only an admin PAK can read the labels of an agent. For a
            // generator PAK the broker answers 403, and the list has no
            // LABELS column.
            let mut show_labels = true;
            let mut rows = Vec::with_capacity(agents.len());
            for a in agents {
                let mut row = vec![a.name, a.id.to_string(), a.status, a.cluster_name];
                if show_labels {
                    match client.agent_labels(a.id).await {
                        Ok(labels) => row.push(join_labels(labels)),
                        Err(e) if e.status().map(|s| s.as_u16()) == Some(403) => {
                            show_labels = false
                        }
                        Err(e) => return Err(e.to_string()),
                    }
                }
                rows.push(row);
            }
            let mut header = vec!["NAME", "ID", "STATUS", "CLUSTER"];
            if show_labels {
                header.push("LABELS");
            } else {
                rows.iter_mut().for_each(|r| r.truncate(header.len()));
            }
            print_table(&header, rows);
        }
    }
    Ok(())
}

pub async fn stack(client: &BrokkrClient, command: StackCommand) -> Result<(), String> {
    match command {
        StackCommand::Label(args) => {
            let (stack, added) = client
                .add_stack_label(&args.stack, &args.label)
                .await
                .map_err(|e| e.to_string())?;
            if added {
                println!("added label \"{}\" to stack \"{}\"", args.label, stack.name);
            } else {
                println!(
                    "unchanged: stack \"{}\" has label \"{}\" already",
                    stack.name, args.label
                );
            }
        }
        StackCommand::Target(args) => {
            let (stack, agent, added) = client
                .target_stack(&args.stack, &args.agent)
                .await
                .map_err(|e| e.to_string())?;
            if added {
                println!(
                    "targeted stack \"{}\" at agent \"{}\"",
                    stack.name, agent.name
                );
            } else {
                println!(
                    "unchanged: stack \"{}\" targets agent \"{}\" already",
                    stack.name, agent.name
                );
            }
        }
        StackCommand::List => {
            let stacks = client.list_stacks().await.map_err(|e| e.to_string())?;
            if stacks.is_empty() {
                println!("no stacks");
                return Ok(());
            }
            let mut rows = Vec::with_capacity(stacks.len());
            for s in stacks {
                let labels = client.stack_labels(s.id).await.map_err(|e| e.to_string())?;
                rows.push(vec![s.name, s.id.to_string(), join_labels(labels)]);
            }
            print_table(&["NAME", "ID", "LABELS"], rows);
        }
    }
    Ok(())
}

fn join_labels(mut labels: Vec<String>) -> String {
    if labels.is_empty() {
        return "-".to_string();
    }
    labels.sort();
    labels.join(",")
}

/// Print rows under a header, with each column padded to its widest cell.
fn print_table(header: &[&str], rows: Vec<Vec<String>>) {
    print!("{}", format_table(header, &rows));
}

fn format_table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = header.iter().map(|h| h.len()).collect();
    for row in rows {
        for (w, cell) in widths.iter_mut().zip(row) {
            *w = (*w).max(cell.chars().count());
        }
    }
    let line = |cells: Vec<&str>| {
        let padded: Vec<String> = cells
            .iter()
            .zip(&widths)
            .map(|(c, w)| format!("{c:<w$}"))
            .collect();
        format!("{}\n", padded.join("  ").trim_end())
    };
    let mut out = line(header.to_vec());
    for row in rows {
        out.push_str(&line(row.iter().map(String::as_str).collect()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_must_be_key_value() {
        assert_eq!(parse_label("env:prod").unwrap(), "env:prod");
        assert!(parse_label("region:us:east").is_ok());
        for bad in ["prod", "env=prod", ":prod", "env:", "env: prod", ""] {
            let err = parse_label(bad).unwrap_err();
            assert!(err.contains("key:value"), "{bad}: {err}");
        }
        let long = format!("k:{}", "v".repeat(LABEL_MAX));
        assert!(parse_label(&long).unwrap_err().contains("too long"));
    }

    #[test]
    fn table_pads_columns() {
        let out = format_table(
            &["NAME", "ID"],
            &[
                vec!["a-long-name".into(), "1".into()],
                vec!["b".into(), "2".into()],
            ],
        );
        assert_eq!(out, "NAME         ID\na-long-name  1\nb            2\n");
    }
}
