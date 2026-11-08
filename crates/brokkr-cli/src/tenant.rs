/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Tenant commands (BROKKR-T-0370): `brokkr tenant create|list`.
//!
//! The admin's one-time steps for a tenant are create it, activate its agent
//! and register the agent with it. The last two were already here; create
//! needed curl, the console, or `brokkr-broker create generator` on the
//! broker host. Each command is a thin shell over one [`BrokkrClient`]
//! method. The API calls a tenant a generator.

use brokkr_client::BrokkrClient;
use clap::{Args, Subcommand};

use crate::fleet::print_table;

#[derive(Debug, Subcommand)]
pub enum TenantCommand {
    /// Create a tenant and print its PAK, once.
    ///
    /// The broker keeps only a hash of the PAK, so it is shown once: store
    /// it where the tenant's CI reads it. The only way back from a lost PAK
    /// is a rotation. Requires an admin PAK. The API calls a tenant a
    /// generator.
    Create(TenantCreateArgs),

    /// List the tenants with their name, id, status and last activity.
    ///
    /// Requires an admin PAK. The system generator is not a tenant and is
    /// not listed.
    List,

    /// Rotate a tenant's PAK and print the new one, once.
    ///
    /// With a tenant PAK, rotates its own; give no tenant. With an admin
    /// PAK, give the tenant by name or id. The current PAK stops at once.
    Rotate(TenantRotateArgs),
}

#[derive(Debug, Args)]
pub struct TenantRotateArgs {
    /// The name or the id of the tenant. Admin only; a tenant PAK rotates
    /// its own.
    #[arg(value_name = "NAME_OR_ID")]
    tenant: Option<String>,
}

#[derive(Debug, Args)]
pub struct TenantCreateArgs {
    /// The name of the tenant: a team or a pipeline, for example team-acme.
    #[arg(value_name = "NAME")]
    name: String,

    /// What the tenant deploys, for whoever inherits it.
    #[arg(long, value_name = "TEXT")]
    description: Option<String>,
}

pub async fn tenant(client: &BrokkrClient, command: TenantCommand) -> Result<(), String> {
    match command {
        TenantCommand::Create(args) => {
            let created = client
                .create_generator(&args.name, args.description.as_deref())
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "created tenant \"{}\" ({})",
                created.generator.name, created.generator.id
            );
            println!("PAK: {}", created.pak);
            println!(
                "The PAK is shown once. Store it where the tenant's CI reads it; \
                 the only way back is a rotation. Next: brokkr register --agent <AGENT> \
                 --generator {}",
                created.generator.name
            );
        }
        TenantCommand::List => {
            let tenants = client.list_generators().await.map_err(|e| e.to_string())?;
            let rows: Vec<Vec<String>> = tenants
                .into_iter()
                .filter(|g| !g.is_system)
                .map(|g| {
                    vec![
                        g.name,
                        g.id.to_string(),
                        if g.is_active { "active" } else { "inactive" }.to_string(),
                        g.last_active_at
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_else(|| "never".to_string()),
                        g.description.unwrap_or_default(),
                    ]
                })
                .collect();
            if rows.is_empty() {
                println!("no tenants");
                return Ok(());
            }
            print_table(
                &["NAME", "ID", "STATUS", "LAST ACTIVE", "DESCRIPTION"],
                rows,
            );
        }
        TenantCommand::Rotate(args) => {
            let id = match &args.tenant {
                Some(tenant) => client
                    .resolve_generator_id(tenant)
                    .await
                    .map_err(|e| e.to_string())?,
                None => {
                    let me = client.whoami().await.map_err(|e| e.to_string())?;
                    match me.generator {
                        Some(id) => uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?,
                        None => {
                            return Err(
                                "this PAK is not a tenant PAK: give the tenant to rotate by name or id"
                                    .to_string(),
                            );
                        }
                    }
                }
            };
            let rotated = client
                .rotate_generator_pak(id)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "rotated the PAK of tenant \"{}\" ({})",
                rotated.generator.name, rotated.generator.id
            );
            println!("PAK: {}", rotated.pak);
            println!(
                "The PAK is shown once. The previous PAK stopped: update every pipeline that uses it."
            );
        }
    }
    Ok(())
}

/// The text of `brokkr whoami` (BROKKR-T-0381).
pub async fn whoami(client: &BrokkrClient) -> Result<(), String> {
    let me = client.whoami().await.map_err(|e| e.to_string())?;
    let url = client.base_url().trim_end_matches('/');
    let url = url.strip_suffix("/api/v1").unwrap_or(url);
    println!("broker: {url}");
    if me.admin {
        println!(
            "PAK: admin{}",
            if me.readonly { " (read-only)" } else { "" }
        );
    } else if let Some(id) = me.generator {
        let id = uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        let tenant = client.get_generator(id).await.map_err(|e| e.to_string())?;
        println!("PAK: tenant \"{}\" ({})", tenant.name, tenant.id);
    } else if let Some(id) = me.agent {
        let id = uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?;
        let agent = client.get_agent(id).await.map_err(|e| e.to_string())?;
        println!(
            "PAK: agent \"{}\" ({}) in cluster \"{}\"",
            agent.name, agent.id, agent.cluster_name
        );
    } else {
        println!("PAK: unknown kind");
    }
    Ok(())
}
