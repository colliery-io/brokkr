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
