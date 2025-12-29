const BASE_URL = process.env.REACT_APP_BROKER_URL || 'http://localhost:3000';
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
