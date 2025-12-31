/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! HTTP API client for the Brokkr broker.

#![allow(dead_code)]

use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// API client for the Brokkr broker
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    admin_pak: String,
}

impl Client {
    pub fn new(base_url: &str, admin_pak: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.to_string(),
            admin_pak: admin_pak.to_string(),
        }
    }

    /// Wait for the broker to be ready
    pub async fn wait_for_ready(&self, timeout_secs: u64) -> Result<()> {
        let start = std::time::Instant::now();
        loop {
            match self.http.get(&format!("{}/healthz", self.base_url)).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => {
                    if start.elapsed() > Duration::from_secs(timeout_secs) {
                        return Err("Timeout waiting for broker".into());
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url)
            .header("Authorization", format!("Bearer {}", self.admin_pak))
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.body(serde_json::to_string(&b)?);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text).into());
        }

        if text.is_empty() {
            // For 204 No Content responses, return default
            return Ok(serde_json::from_str("null")?);
        }

        Ok(serde_json::from_str(&text)?)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::GET, path, None).await
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        self.request(reqwest::Method::POST, path, Some(body)).await
    }

    async fn put<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        self.request(reqwest::Method::PUT, path, Some(body)).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http.delete(&url)
            .header("Authorization", format!("Bearer {}", self.admin_pak))
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(format!("Delete failed: {}", text).into());
        }
        Ok(())
    }

    // =========================================================================
    // Agents
    // =========================================================================

    pub async fn list_agents(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/agents").await
    }

    pub async fn create_agent(&self, name: &str, cluster: &str) -> Result<Value> {
        self.post("/api/v1/agents", json!({
            "name": name,
            "cluster_name": cluster
        })).await
    }

    pub async fn get_agent(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/agents/{}", id)).await
    }

    pub async fn update_agent(&self, id: Uuid, updates: Value) -> Result<Value> {
        self.put(&format!("/api/v1/agents/{}", id), updates).await
    }

    pub async fn add_agent_label(&self, id: Uuid, label: &str) -> Result<Value> {
        self.post(&format!("/api/v1/agents/{}/labels", id), json!({
            "agent_id": id,
            "label": label
        })).await
    }

    pub async fn get_agent_labels(&self, id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/agents/{}/labels", id)).await
    }

    pub async fn add_agent_annotation(&self, id: Uuid, key: &str, value: &str) -> Result<Value> {
        self.post(&format!("/api/v1/agents/{}/annotations", id), json!({
            "agent_id": id,
            "key": key,
            "value": value
        })).await
    }

    pub async fn get_agent_annotations(&self, id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/agents/{}/annotations", id)).await
    }

    pub async fn add_agent_target(&self, agent_id: Uuid, stack_id: Uuid) -> Result<Value> {
        self.post(&format!("/api/v1/agents/{}/targets", agent_id), json!({
            "agent_id": agent_id,
            "stack_id": stack_id
        })).await
    }

    pub async fn get_agent_targets(&self, id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/agents/{}/targets", id)).await
    }

    pub async fn get_agent_stacks(&self, id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/agents/{}/stacks", id)).await
    }

    // =========================================================================
    // Generators
    // =========================================================================

    pub async fn create_generator(&self, name: &str, description: Option<&str>) -> Result<Value> {
        self.post("/api/v1/generators", json!({
            "name": name,
            "description": description
        })).await
    }

    pub async fn list_generators(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/generators").await
    }

    // =========================================================================
    // Stacks
    // =========================================================================

    pub async fn create_stack(&self, name: &str, description: Option<&str>, generator_id: Uuid) -> Result<Value> {
        self.post("/api/v1/stacks", json!({
            "name": name,
            "description": description,
            "generator_id": generator_id
        })).await
    }

    pub async fn list_stacks(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/stacks").await
    }

    pub async fn get_stack(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/stacks/{}", id)).await
    }

    pub async fn add_stack_label(&self, id: Uuid, label: &str) -> Result<Value> {
        // API expects just a plain string, not an object
        self.post(&format!("/api/v1/stacks/{}/labels", id), json!(label)).await
    }

    pub async fn get_stack_labels(&self, id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/stacks/{}/labels", id)).await
    }

    pub async fn add_stack_annotation(&self, id: Uuid, key: &str, value: &str) -> Result<Value> {
        self.post(&format!("/api/v1/stacks/{}/annotations", id), json!({
            "stack_id": id,
            "key": key,
            "value": value
        })).await
    }

    // =========================================================================
    // Deployment Objects
    // =========================================================================

    pub async fn create_deployment(&self, stack_id: Uuid, yaml: &str, is_deletion: bool) -> Result<Value> {
        let checksum = sha256_hex(yaml);
        self.post(&format!("/api/v1/stacks/{}/deployment-objects", stack_id), json!({
            "yaml_content": yaml,
            "yaml_checksum": checksum,
            "is_deletion_marker": is_deletion,
            "sequence_id": null
        })).await
    }

    pub async fn list_deployments(&self, stack_id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/stacks/{}/deployment-objects", stack_id)).await
    }

    pub async fn get_deployment(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/deployment-objects/{}", id)).await
    }

    pub async fn get_deployment_health(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/deployment-objects/{}/health", id)).await
    }

    pub async fn get_stack_health(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/stacks/{}/health", id)).await
    }

    // =========================================================================
    // Templates
    // =========================================================================

    pub async fn create_template(
        &self,
        name: &str,
        description: Option<&str>,
        content: &str,
        schema: &str,
    ) -> Result<Value> {
        self.post("/api/v1/templates", json!({
            "name": name,
            "description": description,
            "template_content": content,
            "parameters_schema": schema
        })).await
    }

    pub async fn list_templates(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/templates").await
    }

    pub async fn instantiate_template(
        &self,
        stack_id: Uuid,
        template_id: Uuid,
        parameters: Value,
    ) -> Result<Value> {
        self.post(
            &format!("/api/v1/stacks/{}/deployment-objects/from-template", stack_id),
            json!({
                "template_id": template_id,
                "parameters": parameters
            }),
        ).await
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/v1/templates/{}", id)).await
    }

    // =========================================================================
    // Work Orders
    // =========================================================================

    pub async fn create_work_order(
        &self,
        work_type: &str,
        yaml: &str,
        target_agent_ids: Option<Vec<Uuid>>,
        target_labels: Option<Vec<&str>>,
    ) -> Result<Value> {
        let mut body = json!({
            "work_type": work_type,
            "yaml_content": yaml
        });

        // Build targeting object
        let mut targeting = json!({});
        if let Some(ids) = target_agent_ids {
            targeting["agent_ids"] = json!(ids);
        }
        if let Some(labels) = target_labels {
            targeting["labels"] = json!(labels);
        }
        body["targeting"] = targeting;

        self.post("/api/v1/work-orders", body).await
    }

    pub async fn list_work_orders(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/work-orders").await
    }

    pub async fn get_work_order(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/work-orders/{}", id)).await
    }

    pub async fn get_work_order_log(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/work-order-log/{}", id)).await
    }

    pub async fn delete_work_order(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/v1/work-orders/{}", id)).await
    }

    // =========================================================================
    // Diagnostics
    // =========================================================================

    pub async fn create_diagnostic(
        &self,
        deployment_id: Uuid,
        agent_id: Uuid,
    ) -> Result<Value> {
        self.post(
            &format!("/api/v1/deployment-objects/{}/diagnostics", deployment_id),
            json!({
                "agent_id": agent_id,
                "requested_by": "e2e-test",
                "retention_minutes": 60
            }),
        ).await
    }

    pub async fn get_diagnostic(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/diagnostics/{}", id)).await
    }

    // =========================================================================
    // Webhooks
    // =========================================================================

    pub async fn create_webhook(
        &self,
        name: &str,
        url: &str,
        event_types: Vec<&str>,
        auth_header: Option<&str>,
    ) -> Result<Value> {
        let mut body = json!({
            "name": name,
            "url": url,
            "event_types": event_types
        });
        if let Some(auth) = auth_header {
            body["auth_header"] = json!(auth);
        }
        self.post("/api/v1/webhooks", body).await
    }

    pub async fn list_webhooks(&self) -> Result<Vec<Value>> {
        self.get("/api/v1/webhooks").await
    }

    pub async fn get_webhook(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/v1/webhooks/{}", id)).await
    }

    pub async fn update_webhook(&self, id: Uuid, updates: Value) -> Result<Value> {
        self.put(&format!("/api/v1/webhooks/{}", id), updates).await
    }

    pub async fn delete_webhook(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/v1/webhooks/{}", id)).await
    }

    pub async fn list_webhook_deliveries(&self, webhook_id: Uuid) -> Result<Vec<Value>> {
        self.get(&format!("/api/v1/webhooks/{}/deliveries", webhook_id)).await
    }

    pub async fn test_webhook(&self, id: Uuid) -> Result<Value> {
        self.post(&format!("/api/v1/webhooks/{}/test", id), json!({})).await
    }

    // =========================================================================
    // Audit Logs
    // =========================================================================

    pub async fn list_audit_logs(&self, limit: Option<i32>) -> Result<Value> {
        let path = match limit {
            Some(l) => format!("/api/v1/admin/audit-logs?limit={}", l),
            None => "/api/v1/admin/audit-logs".to_string(),
        };
        self.get(&path).await
    }

    // =========================================================================
    // Metrics
    // =========================================================================

    /// Fetch Prometheus metrics from the broker
    pub async fn get_metrics(&self) -> Result<String> {
        let url = format!("{}/metrics", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text).into());
        }

        Ok(text)
    }

    /// Fetch health check endpoint
    pub async fn get_healthz(&self) -> Result<String> {
        let url = format!("{}/healthz", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text).into());
        }

        Ok(text)
    }
}

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
