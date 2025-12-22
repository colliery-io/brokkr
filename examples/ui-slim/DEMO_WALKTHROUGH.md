# Brokkr UI Demo Walkthrough

This guide walks through common Brokkr workflows using the slim UI. It demonstrates the core capabilities of the Brokkr deployment orchestration system.

**Prerequisites:**
- Brokkr stack running (`angreal local up`)
- UI accessible at http://localhost:3001
- Broker API at http://localhost:3000

---

## Part 1: Agent Management

### 1.1 View Registered Agents

Navigate to the **Agents** tab. You should see agents that have registered with the broker.

**What you'll see:**
- Agent name and cluster
- Current status (ACTIVE/INACTIVE)
- Labels and target counts
- Last heartbeat timestamp

### 1.2 Activate an Agent (Happy Path)

New agents start as `INACTIVE` for safety - they won't process deployments until explicitly activated.

1. Click on an agent row to open the detail modal
2. Note the status shows `INACTIVE` with a red indicator
3. Click the green **Activate** button
4. Status changes to `ACTIVE` with a green indicator

**Why this matters:** This prevents accidental deployments to newly registered agents. Operators must explicitly approve agents before they receive workloads.

### 1.3 Add Labels to an Agent

Labels are simple tags that enable **automatic stack matching** - agents receive deployments from stacks with matching labels.

1. In the agent modal, find the **Labels** section
2. Type `development` and press Enter or click +
3. Add more labels: `us-west-2`, `frontend`

**How matching works:**
- Agent has label `production`
- Stack has label `production`
- → Agent automatically receives that stack's deployments

**Common labels:**
- `dev`, `staging`, `production`
- `us-west-2`, `eu-central-1`, `ap-southeast-1`
- `frontend`, `backend`, `database`
- `team-platform`, `team-payments`

### 1.4 Add Annotations

Annotations are key-value pairs for richer metadata. They also enable stack matching.

1. In the **Annotations** section
2. Add key: `owner`, value: `platform-team`
3. Add key: `cost-center`, value: `eng-123`

**Note:** Both key and value must have no whitespace (use hyphens).

---

## Part 2: Stack Creation & Deployment

### 2.1 Create a Generator (Admin Tab)

Generators are service accounts for programmatic stack management.

1. Go to **Admin** tab
2. Click **+ Create Generator PAK**
3. Enter name: `ci-pipeline`
4. Enter description: `CI/CD pipeline automation`
5. Click **Create**
6. **IMPORTANT:** Copy the PAK immediately - it won't be shown again!

### 2.2 Create a Stack (Happy Path)

Stacks are logical groupings of Kubernetes resources.

1. Go to **Stacks** tab
2. Click **+ New Stack**
3. Fill in:
   - Name: `demo-application`
   - Description: `Demo application stack for walkthrough`
   - Generator: Select your generator
4. Click **Create**

### 2.3 Deploy the Application Stack (Happy Path)

Click on your new stack, then click **+ Deploy** and paste this complete multi-document YAML:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: demo-app
  labels:
    app: demo
    managed-by: brokkr
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: demo-config
  namespace: demo-app
data:
  APP_ENV: "development"
  LOG_LEVEL: "debug"
  API_ENDPOINT: "https://api.example.com"
  FEATURE_FLAGS: |
    {
      "new_dashboard": true,
      "dark_mode": true,
      "beta_features": false
    }
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: demo-web
  namespace: demo-app
  labels:
    app: demo-web
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
        env:
        - name: APP_ENV
          valueFrom:
            configMapKeyRef:
              name: demo-config
              key: APP_ENV
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "200m"
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 15
          periodSeconds: 20
---
apiVersion: v1
kind: Service
metadata:
  name: demo-web-svc
  namespace: demo-app
spec:
  selector:
    app: demo-web
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
```

This single deployment object contains all resources for the application: namespace, config, deployment, and service. The agent applies them in order.

### 2.4 Connect Agents to Stacks

There are **two ways** for agents to receive deployment objects from a stack:

#### Option A: Label Matching (Recommended for Fleet Deployments)

Agents automatically receive deployments from stacks with matching labels. No explicit targeting needed.

1. Go to **Stacks** tab, click on `demo-application`
2. Add a label: `development`
3. Go to **Agents** tab, click on your agent
4. Add the same label: `development`
5. The agent now automatically receives deployments from any stack with label `development`

**Why this matters:** Deploy to 100 agents by labeling one stack, not by creating 100 explicit targets.

#### Option B: Explicit Targeting (For Specific One-Off Bindings)

Directly bind an agent to a stack:

1. Go to **Agents** tab
2. Click on your active agent
3. In **Stack Targets** section, select `demo-application` from dropdown
4. This agent now explicitly receives deployments from this stack

**When to use explicit targets:** Testing a deployment on one agent before rolling out via labels.

---

## Part 3: Monitoring & Diagnostics

### 3.1 View Deployment Health

After targeting, the agent will apply resources and report health.

1. Go to **Stacks** tab
2. Click on `demo-application`
3. Look for the health indicator next to "Deployment Objects"
4. Each deployment shows:
   - Overall status (healthy/degraded/failing)
   - Agent counts: `2✓` healthy, `0⚠` degraded, `0✕` failing

### 3.2 Run On-Demand Diagnostics (Happy Path)

Get detailed diagnostic information for a deployment:

1. In the stack modal, find a deployment object
2. Click the 🔍 (magnifying glass) button
3. Select an agent from the dropdown
4. Wait for diagnostics to complete
5. View:
   - **Pod Statuses:** Running state of all pods
   - **Events:** Recent Kubernetes events
   - **Logs:** Tail of container logs

---

## Part 4: Templates

### 4.1 Create a Reusable Template

Templates enable parameterized deployments.

1. Go to **Templates** tab
2. Click **+ New Template**
3. Fill in:
   - Name: `microservice`
   - Description: `Standard microservice deployment template`

**Template Content (Tera/Jinja2 syntax):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ name }}
  namespace: {{ namespace | default(value="default") }}
  labels:
    app: {{ name }}
    version: {{ version | default(value="v1") }}
spec:
  replicas: {{ replicas | default(value=2) }}
  selector:
    matchLabels:
      app: {{ name }}
  template:
    metadata:
      labels:
        app: {{ name }}
        version: {{ version | default(value="v1") }}
    spec:
      containers:
      - name: {{ name }}
        image: {{ image }}
        ports:
        - containerPort: {{ port | default(value=8080) }}
{%- if env | length > 0 %}
        env:
{%- for key, value in env %}
        - name: {{ key }}
          value: "{{ value }}"
{%- endfor %}
{%- endif %}
        resources:
          requests:
            memory: {{ memory_request | default(value="128Mi") }}
            cpu: {{ cpu_request | default(value="100m") }}
          limits:
            memory: {{ memory_limit | default(value="256Mi") }}
            cpu: {{ cpu_limit | default(value="500m") }}
---
apiVersion: v1
kind: Service
metadata:
  name: {{ name }}-svc
  namespace: {{ namespace | default(value="default") }}
spec:
  selector:
    app: {{ name }}
  ports:
  - port: {{ service_port | default(value=80) }}
    targetPort: {{ port | default(value=8080) }}
  type: {{ service_type | default(value="ClusterIP") }}
```

**Parameters Schema (JSON):**

```json
{
  "type": "object",
  "required": ["name", "image"],
  "properties": {
    "name": {
      "type": "string",
      "description": "Service name"
    },
    "namespace": {
      "type": "string",
      "default": "default"
    },
    "image": {
      "type": "string",
      "description": "Container image"
    },
    "replicas": {
      "type": "integer",
      "default": 2
    },
    "port": {
      "type": "integer",
      "default": 8080
    },
    "version": {
      "type": "string",
      "default": "v1"
    },
    "env": {
      "type": "object",
      "default": {}
    }
  }
}
```

### 4.2 Instantiate a Template (Happy Path)

1. Click on your `microservice` template
2. Click **Instantiate**
3. Select target stack: `demo-application`
4. Enter parameters:

```json
{
  "name": "api-gateway",
  "namespace": "demo-app",
  "image": "nginx:alpine",
  "replicas": 3,
  "port": 8080,
  "version": "v2",
  "env": {
    "LOG_LEVEL": "info",
    "TIMEOUT": "30s"
  }
}
```

5. Click **Instantiate** - the rendered YAML is added to the stack

---

## Part 5: Work Orders (Jobs)

Work orders distribute one-time tasks to agents.

### 5.1 Create a Work Order (Happy Path)

1. Go to **Jobs** tab
2. Click **+ New Job**
3. Fill in:
   - Work Type: `custom`
   - Target Agents: Select your agent(s)
   - Max Retries: 3
   - Backoff: 60 seconds

**YAML Content:**

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-migration-001
  namespace: demo-app
spec:
  ttlSecondsAfterFinished: 300
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: migrate
        image: alpine:latest
        command: ["sh", "-c", "echo 'Running migration...' && sleep 5 && echo 'Migration complete!'"]
```

4. Click **Create**
5. Watch the work order in **Active Work Orders**
6. Once complete, it moves to **History** with success/failure status

### 5.2 Target by Labels

Instead of selecting specific agents, target by labels:

1. Leave **Target Agents** empty
2. In **Target Labels**, enter: `production, us-west-2`
3. Only agents with ALL these labels will receive the work order

---

## Part 6: Sad Paths & Error Handling

### 6.1 Invalid YAML (Sad Path)

Try deploying invalid YAML to see error handling:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: broken-config
  namespace: demo-app
data:
  key: value
  nested:    # Invalid - can't have nested objects in ConfigMap data
    foo: bar
```

**Expected:** The deployment object is created, but the agent will report an error when trying to apply it.

### 6.2 Missing Namespace (Sad Path)

Deploy to a non-existent namespace:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: orphan-config
  namespace: does-not-exist
data:
  key: value
```

**Expected:** Agent reports failure - namespace doesn't exist.

### 6.3 Invalid Image (Sad Path)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bad-image
  namespace: demo-app
spec:
  replicas: 1
  selector:
    matchLabels:
      app: bad-image
  template:
    metadata:
      labels:
        app: bad-image
    spec:
      containers:
      - name: app
        image: this-image-does-not-exist:v999
```

**Expected:**
- Deployment is created
- Pod enters `ImagePullBackOff` state
- Health status shows `degraded` or `failing`
- Diagnostics reveal the image pull error

### 6.4 Resource Quota Exceeded (Sad Path)

First, create a ResourceQuota:

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: demo-quota
  namespace: demo-app
spec:
  hard:
    pods: "5"
    requests.cpu: "500m"
    requests.memory: "512Mi"
```

Then try to exceed it:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: resource-hog
  namespace: demo-app
spec:
  replicas: 10
  selector:
    matchLabels:
      app: resource-hog
  template:
    metadata:
      labels:
        app: resource-hog
    spec:
      containers:
      - name: app
        image: nginx:alpine
        resources:
          requests:
            memory: "256Mi"
            cpu: "200m"
```

**Expected:** Deployment partially fails due to quota constraints.

### 6.5 Deactivating an Agent Mid-Deployment

1. Start a large deployment
2. Immediately go to **Agents** and click **Deactivate**

**Expected:**
- Agent stops processing new deployment objects
- Existing resources remain in cluster
- No new work orders are claimed

---

## Part 7: Cleanup

### 7.1 Delete Resources (Deletion Marker)

To remove resources, deploy with the **Mark as deletion** checkbox:

1. Go to your stack
2. Click **+ Deploy**
3. Paste the resource YAML
4. Check **Mark as deletion**
5. Click **Deploy**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: demo-web
  namespace: demo-app
```

**Note:** Only include the minimum identifying fields (apiVersion, kind, metadata.name, metadata.namespace).

### 7.2 Full Namespace Cleanup

Deploy as deletion marker:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: demo-app
```

**Warning:** This deletes the entire namespace and all resources within it!

---

## Summary

This walkthrough covered:

| Feature | Happy Path | Sad Path |
|---------|------------|----------|
| Agent Management | Activate, add labels/targets | Deactivate mid-deployment |
| Stacks | Create, deploy YAML | Invalid YAML, missing namespace |
| Deployments | Multi-resource deployment | Bad images, quota exceeded |
| Health Monitoring | View status, run diagnostics | Observe failure states |
| Templates | Create, instantiate | Invalid parameters |
| Work Orders | Create jobs, target by labels | Job failures, retries |
| Cleanup | Deletion markers | Cascading deletes |

For programmatic access, use the PAKs created in the Admin panel with the Broker API at `http://localhost:3000/api/v1/`.
