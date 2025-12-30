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

// Metrics
export const getMetrics = async () => {
  const res = await fetch(`${BASE_URL}/metrics`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.text();
};

// Webhook Catcher (for demo purposes)
const WEBHOOK_CATCHER_URL = 'http://localhost:8888';

export const getWebhookCatcherStats = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/stats`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

export const clearWebhookCatcher = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/clear`, { method: 'POST' });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

// Generate K8s manifests for deploying a Brokkr agent
export const getAgentDeploymentYaml = (agentName, clusterName, pak, labels = {}, annotations = {}) => {
  const labelStr = Object.entries(labels).map(([k, v]) => `${k}: "${v}"`).join('\n        ');
  const annotationStr = Object.entries(annotations).map(([k, v]) => `${k}: "${v}"`).join('\n        ');

  return `---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: brokkr-agent-${agentName}
  namespace: default
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: brokkr-agent-${agentName}
rules:
- apiGroups: [""]
  resources: [pods, pods/log, pods/status, namespaces, nodes, services, endpoints, configmaps, persistentvolumes, persistentvolumeclaims, events]
  verbs: [get, list, watch]
- apiGroups: ["apps"]
  resources: [deployments, deployments/status, statefulsets, daemonsets, replicasets]
  verbs: [get, list, watch, create, update, patch, delete]
- apiGroups: ["batch"]
  resources: [jobs, cronjobs]
  verbs: [get, list, watch]
- apiGroups: ["networking.k8s.io"]
  resources: [ingresses, networkpolicies]
  verbs: [get, list, watch]
- apiGroups: ["shipwright.io"]
  resources: [builds, buildruns, buildstrategies, clusterbuildstrategies]
  verbs: [get, list, watch, create, update, patch, delete]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: brokkr-agent-${agentName}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: brokkr-agent-${agentName}
subjects:
- kind: ServiceAccount
  name: brokkr-agent-${agentName}
  namespace: default
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: brokkr-agent-${agentName}-config
  namespace: default
data:
  BROKKR__AGENT__BROKER_URL: "http://brokkr-broker:3000"
  BROKKR__AGENT__AGENT_NAME: "${agentName}"
  BROKKR__AGENT__CLUSTER_NAME: "${clusterName}"
  BROKKR__AGENT__PAK: "${pak}"
  BROKKR__AGENT__POLLING_INTERVAL: "10"
  BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED: "true"
  BROKKR__AGENT__DEPLOYMENT_HEALTH_INTERVAL: "30"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brokkr-agent-${agentName}
  namespace: default
  labels:
    app: brokkr-agent
    agent: ${agentName}
    ${labelStr}
  annotations:
    ${annotationStr}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: brokkr-agent
      agent: ${agentName}
  template:
    metadata:
      labels:
        app: brokkr-agent
        agent: ${agentName}
    spec:
      serviceAccountName: brokkr-agent-${agentName}
      containers:
      - name: agent
        image: ghcr.io/colliery-io/brokkr-agent:latest
        imagePullPolicy: Always
        args: ["start"]
        envFrom:
        - configMapRef:
            name: brokkr-agent-${agentName}-config
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        resources:
          requests:
            memory: "128Mi"
            cpu: "50m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        volumeMounts:
        - name: tmp
          mountPath: /tmp
      volumes:
      - name: tmp
        emptyDir: {}
`.trim();
};

// Demo/Init Script - demonstrates REAL functionality with the deployed agent
export const runDemoSetup = async (onProgress) => {
  const log = (msg) => onProgress?.({ message: msg, done: false });
  const results = {
    generators: [],
    agents: [],
    stacks: [],
    deployments: [],
    templates: [],
    webhooks: [],
    realAgent: null,
    stagingAgent: null
  };

  try {
    // 1. Find and activate the real deployed agent
    log('Finding real deployed agent...');
    const agents = await getAgents();
    const realAgent = agents.find(a => a.name === 'brokkr-integration-test-agent');

    if (!realAgent) {
      throw new Error('No real agent found. Make sure brokkr-agent is running.');
    }

    results.realAgent = realAgent;
    log(`Found agent: ${realAgent.name} (status: ${realAgent.status})`);

    if (realAgent.status !== 'ACTIVE') {
      log('Activating the real agent...');
      await updateAgent(realAgent.id, { status: 'ACTIVE' });
      log('Agent activated!');
    }

    // Add labels to the real agent
    log('Adding labels to real agent...');
    await addAgentLabel(realAgent.id, 'env:integration').catch(() => {});
    await addAgentLabel(realAgent.id, 'tier:primary').catch(() => {});
    await addAgentAnnotation(realAgent.id, 'role', 'primary-deployer').catch(() => {});
    log('Labels added to integration agent');

    // 2. Create a generator for our stacks
    log('Creating generator...');
    const generator = await createGenerator('demo-pipeline', 'Demo CI/CD pipeline');
    results.generators.push(generator.generator);
    log('Generator created');

    // 3. Create webhook FIRST to capture all subsequent events
    log('Creating webhook to capture events...');
    const webhook = await createWebhook(
      'demo-event-notifier',
      'http://webhook-catcher:8080/receive',
      ['agent.created', 'agent.updated', 'stack.created', 'deployment_object.created', 'work_order.completed'],
      null,
      { maxRetries: 3, timeoutSeconds: 10 }
    );
    results.webhooks.push(webhook);
    log('Webhook ready - will capture all events');

    // 4. Create a stack for deploying a second agent
    log('Creating stack for staging agent deployment...');
    const agentStack = await createStack('staging-agent-stack', 'Deploys a staging environment agent', generator.generator.id);
    results.stacks.push(agentStack);
    log('Agent stack created');

    // 5. Create the staging agent in the broker (to get PAK)
    log('Registering staging agent with broker...');
    const stagingAgentResult = await createAgent('demo-staging-agent', 'brokkr-dev-integration-cluster');
    results.stagingAgent = stagingAgentResult;
    results.agents.push(stagingAgentResult.agent);
    log(`Staging agent registered (PAK received)`);

    // 6. Deploy the staging agent via the real agent
    log('Creating deployment for staging agent...');
    const agentYaml = getAgentDeploymentYaml(
      'demo-staging-agent',
      'brokkr-dev-integration-cluster',
      stagingAgentResult.initial_pak,
      { env: 'staging', tier: 'secondary' },
      { role: 'staging-deployer', 'deployed-by': 'demo-setup' }
    );
    const agentDeployment = await createDeployment(agentStack.id, agentYaml);
    results.deployments.push(agentDeployment);
    log('Staging agent deployment created');

    // 7. Assign the stack to the real agent so it deploys the staging agent
    log('Targeting stack to real agent...');
    await addAgentTarget(realAgent.id, agentStack.id);
    log('Stack targeted - agent will deploy staging agent');

    // 8. Create a stack for demo services
    log('Creating demo services stack...');
    const servicesStack = await createStack('demo-services', 'Demo application services', generator.generator.id);
    results.stacks.push(servicesStack);

    // Add labels
    await addStackLabel(servicesStack.id, 'app:demo');
    await addStackLabel(servicesStack.id, 'team:platform');

    // 9. Deploy demo services (triggers webhooks)
    log('Deploying demo services...');
    const servicesYaml = `---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: demo-web
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      app: demo-web
  template:
    metadata:
      labels:
        app: demo-web
    spec:
      containers:
      - name: web
        image: nginx:alpine
        ports:
        - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: demo-web-svc
  namespace: default
spec:
  selector:
    app: demo-web
  ports:
  - port: 80
    targetPort: 80
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: demo-api
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      app: demo-api
  template:
    metadata:
      labels:
        app: demo-api
    spec:
      containers:
      - name: api
        image: hashicorp/http-echo
        args: ["-text=Hello from Brokkr Demo"]
        ports:
        - containerPort: 5678
---
apiVersion: v1
kind: Service
metadata:
  name: demo-api-svc
  namespace: default
spec:
  selector:
    app: demo-api
  ports:
  - port: 80
    targetPort: 5678`;

    const servicesDeployment = await createDeployment(servicesStack.id, servicesYaml);
    results.deployments.push(servicesDeployment);
    log('Demo services deployment created');

    // Target services stack to real agent
    await addAgentTarget(realAgent.id, servicesStack.id);
    log('Services stack targeted to agent');

    // 10. Create a template for reusable configs
    log('Creating reusable template...');
    const templateContent = `apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}
  namespace: {{ namespace | default(value="default") }}
data:
  config.json: |
    {
      "environment": "{{ environment }}",
      "version": "{{ version | default(value="1.0.0") }}"
    }`;

    const templateSchema = JSON.stringify({
      type: 'object',
      required: ['name', 'environment'],
      properties: {
        name: { type: 'string' },
        namespace: { type: 'string', default: 'default' },
        environment: { type: 'string', enum: ['dev', 'staging', 'prod'] },
        version: { type: 'string', default: '1.0.0' }
      }
    });

    const template = await createTemplate('app-config-template', 'Application configuration template', templateContent, templateSchema);
    results.templates.push(template);
    log('Template created');

    // Wait for agent to process deployments and webhooks to deliver
    log('Waiting for agent to process deployments...');
    await new Promise(resolve => setTimeout(resolve, 5000));

    // 11. Fetch webhook events
    log('Fetching received webhook events...');
    try {
      const catcherStats = await getWebhookCatcherStats();
      results.receivedWebhooks = catcherStats.messages || [];
      log(`Webhook catcher received ${catcherStats.count} events!`);
    } catch (e) {
      log('Could not fetch webhook stats: ' + e.message);
      results.receivedWebhooks = [];
    }

    onProgress?.({ message: 'Demo setup complete! Check Agents tab to see both agents.', done: true, results });
    return results;
  } catch (error) {
    onProgress?.({ message: `Error: ${error.message}`, done: true, error });
    throw error;
  }
};

// Shipwright Build YAML for webhook-catcher from the Brokkr repository
export const getWebhookCatcherBuildYaml = (imageTag = 'latest', branch = 'feature/event-webhooks') => `
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: webhook-catcher-build
  namespace: default
spec:
  source:
    type: Git
    git:
      url: https://github.com/colliery-io/brokkr
      revision: ${branch}
    contextDir: tools/webhook-catcher
  strategy:
    name: buildah
    kind: ClusterBuildStrategy
  output:
    image: registry:5000/webhook-catcher:${imageTag}
`.trim();

// Create a build work order for the webhook-catcher
export const createBuildWorkOrder = async (imageTag = 'latest') => {
  const buildYaml = getWebhookCatcherBuildYaml(imageTag);

  // Target all agents (they'll compete to claim it)
  const targeting = {
    labels: []  // Empty labels means any agent can claim it
  };

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
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
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
