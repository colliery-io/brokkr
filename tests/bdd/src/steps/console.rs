/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! What the operator console shows (BROKKR-T-0400). Each step reads a view
//! with a browser (`crate::console`) until it shows what the step expects.

use cucumber::then;

use crate::console::{self, Fleet, Overview};
use crate::world::World;

/// The status of a work order as the console shows it, in lower case.
fn lower(s: &Option<String>) -> String {
    s.as_deref().unwrap_or("").to_lowercase()
}

#[then(expr = "the Fleet view shows the agent {string} as {word}")]
async fn fleet_shows_health(w: &mut World, agent: String, health: String) {
    let name = w.agent(&agent).name.clone();
    let url = w.env.broker_url.clone();
    console::until(
        &format!("the Fleet view shows the agent {agent:?} as {health}"),
        || console::fleet(&url),
        |f: &Fleet| match f.agent(&name) {
            None => Err(format!("no row for the agent {name:?}")),
            Some(row) if lower(&row.health) == health => Ok(()),
            Some(row) => Err(format!("the health pill reads {:?}", row.health)),
        },
    )
    .await;
}

#[then(expr = "the Fleet view shows no failing and no degraded pill for the agent {string}")]
async fn fleet_shows_no_bad_pill(w: &mut World, agent: String) {
    let name = w.agent(&agent).name.clone();
    let url = w.env.broker_url.clone();
    console::until(
        &format!("the Fleet view shows no failing and no degraded pill for {agent:?}"),
        || console::fleet(&url),
        |f: &Fleet| match f.agent(&name) {
            None => Err(format!("no row for the agent {name:?}")),
            Some(row) if ["failing", "degraded"].contains(&lower(&row.health).as_str()) => {
                Err(format!("the health pill reads {:?}", row.health))
            }
            Some(_) => Ok(()),
        },
    )
    .await;
}

/// The Fleet tiles and the Overview tiles give the same counts. The two
/// reads are not at the same moment, so a change between them makes the
/// step read both again.
#[then("the Overview counts agree with the Fleet view")]
async fn overview_agrees_with_fleet(w: &mut World) {
    let url = w.env.broker_url.clone();
    console::until(
        "the Overview counts agree with the Fleet view",
        || {
            let url = url.clone();
            async move {
                let f = console::fleet(&url).await?;
                let o = console::overview(&url).await?;
                Ok((f, o))
            }
        },
        |(f, o): &(Fleet, Overview)| {
            let tile = |tiles: &std::collections::BTreeMap<String, String>, k: &str| {
                tiles.get(k).cloned().unwrap_or_default()
            };
            let fleet_active = format!("{}/{}", tile(&f.tiles, "active"), tile(&f.tiles, "agents"));
            let pairs = [
                (
                    "active agents",
                    fleet_active,
                    tile(&o.tiles, "active agents"),
                ),
                (
                    "degraded",
                    tile(&f.tiles, "degraded"),
                    tile(&o.tiles, "degraded"),
                ),
                (
                    "failing",
                    tile(&f.tiles, "failing"),
                    tile(&o.tiles, "failing"),
                ),
            ];
            let wrong: Vec<String> = pairs
                .iter()
                .filter(|(_, fv, ov)| fv.is_empty() || fv != ov)
                .map(|(k, fv, ov)| format!("{k}: Fleet {fv:?}, Overview {ov:?}"))
                .collect();
            // Each agent row of the Fleet view is in the agents tile.
            let rows = f.agents.len().to_string();
            if rows != tile(&f.tiles, "agents") {
                return Err(format!(
                    "the Fleet view has {rows} rows and the agents tile reads {:?}",
                    tile(&f.tiles, "agents")
                ));
            }
            if wrong.is_empty() {
                Ok(())
            } else {
                Err(wrong.join("; "))
            }
        },
    )
    .await;
}

#[then(expr = "the Overview shows the work order {string} as {word}")]
async fn overview_shows_work_order(w: &mut World, name: String, status: String) {
    let id = *w
        .work_orders
        .get(&name)
        .unwrap_or_else(|| panic!("no work order {name:?} in this scenario"));
    let id8: String = id.to_string().chars().take(8).collect();
    let url = w.env.broker_url.clone();
    console::until(
        &format!("the Overview shows the work order {name:?} as {status}"),
        || console::overview(&url),
        |o: &Overview| match o.work_orders.iter().find(|r| r.id8 == id8) {
            None => Err(format!("no row for the work order {id8}")),
            Some(r) if lower(&r.status) == status.to_lowercase() => Ok(()),
            Some(r) => Err(format!("the status pill reads {:?}", r.status)),
        },
    )
    .await;
}

#[then(expr = "the Overview shows the cluster of the agent {string} as {int} of {int} up")]
async fn overview_shows_cluster_up(w: &mut World, agent: String, up: u32, total: u32) {
    let cluster = w.agent(&agent).cluster.clone();
    let want = format!("{up}/{total} up");
    let url = w.env.broker_url.clone();
    console::until(
        &format!("the Overview shows the cluster of {agent:?} as {want}"),
        || console::overview(&url),
        |o: &Overview| match o.cluster(&cluster) {
            None => Err(format!("no row for the cluster {cluster:?}")),
            Some(c) if c.caption == want => Ok(()),
            Some(c) => Err(format!("the row reads {:?}", c.caption)),
        },
    )
    .await;
}
