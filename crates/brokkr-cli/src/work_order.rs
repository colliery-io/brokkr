/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Work order commands (BROKKR-T-0407, BROKKR-A-0013).
//!
//! `brokkr work-order create|list|history|status|delete|logs|events`. A work order
//! runs once: an agent applies its YAML and watches it to its end. An admin
//! or a generator creates one, and the PAK that creates it owns it. A work
//! order has no name, so each command takes its id. Each command is a thin
//! shell over one [`BrokkrClient`] method; the texts are pure functions.

use std::collections::HashMap;
use std::path::PathBuf;

use brokkr_client::types::{WorkOrder, WorkOrderLog};
use brokkr_client::{BrokkrClient, BrokkrError, NewWorkOrderSpec, WorkOrderLogFilter};
use chrono::{DateTime, Utc};
use clap::{ArgGroup, Args, Subcommand, ValueEnum};
use uuid::Uuid;

use crate::fleet::{parse_label, print_table};

/// The statuses of a work order that has ended. The broker answers them
/// from the log.
const ENDED: [&str; 2] = ["COMPLETED", "FAILED"];

#[derive(Debug, Subcommand)]
pub enum WorkOrderCommand {
    /// Create a work order from a file of YAML.
    ///
    /// An agent that matches a target applies the YAML one time and watches
    /// it to its end. Give one target at least: an agent, a label or an
    /// annotation. The PAK that creates the work order owns it. With a
    /// generator PAK, each agent must be registered with the generator.
    /// Requires an admin PAK or a generator PAK.
    Create(CreateArgs),

    /// List the work orders that wait or run.
    ///
    /// An admin PAK lists all of them. A generator PAK lists the work
    /// orders of its generator. A work order that ended is not in this
    /// list: use brokkr work-order history.
    List,

    /// List the work orders that ended, newest first.
    ///
    /// Each row has the result, the agent that ran the work order, the
    /// owner, the time it ended and the first line of the message. An admin
    /// PAK lists all of them. A generator PAK lists the work orders of its
    /// generator.
    History(HistoryArgs),

    /// Show the status of one work order, and its result when it has ended.
    ///
    /// PENDING: no agent has it yet. CLAIMED: an agent runs it.
    /// RETRY_PENDING: a try failed and the broker waits to try again.
    /// COMPLETED or FAILED: it ended, and the command shows the message.
    /// Requires the PAK of the owner or an admin PAK.
    Status(WorkOrderRef),

    /// Delete a work order that has not ended.
    ///
    /// A work order that ended stays in the log and cannot be deleted.
    /// Requires the PAK of the owner or an admin PAK.
    Delete(WorkOrderRef),

    /// Show the retained pod log lines of a work order.
    ///
    /// The agent that runs the work order sends the lines of its pods. The
    /// broker keeps a short window, also after the work order ended.
    /// Requires the PAK of the owner or an admin PAK.
    Logs(WorkOrderTelemetryArgs),

    /// Show the retained Kubernetes events of a work order.
    ///
    /// The agent that runs the work order sends the events of its objects.
    /// The broker keeps a short window, also after the work order ended.
    /// Requires the PAK of the owner or an admin PAK.
    Events(WorkOrderTelemetryArgs),
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("targets")
        .required(true)
        .multiple(true)
        .args(["agents", "labels", "annotations"])
))]
pub struct CreateArgs {
    /// The file of YAML that the agent applies, for example a Job.
    #[arg(short = 'f', long = "filename", value_name = "PATH")]
    filename: PathBuf,

    /// The type of work: custom or build.
    #[arg(long = "type", value_name = "TYPE", default_value = "custom")]
    work_type: String,

    /// An agent that can have the work order, by name or id. Repeatable.
    #[arg(long = "agent", value_name = "NAME_OR_ID")]
    agents: Vec<String>,

    /// An agent with this label can have the work order. The label has the
    /// form key:value, for example env:prod. Repeatable.
    #[arg(long = "label", value_name = "LABEL", value_parser = parse_label)]
    labels: Vec<String>,

    /// An agent with this annotation can have the work order. The
    /// annotation has the form key=value, for example tier=gold. Repeatable.
    #[arg(long = "annotation", value_name = "KEY=VALUE", value_parser = parse_annotation)]
    annotations: Vec<(String, String)>,

    /// How long an agent can hold the work order, in seconds. After this
    /// time the broker gives the work order to the queue again. The
    /// broker's default is 3600.
    #[arg(long = "claim-timeout", value_name = "SECONDS")]
    claim_timeout: Option<i32>,

    /// How many times the broker tries again after a failure. The broker's
    /// default is 3.
    #[arg(long = "max-retries", value_name = "N")]
    max_retries: Option<i32>,

    /// The wait before the first new try, in seconds. The wait doubles for
    /// each new try. The broker's default is 60.
    #[arg(long = "backoff", value_name = "SECONDS")]
    backoff: Option<i32>,
}

/// How a work order ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ResultFilter {
    Success,
    Failure,
}

#[derive(Debug, Args)]
pub struct HistoryArgs {
    /// Only this type of work, for example custom or build.
    #[arg(long = "type", value_name = "TYPE")]
    work_type: Option<String>,

    /// Only the work orders that ended with this result.
    #[arg(long, value_enum, value_name = "RESULT")]
    result: Option<ResultFilter>,

    /// Only the work orders that this agent ran. Give the name or the id of
    /// the agent.
    #[arg(long, value_name = "NAME_OR_ID")]
    agent: Option<String>,

    /// At most this many rows, newest first.
    #[arg(long, value_name = "N")]
    limit: Option<i64>,
}

#[derive(Debug, Args)]
pub struct WorkOrderRef {
    /// The id of the work order.
    #[arg(value_name = "ID")]
    id: Uuid,
}

#[derive(Debug, Args)]
pub struct WorkOrderTelemetryArgs {
    /// The id of the work order.
    #[arg(value_name = "ID")]
    id: Uuid,

    /// At most this many rows, newest first. The broker's default is 500.
    #[arg(long, value_name = "N")]
    limit: Option<i64>,
}

/// Accept an annotation only in the form `key=value`. A label has the form
/// `key:value`; the two forms are different on purpose, as in the broker.
pub fn parse_annotation(raw: &str) -> Result<(String, String), String> {
    let shape =
        || format!("the annotation \"{raw}\" must have the form key=value, for example tier=gold");
    if raw.chars().any(char::is_whitespace) {
        return Err(format!("{}. An annotation cannot contain spaces", shape()));
    }
    match raw.split_once('=') {
        Some((key, value)) if !key.is_empty() && !value.is_empty() => {
            Ok((key.to_string(), value.to_string()))
        }
        _ => Err(shape()),
    }
}

pub async fn work_order(client: &BrokkrClient, command: WorkOrderCommand) -> Result<(), String> {
    match command {
        WorkOrderCommand::Create(args) => create(client, args).await,
        WorkOrderCommand::List => {
            let work_orders = client
                .list_work_orders()
                .await
                .map_err(|e| explain(&e, None))?;
            if work_orders.is_empty() {
                println!(
                    "no work orders wait or run. To see the ones that ended, run: \
                     brokkr work-order history"
                );
                return Ok(());
            }
            let generators = generator_names(client).await;
            let now = Utc::now();
            let rows = work_orders
                .iter()
                .map(|w| list_row(w, &generators, now))
                .collect();
            print_table(&["ID", "TYPE", "STATUS", "OWNER", "AGE"], rows);
            Ok(())
        }
        WorkOrderCommand::History(args) => {
            let agent_id = match &args.agent {
                Some(given) => Some(
                    client
                        .resolve_agent_id(given)
                        .await
                        .map_err(|e| e.to_string())?,
                ),
                None => None,
            };
            let entries = client
                .list_work_order_log(WorkOrderLogFilter {
                    work_type: args.work_type,
                    success: args.result.map(|r| r == ResultFilter::Success),
                    agent_id,
                    limit: args.limit,
                })
                .await
                .map_err(|e| explain(&e, None))?;
            if entries.is_empty() {
                println!(
                    "no work order that matches has ended. To see the ones that wait or \
                     run, run: brokkr work-order list"
                );
                return Ok(());
            }
            let generators = generator_names(client).await;
            let agents = agent_names(client).await;
            let rows = entries
                .iter()
                .map(|e| history_row(e, &generators, &agents))
                .collect();
            print_table(
                &["ID", "TYPE", "RESULT", "AGENT", "OWNER", "ENDED", "MESSAGE"],
                rows,
            );
            Ok(())
        }
        WorkOrderCommand::Status(args) => {
            let found = client
                .get_work_order(args.id)
                .await
                .map_err(|e| explain(&e, Some(args.id)))?;
            // The result and its message are in the log entry.
            let entry = if ENDED.contains(&found.status.as_str()) {
                Some(
                    client
                        .get_work_order_log(args.id)
                        .await
                        .map_err(|e| explain(&e, Some(args.id)))?,
                )
            } else {
                None
            };
            let generators = generator_names(client).await;
            let agents = agent_names(client).await;
            print!(
                "{}",
                status_report(&found, entry.as_ref(), &generators, &agents)
            );
            Ok(())
        }
        WorkOrderCommand::Delete(args) => {
            let found = client
                .get_work_order(args.id)
                .await
                .map_err(|e| explain(&e, Some(args.id)))?;
            if ENDED.contains(&found.status.as_str()) {
                return Err(format!(
                    "work order {} has ended ({}). A work order that ended stays in the log \
                     and cannot be deleted.",
                    args.id, found.status
                ));
            }
            client
                .delete_work_order(args.id)
                .await
                .map_err(|e| explain(&e, Some(args.id)))?;
            println!("deleted work order {}", args.id);
            Ok(())
        }
        WorkOrderCommand::Logs(args) => {
            let history = client
                .work_order_logs(args.id, args.limit)
                .await
                .map_err(|e| explain(&e, Some(args.id)))?;
            if history.lines.is_empty() {
                println!(
                    "no log lines for work order {} in the retained window. The agent that \
                     runs the work order sends the lines of its pods.",
                    args.id
                );
                return Ok(());
            }
            for l in history.lines {
                println!(
                    "{} {}/{}/{}: {}",
                    l.ts.to_rfc3339(),
                    l.namespace,
                    l.pod,
                    l.container,
                    l.line
                );
            }
            Ok(())
        }
        WorkOrderCommand::Events(args) => {
            let history = client
                .work_order_events(args.id, args.limit)
                .await
                .map_err(|e| explain(&e, Some(args.id)))?;
            if history.events.is_empty() {
                println!(
                    "no events for work order {} in the retained window",
                    args.id
                );
                return Ok(());
            }
            let rows = history
                .events
                .into_iter()
                .map(|e| {
                    let object = format!(
                        "{}/{}",
                        e.involved_object["kind"].as_str().unwrap_or("?"),
                        e.involved_object["name"].as_str().unwrap_or("?")
                    );
                    vec![
                        e.observed_at.to_rfc3339(),
                        e.event_type,
                        e.reason,
                        object,
                        e.message,
                    ]
                })
                .collect();
            print_table(&["OBSERVED", "TYPE", "REASON", "OBJECT", "MESSAGE"], rows);
            Ok(())
        }
    }
}

async fn create(client: &BrokkrClient, args: CreateArgs) -> Result<(), String> {
    let yaml_content = std::fs::read_to_string(&args.filename)
        .map_err(|e| format!("cannot read the file {}: {e}", args.filename.display()))?;
    if yaml_content.trim().is_empty() {
        return Err(format!("the file {} is empty", args.filename.display()));
    }

    // Each agent by name or id. The names stay for the texts.
    let mut agents: Vec<(Uuid, String)> = Vec::with_capacity(args.agents.len());
    for given in &args.agents {
        let agent = client.find_agent(given).await.map_err(|e| e.to_string())?;
        agents.push((agent.id, agent.name));
    }

    let created = client
        .create_work_order(NewWorkOrderSpec {
            work_type: args.work_type,
            yaml_content,
            agent_ids: agents.iter().map(|(id, _)| *id).collect(),
            labels: args.labels.clone(),
            annotations: args.annotations.clone(),
            claim_timeout_seconds: args.claim_timeout,
            max_retries: args.max_retries,
            backoff_seconds: args.backoff,
        })
        .await;
    let created = match created {
        Ok(created) => created,
        Err(e) if e.code() == Some("agent_not_registered") => {
            // The command to run needs the generator of this PAK.
            let generator = match client.whoami().await {
                Ok(me) => me.generator.and_then(|id| Uuid::parse_str(&id).ok()),
                Err(_) => None,
            };
            let message = match &e {
                BrokkrError::Api(body, _) => body.message.as_str(),
                _ => "",
            };
            return Err(not_registered_message(message, &agents, generator));
        }
        Err(e) => return Err(explain(&e, None)),
    };
    print!(
        "{}",
        create_report(&created, &agents, &args.labels, &args.annotations)
    );
    Ok(())
}

/// The names of the generators this PAK can read, by id. An admin PAK reads
/// all of them. A generator PAK reads its own. When the lookup fails, the
/// map is empty and the commands show ids.
async fn generator_names(client: &BrokkrClient) -> HashMap<Uuid, String> {
    if let Ok(all) = client.list_generators().await {
        return all.into_iter().map(|g| (g.id, g.name)).collect();
    }
    let mut names = HashMap::new();
    if let Ok(me) = client.whoami().await
        && let Some(id) = me.generator.and_then(|id| Uuid::parse_str(&id).ok())
        && let Ok(generator) = client.get_generator(id).await
    {
        names.insert(generator.id, generator.name);
    }
    names
}

/// The names of the agents this PAK can see, by id. Empty when the lookup
/// fails; the commands then show ids.
async fn agent_names(client: &BrokkrClient) -> HashMap<Uuid, String> {
    match client.list_agents().await {
        Ok(agents) => agents.into_iter().map(|a| (a.id, a.name)).collect(),
        Err(_) => HashMap::new(),
    }
}

/// The text for a refusal of a work order command, with what to do next.
/// `id` is the work order of the command, when it has one. An error with no
/// known code prints as the SDK gives it.
pub fn explain(error: &BrokkrError, id: Option<Uuid>) -> String {
    let which = match id {
        Some(id) => format!("work order {id}"),
        None => "the work order".to_string(),
    };
    match error.code() {
        Some("work_order_not_found") => match id {
            Some(id) => format!(
                "no work order has the id {id}. To see the ids of the work orders that wait \
                 or run, run: brokkr work-order list"
            ),
            None => "the broker did not find the work order".to_string(),
        },
        Some("work_order_not_owned") => format!(
            "{which} belongs to another owner. Use the PAK of the generator that owns it, or \
             an admin PAK. To see which PAK you use, run: brokkr whoami"
        ),
        Some("work_order_access_denied") => "an agent PAK cannot use the work order commands. \
             Use a generator PAK or an admin PAK. To see which PAK you use, run: brokkr whoami"
            .to_string(),
        Some("no_targeting_specified") => "the work order has no target. Give one target at \
             least: --agent, --label or --annotation"
            .to_string(),
        _ => error.to_string(),
    }
}

/// The text for `agent_not_registered` on a create. The broker's message
/// holds the id of the agent; `agents` are the agents of the command, with
/// their names; `generator` is the generator of the PAK, when it has one.
pub fn not_registered_message(
    broker_message: &str,
    agents: &[(Uuid, String)],
    generator: Option<Uuid>,
) -> String {
    let agent = agents
        .iter()
        .find(|(id, _)| broker_message.contains(&id.to_string()));
    let (shown, agent_arg) = match agent {
        Some((id, name)) => (format!("agent \"{name}\" ({id})"), id.to_string()),
        None => ("an agent of the command".to_string(), "<AGENT>".to_string()),
    };
    let generator_arg = match generator {
        Some(id) => id.to_string(),
        None => "<GENERATOR>".to_string(),
    };
    format!(
        "{shown} is not registered with your generator, so the generator cannot send it a \
         work order. The broker created nothing. An admin registers it: \
         brokkr register --agent {agent_arg} --generator {generator_arg}"
    )
}

/// The owner as a person reads it: `admin`, the name of the generator, or
/// its id when this PAK cannot read the name.
fn owner_text(owner: Option<Uuid>, generators: &HashMap<Uuid, String>) -> String {
    match owner {
        None => "admin".to_string(),
        Some(id) => match generators.get(&id) {
            Some(name) => name.clone(),
            None => id.to_string(),
        },
    }
}

/// An age in one unit: `45s`, `12m`, `3h`, `2d`.
pub fn age(now: DateTime<Utc>, then: DateTime<Utc>) -> String {
    let secs = (now - then).num_seconds().max(0);
    match secs {
        s if s < 60 => format!("{s}s"),
        s if s < 3_600 => format!("{}m", s / 60),
        s if s < 86_400 => format!("{}h", s / 3_600),
        s => format!("{}d", s / 86_400),
    }
}

/// One row of `brokkr work-order list`.
fn list_row(
    work_order: &WorkOrder,
    generators: &HashMap<Uuid, String>,
    now: DateTime<Utc>,
) -> Vec<String> {
    vec![
        work_order.id.to_string(),
        work_order.work_type.clone(),
        work_order.status.clone(),
        owner_text(work_order.owner_generator_id, generators),
        age(now, work_order.created_at),
    ]
}

/// The longest message that a row of `brokkr work-order history` shows.
const MESSAGE_MAX: usize = 60;

/// The first line of a message, cut to [`MESSAGE_MAX`] characters. `-` when
/// there is no message.
pub fn one_line(message: Option<&str>) -> String {
    let first = message
        .and_then(|m| m.lines().find(|l| !l.trim().is_empty()))
        .map(str::trim)
        .unwrap_or("");
    if first.is_empty() {
        return "-".to_string();
    }
    if first.chars().count() <= MESSAGE_MAX {
        return first.to_string();
    }
    let cut: String = first.chars().take(MESSAGE_MAX - 3).collect();
    format!("{cut}...")
}

/// One row of `brokkr work-order history`.
fn history_row(
    entry: &WorkOrderLog,
    generators: &HashMap<Uuid, String>,
    agents: &HashMap<Uuid, String>,
) -> Vec<String> {
    let agent = match entry.claimed_by {
        None => "-".to_string(),
        Some(id) => agents.get(&id).cloned().unwrap_or_else(|| id.to_string()),
    };
    vec![
        entry.id.to_string(),
        entry.work_type.clone(),
        if entry.success { "success" } else { "failure" }.to_string(),
        agent,
        owner_text(entry.owner_generator_id, generators),
        entry.completed_at.to_rfc3339(),
        one_line(entry.result_message.as_deref()),
    ]
}

/// The text of `brokkr work-order create`.
pub fn create_report(
    created: &WorkOrder,
    agents: &[(Uuid, String)],
    labels: &[String],
    annotations: &[(String, String)],
) -> String {
    let mut targets: Vec<String> = Vec::new();
    targets.extend(agents.iter().map(|(_, name)| format!("agent \"{name}\"")));
    targets.extend(labels.iter().map(|l| format!("label {l}")));
    targets.extend(
        annotations
            .iter()
            .map(|(k, v)| format!("annotation {k}={v}")),
    );
    format!(
        "created work order {id} (type {work_type})\n\
         targets: {targets}\n\
         An agent that matches a target claims the work order on its next poll.\n\
         To follow it, run: brokkr work-order status {id}\n",
        id = created.id,
        work_type = created.work_type,
        targets = targets.join(", "),
    )
}

/// The text of `brokkr work-order status`. `entry` is the log entry of a
/// work order that ended.
pub fn status_report(
    work_order: &WorkOrder,
    entry: Option<&WorkOrderLog>,
    generators: &HashMap<Uuid, String>,
    agents: &HashMap<Uuid, String>,
) -> String {
    let agent_text = |id: Option<Uuid>| match id {
        None => "an agent".to_string(),
        Some(id) => match agents.get(&id) {
            Some(name) => format!("agent \"{name}\""),
            None => format!("agent {id}"),
        },
    };
    let mut out = format!("work order {}\n", work_order.id);
    out.push_str(&format!("type: {}\n", work_order.work_type));
    out.push_str(&format!(
        "owner: {}\n",
        owner_text(work_order.owner_generator_id, generators)
    ));
    out.push_str(&format!(
        "created: {}\n",
        work_order.created_at.to_rfc3339()
    ));

    match (work_order.status.as_str(), entry) {
        (_, Some(entry)) => {
            out.push_str(&format!(
                "status: {} at {}. {} ran it. Tries after the first: {}.\n",
                work_order.status,
                entry.completed_at.to_rfc3339(),
                agent_text(entry.claimed_by),
                entry.retries_attempted
            ));
            out.push_str(&format!(
                "result: {}\n",
                if entry.success { "success" } else { "failure" }
            ));
            match entry.result_message.as_deref() {
                Some(message) if !message.trim().is_empty() => {
                    out.push_str(&format!("message: {message}\n"))
                }
                _ => out.push_str("message: none\n"),
            }
        }
        ("CLAIMED", None) => {
            out.push_str(&format!(
                "status: CLAIMED. {} runs it{}.\n",
                agent_text(work_order.claimed_by),
                match work_order.claimed_at {
                    Some(at) => format!(", since {}", at.to_rfc3339()),
                    None => String::new(),
                }
            ));
            out.push_str(&format!(
                "The agent can hold it for {} seconds.\n",
                work_order.claim_timeout_seconds
            ));
        }
        ("RETRY_PENDING", None) => {
            out.push_str(&format!(
                "status: RETRY_PENDING. Try {} of {} failed.\n",
                work_order.retry_count,
                work_order.max_retries + 1
            ));
            if let Some(error) = work_order.last_error.as_deref() {
                out.push_str(&format!("last error: {error}\n"));
            }
            if let Some(at) = work_order.next_retry_after {
                out.push_str(&format!(
                    "The broker gives it to the queue again after {}.\n",
                    at.to_rfc3339()
                ));
            }
        }
        (status, None) => {
            out.push_str(&format!("status: {status}. No agent has claimed it yet.\n"));
            out.push_str(
                "An agent that matches a target claims the work order on its next poll.\n",
            );
        }
    }
    out.push_str(&format!(
        "To see the lines of its pods, run: brokkr work-order logs {}\n",
        work_order.id
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_800_000_000 + secs, 0).unwrap()
    }

    fn pending(owner: Option<Uuid>) -> WorkOrder {
        WorkOrder {
            id: Uuid::from_u128(1),
            created_at: at(0),
            updated_at: at(0),
            work_type: "custom".to_string(),
            yaml_content: "kind: Job\n".to_string(),
            status: "PENDING".to_string(),
            claimed_by: None,
            claimed_at: None,
            claim_timeout_seconds: 3600,
            max_retries: 3,
            retry_count: 0,
            backoff_seconds: 60,
            next_retry_after: None,
            last_error: None,
            last_error_at: None,
            owner_generator_id: owner,
        }
    }

    fn api_error(status: u16, code: &str, message: &str) -> BrokkrError {
        BrokkrError::Api(
            brokkr_client::types::ErrorResponse {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            },
            status.try_into().unwrap(),
        )
    }

    #[test]
    fn annotation_must_be_key_value() {
        assert_eq!(
            parse_annotation("tier=gold").unwrap(),
            ("tier".to_string(), "gold".to_string())
        );
        // Only the first `=` splits: a value can hold one.
        assert_eq!(
            parse_annotation("expr=a=b").unwrap(),
            ("expr".to_string(), "a=b".to_string())
        );
        for bad in ["tier", "tier:gold", "=gold", "tier=", "tier = gold", ""] {
            let err = parse_annotation(bad).unwrap_err();
            assert!(err.contains("key=value"), "{bad}: {err}");
        }
    }

    #[test]
    fn age_uses_one_unit() {
        assert_eq!(age(at(45), at(0)), "45s");
        assert_eq!(age(at(12 * 60 + 5), at(0)), "12m");
        assert_eq!(age(at(3 * 3_600 + 59), at(0)), "3h");
        assert_eq!(age(at(2 * 86_400 + 1), at(0)), "2d");
        // A clock that is behind the broker's gives no negative age.
        assert_eq!(age(at(0), at(30)), "0s");
    }

    #[test]
    fn each_refusal_says_what_to_do_next() {
        let id = Uuid::from_u128(9);

        let not_found = explain(
            &api_error(404, "work_order_not_found", "work order not found"),
            Some(id),
        );
        assert!(
            not_found.starts_with(&format!("no work order has the id {id}.")),
            "{not_found}"
        );
        assert!(not_found.contains("brokkr work-order list"), "{not_found}");

        let not_owned = explain(&api_error(403, "work_order_not_owned", "x"), Some(id));
        assert!(
            not_owned.starts_with(&format!("work order {id} belongs to another owner.")),
            "{not_owned}"
        );
        assert!(not_owned.contains("admin PAK"), "{not_owned}");
        assert!(not_owned.contains("brokkr whoami"), "{not_owned}");

        let agent_pak = explain(&api_error(403, "work_order_access_denied", "x"), None);
        assert!(
            agent_pak.starts_with("an agent PAK cannot use the work order commands."),
            "{agent_pak}"
        );

        let no_target = explain(&api_error(400, "no_targeting_specified", "x"), None);
        assert!(no_target.contains("--agent, --label or --annotation"));

        // Another error prints as the SDK gives it, with its status and code.
        let other = explain(&api_error(500, "internal_error", "boom"), Some(id));
        assert_eq!(other, "500 internal_error: boom");
    }

    #[test]
    fn an_unregistered_agent_names_the_register_command() {
        let (agent, generator) = (Uuid::from_u128(3), Uuid::from_u128(4));
        let agents = [
            (Uuid::from_u128(2), "prod-1".to_string()),
            (agent, "prod-2".to_string()),
        ];
        let broker = format!(
            "target agent {agent} is not registered with the generator that owns the work order"
        );
        let message = not_registered_message(&broker, &agents, Some(generator));
        assert!(
            message.starts_with(&format!(
                "agent \"prod-2\" ({agent}) is not registered with your generator"
            )),
            "{message}"
        );
        assert!(message.contains("The broker created nothing."), "{message}");
        assert!(
            message.ends_with(&format!(
                "brokkr register --agent {agent} --generator {generator}"
            )),
            "{message}"
        );

        // With no generator and no match, the command still has its shape.
        let bare = not_registered_message("target agent is not registered", &agents, None);
        assert!(
            bare.ends_with("brokkr register --agent <AGENT> --generator <GENERATOR>"),
            "{bare}"
        );
    }

    #[test]
    fn the_owner_is_admin_a_name_or_an_id() {
        let known = Uuid::from_u128(5);
        let unknown = Uuid::from_u128(6);
        let generators = HashMap::from([(known, "team-robin".to_string())]);
        assert_eq!(owner_text(None, &generators), "admin");
        assert_eq!(owner_text(Some(known), &generators), "team-robin");
        assert_eq!(owner_text(Some(unknown), &generators), unknown.to_string());
    }

    #[test]
    fn a_list_row_has_the_id_type_status_owner_and_age() {
        let owner = Uuid::from_u128(5);
        let generators = HashMap::from([(owner, "team-robin".to_string())]);
        let row = list_row(&pending(Some(owner)), &generators, at(125));
        assert_eq!(
            row,
            vec![
                Uuid::from_u128(1).to_string(),
                "custom".to_string(),
                "PENDING".to_string(),
                "team-robin".to_string(),
                "2m".to_string(),
            ]
        );
        let of_admin = list_row(&pending(None), &generators, at(5));
        assert_eq!(of_admin[3], "admin");
    }

    #[test]
    fn a_message_is_one_line_and_cut() {
        assert_eq!(one_line(None), "-");
        assert_eq!(one_line(Some("  \n ")), "-");
        assert_eq!(one_line(Some("sha256:abc")), "sha256:abc");
        assert_eq!(one_line(Some("\nfirst line\nsecond line")), "first line");
        let long = "x".repeat(MESSAGE_MAX + 20);
        let cut = one_line(Some(&long));
        assert_eq!(cut.chars().count(), MESSAGE_MAX);
        assert!(cut.ends_with("..."), "{cut}");
        // A message of the longest size is not cut.
        let fits = "y".repeat(MESSAGE_MAX);
        assert_eq!(one_line(Some(&fits)), fits);
    }

    #[test]
    fn a_history_row_has_the_result_the_agent_the_owner_and_the_message() {
        let (agent, owner) = (Uuid::from_u128(2), Uuid::from_u128(5));
        let agents = HashMap::from([(agent, "prod-1".to_string())]);
        let generators = HashMap::from([(owner, "team-robin".to_string())]);
        let mut entry = WorkOrderLog {
            id: Uuid::from_u128(1),
            work_type: "custom".to_string(),
            created_at: at(0),
            claimed_at: Some(at(10)),
            completed_at: at(60),
            claimed_by: Some(agent),
            success: false,
            retries_attempted: 3,
            result_message: Some("job failed\nBackoffLimitExceeded".to_string()),
            yaml_content: "kind: Job\n".to_string(),
            owner_generator_id: Some(owner),
        };
        assert_eq!(
            history_row(&entry, &generators, &agents),
            vec![
                Uuid::from_u128(1).to_string(),
                "custom".to_string(),
                "failure".to_string(),
                "prod-1".to_string(),
                "team-robin".to_string(),
                at(60).to_rfc3339(),
                "job failed".to_string(),
            ]
        );
        // An admin's, with success, by an agent this PAK cannot name, and
        // one that no agent ran.
        entry.success = true;
        entry.owner_generator_id = None;
        entry.result_message = None;
        let unknown = Uuid::from_u128(8);
        entry.claimed_by = Some(unknown);
        let row = history_row(&entry, &generators, &agents);
        assert_eq!(
            (
                row[2].as_str(),
                row[3].as_str(),
                row[4].as_str(),
                row[6].as_str()
            ),
            ("success", unknown.to_string().as_str(), "admin", "-")
        );
        entry.claimed_by = None;
        assert_eq!(history_row(&entry, &generators, &agents)[3], "-");
    }

    #[test]
    fn create_prints_the_id_the_targets_and_the_next_step() {
        let created = pending(None);
        let report = create_report(
            &created,
            &[(Uuid::from_u128(2), "prod-1".to_string())],
            &["env:prod".to_string()],
            &[("tier".to_string(), "gold".to_string())],
        );
        let id = created.id;
        assert_eq!(
            report,
            format!(
                "created work order {id} (type custom)\n\
                 targets: agent \"prod-1\", label env:prod, annotation tier=gold\n\
                 An agent that matches a target claims the work order on its next poll.\n\
                 To follow it, run: brokkr work-order status {id}\n"
            )
        );
    }

    #[test]
    fn status_shows_each_state_of_a_work_order() {
        let agent = Uuid::from_u128(2);
        let agents = HashMap::from([(agent, "prod-1".to_string())]);
        let none = HashMap::new();

        let waiting = status_report(&pending(None), None, &none, &agents);
        assert!(waiting.contains("owner: admin\n"), "{waiting}");
        assert!(
            waiting.contains("status: PENDING. No agent has claimed it yet.\n"),
            "{waiting}"
        );

        let mut claimed = pending(None);
        claimed.status = "CLAIMED".to_string();
        claimed.claimed_by = Some(agent);
        claimed.claimed_at = Some(at(10));
        let running = status_report(&claimed, None, &none, &agents);
        assert!(
            running.contains(&format!(
                "status: CLAIMED. agent \"prod-1\" runs it, since {}.\n",
                at(10).to_rfc3339()
            )),
            "{running}"
        );
        assert!(running.contains("hold it for 3600 seconds"), "{running}");

        let mut retry = pending(None);
        retry.status = "RETRY_PENDING".to_string();
        retry.retry_count = 1;
        retry.last_error = Some("image pull failed".to_string());
        retry.next_retry_after = Some(at(120));
        let again = status_report(&retry, None, &none, &agents);
        assert!(
            again.contains("status: RETRY_PENDING. Try 1 of 4 failed.\n"),
            "{again}"
        );
        assert!(again.contains("last error: image pull failed\n"), "{again}");
        assert!(again.contains(&at(120).to_rfc3339()), "{again}");
    }

    #[test]
    fn status_shows_the_result_of_a_work_order_that_ended() {
        let agent = Uuid::from_u128(2);
        let agents = HashMap::from([(agent, "prod-1".to_string())]);
        let none = HashMap::new();
        let entry = |success: bool, message: Option<&str>| WorkOrderLog {
            id: Uuid::from_u128(1),
            work_type: "custom".to_string(),
            created_at: at(0),
            claimed_at: Some(at(10)),
            completed_at: at(60),
            claimed_by: Some(agent),
            success,
            retries_attempted: 0,
            result_message: message.map(str::to_string),
            yaml_content: "kind: Job\n".to_string(),
            owner_generator_id: None,
        };

        let mut ended = pending(None);
        ended.status = "COMPLETED".to_string();
        let good = status_report(
            &ended,
            Some(&entry(true, Some("sha256:abc"))),
            &none,
            &agents,
        );
        assert!(
            good.contains(&format!(
                "status: COMPLETED at {}. agent \"prod-1\" ran it. Tries after the first: 0.\n",
                at(60).to_rfc3339()
            )),
            "{good}"
        );
        assert!(good.contains("result: success\n"), "{good}");
        assert!(good.contains("message: sha256:abc\n"), "{good}");
        assert!(
            good.ends_with(&format!(
                "To see the lines of its pods, run: brokkr work-order logs {}\n",
                ended.id
            )),
            "{good}"
        );

        ended.status = "FAILED".to_string();
        let bad = status_report(&ended, Some(&entry(false, None)), &none, &agents);
        assert!(bad.contains("status: FAILED at "), "{bad}");
        assert!(bad.contains("result: failure\n"), "{bad}");
        assert!(bad.contains("message: none\n"), "{bad}");
    }
}
