const BASE_URL = process.env.REACT_APP_BROKER_URL || 'http://localhost:30300';
const AUTH = `Bearer ${process.env.REACT_APP_ADMIN_PAK || ''}`;

const sha256 = async (str) => {
  const data = new TextEncoder().encode(str);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return Array.from(new Uint8Array(hash)).map(b => b.toString(16).padStart(2, '0')).join('');
};

const request = async (path, options = {}) => {
  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: { 'Authorization': AUTH, 'Content-Type': 'application/json', ...options.headers }
  });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.status === 204 ? null : res.json();
};

// Agents
export const getAgents = () => request('/api/v1/agents');
export const getAgentLabels = (id) => request(`/api/v1/agents/${id}/labels`);
export const getAgentAnnotations = (id) => request(`/api/v1/agents/${id}/annotations`);
export const getAgentTargets = (id) => request(`/api/v1/agents/${id}/targets`);
export const getAgentEvents = (id) => request(`/api/v1/agents/${id}/events?limit=20`);
export const getAgentStacks = (id) => request(`/api/v1/agents/${id}/stacks`);
export const addAgentLabel = (id, label) => request(`/api/v1/agents/${id}/labels`, { method: 'POST', body: JSON.stringify({ agent_id: id, label }) });
export const removeAgentLabel = (id, label) => request(`/api/v1/agents/${id}/labels/${label}`, { method: 'DELETE' });
export const addAgentAnnotation = (id, key, value) => request(`/api/v1/agents/${id}/annotations`, { method: 'POST', body: JSON.stringify({ agent_id: id, key, value }) });
export const removeAgentAnnotation = (id, key) => request(`/api/v1/agents/${id}/annotations/${key}`, { method: 'DELETE' });
export const addAgentTarget = (id, stackId) => request(`/api/v1/agents/${id}/targets`, { method: 'POST', body: JSON.stringify({ agent_id: id, stack_id: stackId }) });
export const removeAgentTarget = (id, stackId) => request(`/api/v1/agents/${id}/targets/${stackId}`, { method: 'DELETE' });
export const createAgent = (name, cluster) => request('/api/v1/agents', { method: 'POST', body: JSON.stringify({ name, cluster_name: cluster }) });
export const updateAgent = (id, updates) => request(`/api/v1/agents/${id}`, { method: 'PUT', body: JSON.stringify(updates) });
export const rotateAgentPak = (id) => request(`/api/v1/agents/${id}/rotate-pak`, { method: 'POST' });

// Stacks
export const getStacks = () => request('/api/v1/stacks');
export const getStackLabels = (id) => request(`/api/v1/stacks/${id}/labels`);
export const getStackAnnotations = (id) => request(`/api/v1/stacks/${id}/annotations`);
export const getStackDeployments = (id) => request(`/api/v1/stacks/${id}/deployment-objects`);
export const createStack = (name, description, generatorId) => request('/api/v1/stacks', { method: 'POST', body: JSON.stringify({ name, description: description || null, generator_id: generatorId }) });
export const addStackLabel = (id, label) => request(`/api/v1/stacks/${id}/labels`, { method: 'POST', body: JSON.stringify(label) });
export const removeStackLabel = (id, label) => request(`/api/v1/stacks/${id}/labels/${label}`, { method: 'DELETE' });
export const addStackAnnotation = (id, key, value) => request(`/api/v1/stacks/${id}/annotations`, { method: 'POST', body: JSON.stringify({ stack_id: id, key, value }) });
export const removeStackAnnotation = (id, key) => request(`/api/v1/stacks/${id}/annotations/${key}`, { method: 'DELETE' });
export const createDeployment = async (stackId, yaml, isDeletion = false) => {
  const checksum = await sha256(yaml);
  return request(`/api/v1/stacks/${stackId}/deployment-objects`, { method: 'POST', body: JSON.stringify({ yaml_content: yaml, yaml_checksum: checksum, is_deletion_marker: isDeletion, sequence_id: null }) });
};
export const getDeployment = (id) => request(`/api/v1/deployment-objects/${id}`);

// Templates
export const getTemplates = () => request('/api/v1/templates');
export const getTemplateLabels = (id) => request(`/api/v1/templates/${id}/labels`);
export const getTemplateAnnotations = (id) => request(`/api/v1/templates/${id}/annotations`);
export const createTemplate = (name, description, content, schema) => request('/api/v1/templates', { method: 'POST', body: JSON.stringify({ name, description: description || null, template_content: content, parameters_schema: schema }) });
export const updateTemplate = (id, description, content, schema) => request(`/api/v1/templates/${id}`, { method: 'PUT', body: JSON.stringify({ description: description || null, template_content: content, parameters_schema: schema }) });
export const deleteTemplate = (id) => request(`/api/v1/templates/${id}`, { method: 'DELETE' });
export const addTemplateLabel = (id, label) => request(`/api/v1/templates/${id}/labels`, { method: 'POST', body: JSON.stringify(label) });
export const removeTemplateLabel = (id, label) => request(`/api/v1/templates/${id}/labels/${label}`, { method: 'DELETE' });
export const instantiateTemplate = (stackId, templateId, params) => request(`/api/v1/stacks/${stackId}/deployment-objects/from-template`, { method: 'POST', body: JSON.stringify({ template_id: templateId, parameters: params }) });

// Generators
export const getGenerators = () => request('/api/v1/generators');
export const createGenerator = (name, description) => request('/api/v1/generators', { method: 'POST', body: JSON.stringify({ name, description: description || null }) });
export const rotateGeneratorPak = (id) => request(`/api/v1/generators/${id}/rotate-pak`, { method: 'POST' });

// Work Orders (Jobs)
export const getWorkOrders = (status, workType) => {
  const params = new URLSearchParams();
  if (status) params.append('status', status);
  if (workType) params.append('work_type', workType);
  const query = params.toString();
  return request(`/api/v1/work-orders${query ? '?' + query : ''}`);
};
export const getWorkOrder = (id) => request(`/api/v1/work-orders/${id}`);
export const createWorkOrder = (workType, yamlContent, targeting, options = {}) => request('/api/v1/work-orders', {
  method: 'POST',
  body: JSON.stringify({
    work_type: workType,
    yaml_content: yamlContent,
    targeting,
    max_retries: options.maxRetries,
    backoff_seconds: options.backoffSeconds,
    claim_timeout_seconds: options.claimTimeoutSeconds
  })
});
export const deleteWorkOrder = (id) => request(`/api/v1/work-orders/${id}`, { method: 'DELETE' });
export const getWorkOrderLog = (workType, success, agentId, limit) => {
  const params = new URLSearchParams();
  if (workType) params.append('work_type', workType);
  if (success !== undefined && success !== null) params.append('success', success);
  if (agentId) params.append('agent_id', agentId);
  if (limit) params.append('limit', limit);
  const query = params.toString();
  return request(`/api/v1/work-order-log${query ? '?' + query : ''}`);
};

// Diagnostics
export const createDiagnostic = (deploymentObjectId, agentId, requestedBy, retentionMinutes) =>
  request(`/api/v1/deployment-objects/${deploymentObjectId}/diagnostics`, {
    method: 'POST',
    body: JSON.stringify({ agent_id: agentId, requested_by: requestedBy, retention_minutes: retentionMinutes })
  });
export const getDiagnostic = (id) => request(`/api/v1/diagnostics/${id}`);

// Deployment Health
export const getDeploymentHealth = (id) => request(`/api/v1/deployment-objects/${id}/health`);
export const getStackHealth = (id) => request(`/api/v1/stacks/${id}/health`);

// Webhooks
export const getWebhooks = () => request('/api/v1/webhooks');
export const getWebhook = (id) => request(`/api/v1/webhooks/${id}`);
export const createWebhook = (name, url, eventTypes, authHeader, options = {}) => request('/api/v1/webhooks', {
  method: 'POST',
  body: JSON.stringify({
    name,
    url,
    event_types: eventTypes,
    auth_header: authHeader || null,
    enabled: options.enabled !== false,
    max_retries: options.maxRetries || 5,
    timeout_seconds: options.timeoutSeconds || 30
  })
});
export const updateWebhook = (id, updates) => request(`/api/v1/webhooks/${id}`, {
  method: 'PUT',
  body: JSON.stringify(updates)
});
export const deleteWebhook = (id) => request(`/api/v1/webhooks/${id}`, { method: 'DELETE' });
export const getWebhookEventTypes = () => request('/api/v1/webhooks/event-types');
export const getWebhookDeliveries = (id, status, limit) => {
  const params = new URLSearchParams();
  if (status) params.append('status', status);
  if (limit) params.append('limit', limit);
  const query = params.toString();
  return request(`/api/v1/webhooks/${id}/deliveries${query ? '?' + query : ''}`);
};

// Metrics
export const getMetrics = async () => {
  const res = await fetch(`${BASE_URL}/metrics`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.text();
};

// Webhook Catcher (for demo purposes)
// Port 8090 is exposed by docker-compose for localhost access
const WEBHOOK_CATCHER_URL = 'http://localhost:8090';

export const getWebhookCatcherStats = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

export const clearWebhookCatcher = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`, { method: 'DELETE' });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

// Shipwright Build YAML - uses same proven config as e2e tests
// Uses sample-go repo with ttl.sh for ephemeral image storage (no registry credentials needed)
export const getDemoBuildYaml = () => `
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: demo-build
  namespace: default
spec:
  source:
    type: Git
    git:
      url: https://github.com/shipwright-io/sample-go
    contextDir: docker-build
  strategy:
    name: kaniko
    kind: ClusterBuildStrategy
  output:
    image: ttl.sh/brokkr-demo-build:1h
`.trim();

// Create a build work order for the demo
// If agentId is provided, targets that specific agent (like e2e tests do)
export const createBuildWorkOrder = async (imageTag = 'latest', agentId = null) => {
  const buildYaml = getDemoBuildYaml();

  // Target specific agent if provided, otherwise any agent can claim it
  const targeting = agentId
    ? { agent_ids: [agentId] }
    : { labels: [] };

  return createWorkOrder('build', buildYaml, targeting, {
    maxRetries: 3,
    backoffSeconds: 30
  });
};

// Kubernetes Deployment YAML for the built webhook-catcher
export const getWebhookCatcherDeploymentYaml = (imageTag = 'latest') => `
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webhook-catcher
  namespace: default
  labels:
    app: webhook-catcher
spec:
  replicas: 1
  selector:
    matchLabels:
      app: webhook-catcher
  template:
    metadata:
      labels:
        app: webhook-catcher
    spec:
      containers:
      - name: webhook-catcher
        image: registry:5000/webhook-catcher:${imageTag}
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: webhook-catcher
  namespace: default
spec:
  selector:
    app: webhook-catcher
  ports:
  - port: 8080
    targetPort: 8080
`.trim();

// Parse Prometheus metrics text format into an object
export const parseMetrics = (metricsText) => {
  const metrics = {};
  const lines = metricsText.split('\n');

  for (const line of lines) {
    // Skip comments and empty lines
    if (line.startsWith('#') || !line.trim()) continue;

    // Parse metric line: metric_name{labels} value
    const match = line.match(/^([a-zA-Z_:][a-zA-Z0-9_:]*)(\{[^}]*\})?\s+(.+)$/);
    if (match) {
      const [, name, labelsStr, value] = match;
      const labels = {};

      if (labelsStr) {
        // Parse labels: {key="value",key2="value2"}
        const labelMatches = labelsStr.matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)="([^"]*)"/g);
        for (const [, key, val] of labelMatches) {
          labels[key] = val;
        }
      }

      if (!metrics[name]) metrics[name] = [];
      metrics[name].push({ labels, value: parseFloat(value) });
    }
  }

  return metrics;
};

// ============================================
// DEMO PANEL API FUNCTIONS
// ============================================

// Check environment prerequisites for demo
export const checkEnvironment = async () => {
  const result = {
    brokerHealthy: false,
    integrationAgent: null,
    shipwrightReady: false,
    buildStrategies: 0,
    errors: []
  };

  // Check broker health
  try {
    const res = await fetch(`${BASE_URL}/healthz`);
    result.brokerHealthy = res.ok;
  } catch (e) {
    result.errors.push('Broker API not reachable');
  }

  // Find integration agent
  try {
    const agents = await getAgents();
    result.integrationAgent = agents.find(a => a.name === 'brokkr-integration-test-agent');
    if (!result.integrationAgent) {
      result.errors.push('Integration test agent not found');
    }
  } catch (e) {
    result.errors.push('Failed to fetch agents: ' + e.message);
  }

  // Check for ClusterBuildStrategies (indicates Shipwright is ready)
  // We check this via work order creation attempt later, but for now we can assume ready
  // if the agent is present (Helm chart installs Shipwright)
  if (result.integrationAgent?.status === 'ACTIVE') {
    result.shipwrightReady = true;
    result.buildStrategies = 3; // Helm chart installs buildah, buildpacks, kaniko
  }

  return result;
};

// Get webhook catcher events
export const getWebhookCatcherEvents = async () => {
  try {
    const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`);
    if (!res.ok) return { count: 0, messages: [] };
    return res.json();
  } catch (e) {
    return { count: 0, messages: [] };
  }
};

// Poll for a condition with timeout
export const pollForCondition = async (checkFn, intervalMs = 2000, timeoutMs = 60000) => {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const result = await checkFn();
    if (result.done) {
      return result;
    }
    await new Promise(resolve => setTimeout(resolve, intervalMs));
  }

  throw new Error('Polling timed out');
};

// Poll agent status until ACTIVE with recent heartbeat
export const pollAgentStatus = async (agentId, timeoutMs = 120000) => {
  return pollForCondition(async () => {
    try {
      const agents = await getAgents();
      const agent = agents.find(a => a.id === agentId);
      if (!agent) return { done: false, agent: null };

      if (agent.status === 'ACTIVE' && agent.last_heartbeat) {
        const heartbeatAge = Date.now() - new Date(agent.last_heartbeat).getTime();
        if (heartbeatAge < 60000) { // Within last minute
          return { done: true, agent };
        }
      }
      return { done: false, agent };
    } catch (e) {
      return { done: false, agent: null, error: e.message };
    }
  }, 3000, timeoutMs);
};

// Poll work order until completed or failed
export const pollWorkOrderStatus = async (workOrderId, timeoutMs = 300000) => {
  return pollForCondition(async () => {
    try {
      const wo = await getWorkOrder(workOrderId);
      const terminalStates = ['COMPLETED', 'FAILED', 'CANCELLED'];
      if (terminalStates.includes(wo.status)) {
        return { done: true, workOrder: wo };
      }
      return { done: false, workOrder: wo };
    } catch (e) {
      return { done: false, workOrder: null, error: e.message };
    }
  }, 3000, timeoutMs);
};

// Delete a stack and all its resources
export const deleteStack = (id) => request(`/api/v1/stacks/${id}`, { method: 'DELETE' });

// Delete an agent
export const deleteAgent = (id) => request(`/api/v1/agents/${id}`, { method: 'DELETE' });

// Delete a generator
export const deleteGenerator = (id) => request(`/api/v1/generators/${id}`, { method: 'DELETE' });

// Cleanup demo resources
export const cleanupDemo = async (resources, onProgress) => {
  const log = (step, status) => onProgress?.(step, status);
  const errors = [];

  // 1. Remove agent targets first
  if (resources.agentTargets?.length) {
    log('Removing agent targets...', 'active');
    for (const target of resources.agentTargets) {
      try {
        await removeAgentTarget(target.agentId, target.stackId);
      } catch (e) {
        errors.push(`Failed to remove target: ${e.message}`);
      }
    }
    log('Agent targets removed', 'done');
  }

  // 2. Delete webhook subscriptions
  if (resources.webhookIds?.length) {
    log('Removing webhook subscriptions...', 'active');
    for (const id of resources.webhookIds) {
      try {
        await deleteWebhook(id);
      } catch (e) {
        errors.push(`Failed to delete webhook ${id}: ${e.message}`);
      }
    }
    log('Webhooks removed', 'done');
  }

  // 3. Create deletion markers for deployments (triggers K8s cleanup)
  if (resources.deploymentIds?.length) {
    log('Creating deletion markers...', 'active');
    for (const deployment of resources.deploymentIds) {
      try {
        // Create a deletion marker deployment object
        await createDeployment(deployment.stackId, deployment.yaml, true);
      } catch (e) {
        errors.push(`Failed to create deletion marker: ${e.message}`);
      }
    }
    log('Deletion markers created', 'done');
  }

  // 4. Delete stacks
  if (resources.stackIds?.length) {
    log('Deleting stacks...', 'active');
    for (const id of resources.stackIds) {
      try {
        await deleteStack(id);
      } catch (e) {
        errors.push(`Failed to delete stack ${id}: ${e.message}`);
      }
    }
    log('Stacks deleted', 'done');
  }

  // 5. Delete templates
  if (resources.templateIds?.length) {
    log('Deleting templates...', 'active');
    for (const id of resources.templateIds) {
      try {
        await deleteTemplate(id);
      } catch (e) {
        errors.push(`Failed to delete template ${id}: ${e.message}`);
      }
    }
    log('Templates deleted', 'done');
  }

  // 6. Deactivate demo agents (don't delete - keep for audit)
  if (resources.agentIds?.length) {
    log('Deactivating demo agents...', 'active');
    for (const id of resources.agentIds) {
      try {
        await updateAgent(id, { status: 'DEACTIVATED' });
      } catch (e) {
        errors.push(`Failed to deactivate agent ${id}: ${e.message}`);
      }
    }
    log('Demo agents deactivated', 'done');
  }

  // 7. Clear webhook catcher
  log('Clearing webhook catcher...', 'active');
  try {
    await clearWebhookCatcher();
  } catch (e) {
    // Not critical
  }
  log('Webhook catcher cleared', 'done');

  return { success: errors.length === 0, errors };
};
