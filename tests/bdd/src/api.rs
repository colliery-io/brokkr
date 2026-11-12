/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! The broker API, for the steps that have no CLI command (a record read
//! with another generator's PAK, a health read by id) and for the fixtures and
//! the cleanup that the `after` hook runs as the admin.

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

/// One answer of the broker: the status and the body, JSON when it parses.
#[derive(Debug, Clone)]
pub struct Answer {
    pub status: StatusCode,
    pub body: Value,
}

impl Answer {
    pub fn ok(&self) -> bool {
        self.status.is_success()
    }

    /// The stable error code of a refusal, when the body has one.
    pub fn code(&self) -> Option<&str> {
        self.body.get("code").and_then(Value::as_str)
    }
}

/// Wait until `/healthz` answers 2xx.
pub async fn wait_for_ready(broker_url: &str, timeout: Duration) -> Result<()> {
    let http = Client::new();
    let url = format!("{}/healthz", broker_url.trim_end_matches('/'));
    let start = Instant::now();
    loop {
        if let Ok(resp) = http.get(&url).send().await
            && resp.status().is_success()
        {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(anyhow!("{url} did not answer 2xx within {timeout:?}"));
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[derive(Debug, Clone)]
pub struct Api {
    http: Client,
    base: String,
}

impl Api {
    pub fn new(broker_url: &str) -> Self {
        let trimmed = broker_url.trim_end_matches('/');
        let base = if trimmed.ends_with("/api/v1") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/api/v1")
        };
        Self {
            http: Client::new(),
            base,
        }
    }

    /// One request with a PAK. A non-2xx status is not an error here: the
    /// caller reads it from the [`Answer`].
    pub async fn call(
        &self,
        pak: &str,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Answer> {
        let url = format!("{}{}", self.base, path);
        let mut req = self
            .http
            .request(method.clone(), &url)
            .bearer_auth(pak)
            .header("Content-Type", "application/json");
        if let Some(b) = &body {
            req = req.body(serde_json::to_string(b)?);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("{method} {url}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let body = serde_json::from_str(&text).unwrap_or(Value::String(text));
        Ok(Answer { status, body })
    }

    /// Like [`Self::call`], but a non-2xx status is an error with the body.
    pub async fn expect(
        &self,
        pak: &str,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        let answer = self.call(pak, method.clone(), path, body).await?;
        if answer.ok() {
            Ok(answer.body)
        } else {
            Err(anyhow!(
                "{method} {path} answered {}: {}",
                answer.status,
                answer.body
            ))
        }
    }

    // Reads

    pub async fn list_generators(&self, admin: &str) -> Result<Vec<Value>> {
        as_list(self.expect(admin, Method::GET, "/generators", None).await?)
    }

    pub async fn list_agents(&self, pak: &str) -> Result<Vec<Value>> {
        as_list(self.expect(pak, Method::GET, "/agents", None).await?)
    }

    pub async fn list_stacks(&self, pak: &str) -> Result<Vec<Value>> {
        as_list(self.expect(pak, Method::GET, "/stacks", None).await?)
    }

    pub async fn get_agent(&self, admin: &str, id: Uuid) -> Result<Value> {
        self.expect(admin, Method::GET, &format!("/agents/{id}"), None)
            .await
    }

    pub async fn agent_registrations(&self, admin: &str, id: Uuid) -> Result<Vec<Value>> {
        as_list(
            self.expect(
                admin,
                Method::GET,
                &format!("/agents/{id}/registrations"),
                None,
            )
            .await?,
        )
    }

    pub async fn get_generator(&self, pak: &str, id: Uuid) -> Result<Answer> {
        self.call(pak, Method::GET, &format!("/generators/{id}"), None)
            .await
    }

    pub async fn stack_health(&self, pak: &str, id: Uuid) -> Result<Answer> {
        self.call(pak, Method::GET, &format!("/stacks/{id}/health"), None)
            .await
    }

    pub async fn agent_labels(&self, pak: &str, id: Uuid) -> Result<Answer> {
        self.call(pak, Method::GET, &format!("/agents/{id}/labels"), None)
            .await
    }

    /// The fleet record of one agent: `GET /agents/{id}/fleet-status`.
    pub async fn fleet_record(&self, admin: &str, id: Uuid) -> Result<Value> {
        let body = self
            .expect(
                admin,
                Method::GET,
                &format!("/agents/{id}/fleet-status"),
                None,
            )
            .await?;
        body.get("record")
            .cloned()
            .ok_or_else(|| anyhow!("no `record` in the fleet status: {body}"))
    }

    pub async fn fleet(&self, admin: &str) -> Result<Vec<Value>> {
        as_list(self.expect(admin, Method::GET, "/fleet", None).await?)
    }

    /// The agents registered with a generator: one `{agent_id, generator_id}`
    /// record each.
    pub async fn registered_agents(&self, pak: &str, generator: Uuid) -> Result<Vec<Value>> {
        as_list(
            self.expect(
                pak,
                Method::GET,
                &format!("/generators/{generator}/registered-agents"),
                None,
            )
            .await?,
        )
    }

    /// The full target state of an agent, as the agent reads it for its
    /// health tick: the latest object of each stack it serves.
    pub async fn target_state(&self, agent_pak: &str, agent: Uuid) -> Result<Vec<Value>> {
        as_list(
            self.expect(
                agent_pak,
                Method::GET,
                &format!("/agents/{agent}/target-state?mode=full"),
                None,
            )
            .await?,
        )
    }

    pub async fn deployment_object(&self, pak: &str, id: Uuid) -> Result<Answer> {
        self.call(pak, Method::GET, &format!("/deployment-objects/{id}"), None)
            .await
    }

    // What an agent sends

    pub async fn heartbeat(&self, agent_pak: &str, agent: Uuid) -> Result<Value> {
        self.expect(
            agent_pak,
            Method::POST,
            &format!("/agents/{agent}/heartbeat"),
            Some(json!({})),
        )
        .await
    }

    /// One health report of an agent on one deployment object.
    pub async fn report_health(
        &self,
        agent_pak: &str,
        agent: Uuid,
        object: Uuid,
        status: &str,
    ) -> Result<Value> {
        self.expect(
            agent_pak,
            Method::PATCH,
            &format!("/agents/{agent}/health-status"),
            Some(json!({
                "deployment_objects": [{
                    "id": object,
                    "status": status,
                    "summary": null,
                    "checked_at": now_rfc3339(),
                }]
            })),
        )
        .await
    }

    // Writes with no command

    pub async fn remove_target(&self, admin: &str, agent: Uuid, stack: Uuid) -> Result<Value> {
        self.expect(
            admin,
            Method::DELETE,
            &format!("/agents/{agent}/targets/{stack}"),
            None,
        )
        .await
    }

    // Writes the cleanup needs

    pub async fn deregister(&self, admin: &str, generator: Uuid, agent: Uuid) -> Result<Answer> {
        self.call(
            admin,
            Method::DELETE,
            &format!("/generators/{generator}/register"),
            Some(json!({ "agent_id": agent })),
        )
        .await
    }

    pub async fn delete_stack(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::DELETE, &format!("/stacks/{id}"), None)
            .await
    }

    pub async fn delete_agent(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::DELETE, &format!("/agents/{id}"), None)
            .await
    }

    pub async fn delete_generator(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::DELETE, &format!("/generators/{id}"), None)
            .await
    }
}

fn as_list(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(items) => Ok(items),
        other => Err(anyhow!("expected a list, got {other}")),
    }
}

/// The `id` of a record as a [`Uuid`].
pub fn id_of(record: &Value) -> Option<Uuid> {
    record
        .get("id")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// The record in `records` whose `name` is `name`.
pub fn named<'a>(records: &'a [Value], name: &str) -> Option<&'a Value> {
    records
        .iter()
        .find(|r| r.get("name").and_then(Value::as_str) == Some(name))
}

/// Now, in UTC, as RFC 3339 to the second (`2026-11-12T01:02:03Z`).
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after 1970")
        .as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

// Work orders (BROKKR-T-0399)

impl Api {
    /// `POST /work-orders` as the admin. The answer is kept whole: a refusal
    /// is something a scenario checks.
    pub async fn create_work_order(&self, admin: &str, body: Value) -> Result<Answer> {
        self.call(admin, Method::POST, "/work-orders", Some(body))
            .await
    }

    /// A work order by its id. One that ended comes from the log, with the
    /// status `COMPLETED` or `FAILED`.
    pub async fn get_work_order(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::GET, &format!("/work-orders/{id}"), None)
            .await
    }

    pub async fn work_order_log(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::GET, &format!("/work-order-log/{id}"), None)
            .await
    }

    pub async fn delete_work_order(&self, admin: &str, id: Uuid) -> Result<Answer> {
        self.call(admin, Method::DELETE, &format!("/work-orders/{id}"), None)
            .await
    }

    /// The pending work orders that reach an agent, as the agent reads them.
    pub async fn pending_work_orders(&self, agent_pak: &str, agent: Uuid) -> Result<Vec<Value>> {
        as_list(
            self.expect(
                agent_pak,
                Method::GET,
                &format!("/agents/{agent}/work-orders/pending"),
                None,
            )
            .await?,
        )
    }

    pub async fn claim_work_order(&self, agent_pak: &str, agent: Uuid, id: Uuid) -> Result<Answer> {
        self.call(
            agent_pak,
            Method::POST,
            &format!("/work-orders/{id}/claim"),
            Some(json!({ "agent_id": agent })),
        )
        .await
    }

    pub async fn complete_work_order(
        &self,
        agent_pak: &str,
        id: Uuid,
        success: bool,
        message: &str,
    ) -> Result<Answer> {
        self.call(
            agent_pak,
            Method::POST,
            &format!("/work-orders/{id}/complete"),
            Some(json!({ "success": success, "message": message, "retryable": true })),
        )
        .await
    }
}
