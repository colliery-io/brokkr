import React, { useState, useEffect, useCallback } from 'react';
import * as api from './api';
import './styles.css';

// Reusable tag/chip component
const Tag = ({ children, onRemove, variant = 'default' }) => (
  <span className={`tag tag-${variant}`}>
    {children}
    {onRemove && <button onClick={onRemove} className="tag-remove">×</button>}
  </span>
);

// Collapsible section
const Section = ({ title, icon, children, defaultOpen = false, count }) => {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className={`section ${open ? 'open' : ''}`}>
      <button className="section-header" onClick={() => setOpen(!open)}>
        <span className="section-icon">{icon}</span>
        <span className="section-title">{title}</span>
        {count !== undefined && <span className="section-count">{count}</span>}
        <span className="section-arrow">{open ? '▼' : '▶'}</span>
      </button>
      {open && <div className="section-content">{children}</div>}
    </div>
  );
};

// Inline form for adding items
const InlineAdd = ({ placeholder, onAdd, fields = 1 }) => {
  const [values, setValues] = useState(fields === 1 ? '' : { key: '', value: '' });
  const handleSubmit = (e) => {
    e.preventDefault();
    if (fields === 1 && values.trim()) {
      onAdd(values.trim());
      setValues('');
    } else if (fields === 2 && values.key.trim() && values.value.trim()) {
      onAdd(values.key.trim(), values.value.trim());
      setValues({ key: '', value: '' });
    }
  };
  return (
    <form onSubmit={handleSubmit} className="inline-add">
      {fields === 1 ? (
        <input value={values} onChange={(e) => setValues(e.target.value)} placeholder={placeholder} />
      ) : (
        <>
          <input value={values.key} onChange={(e) => setValues({ ...values, key: e.target.value })} placeholder="key" />
          <input value={values.value} onChange={(e) => setValues({ ...values, value: e.target.value })} placeholder="value" />
        </>
      )}
      <button type="submit">+</button>
    </form>
  );
};

// Status indicator
const Status = ({ status }) => {
  const s = (status || '').toLowerCase();
  const color = s === 'active' || s === 'success' || s === 'healthy' ? 'green' :
                s === 'inactive' || s === 'failure' || s === 'failed' ? 'red' :
                s === 'pending' || s === 'in_progress' ? 'yellow' : 'gray';
  return <span className={`status status-${color}`}>{status}</span>;
};

// Modal component
const Modal = ({ title, onClose, children }) => (
  <div className="modal-overlay" onClick={onClose}>
    <div className="modal" onClick={(e) => e.stopPropagation()}>
      <div className="modal-header">
        <h3>{title}</h3>
        <button onClick={onClose} className="modal-close">×</button>
      </div>
      <div className="modal-body">{children}</div>
    </div>
  </div>
);

// Toast notification
const Toast = ({ message, type = 'info', onClose }) => (
  <div className={`toast toast-${type}`}>
    {message}
    <button onClick={onClose}>×</button>
  </div>
);

// ==================== AGENTS PANEL ====================
const AgentsPanel = ({ stacks, onRefresh }) => {
  const [agents, setAgents] = useState([]);
  const [details, setDetails] = useState({});
  const [selected, setSelected] = useState(null);
  const [events, setEvents] = useState([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.getAgents();
      setAgents(data);
      onRefresh?.(data);
      const d = {};
      await Promise.all(data.map(async (a) => {
        const [labels, annotations, targets] = await Promise.all([
          api.getAgentLabels(a.id), api.getAgentAnnotations(a.id), api.getAgentTargets(a.id)
        ]);
        d[a.id] = { labels, annotations, targets };
      }));
      setDetails(d);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, [onRefresh]);

  useEffect(() => { load(); }, [load]);

  const selectAgent = async (agent) => {
    setSelected(agent);
    const evts = await api.getAgentEvents(agent.id);
    setEvents(evts);
  };

  const addLabel = async (label) => {
    await api.addAgentLabel(selected.id, label);
    const labels = await api.getAgentLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  const removeLabel = async (label) => {
    await api.removeAgentLabel(selected.id, label);
    const labels = await api.getAgentLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  const addAnnotation = async (key, value) => {
    await api.addAgentAnnotation(selected.id, key, value);
    const annotations = await api.getAgentAnnotations(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
  };

  const removeAnnotation = async (key) => {
    await api.removeAgentAnnotation(selected.id, key);
    const annotations = await api.getAgentAnnotations(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
  };

  const addTarget = async (stackId) => {
    await api.addAgentTarget(selected.id, stackId);
    const targets = await api.getAgentTargets(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], targets } });
  };

  const removeTarget = async (stackId) => {
    await api.removeAgentTarget(selected.id, stackId);
    const targets = await api.getAgentTargets(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], targets } });
  };

  const toggleStatus = async () => {
    const newStatus = selected.status === 'ACTIVE' ? 'INACTIVE' : 'ACTIVE';
    const updated = await api.updateAgent(selected.id, { status: newStatus });
    setSelected(updated);
    setAgents(agents.map(a => a.id === updated.id ? updated : a));
    onRefresh?.(agents.map(a => a.id === updated.id ? updated : a));
  };

  if (loading) return <div className="loading">Loading agents...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>⬡ Agents</h2>
        <button onClick={load} className="btn-icon">↻</button>
      </div>

      {agents.length === 0 ? (
        <div className="empty">No agents registered</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Cluster</th>
                <th>Status</th>
                <th>Labels</th>
                <th>Targets</th>
                <th>Last Seen</th>
              </tr>
            </thead>
            <tbody>
              {agents.map((a) => (
                <tr key={a.id} onClick={() => selectAgent(a)} className="clickable">
                  <td className="mono">{a.name}</td>
                  <td className="mono dim">{a.cluster_name}</td>
                  <td><Status status={a.status} /></td>
                  <td>
                    {details[a.id]?.labels?.map((l) => (
                      <Tag key={l.id} variant="label">{l.label}</Tag>
                    ))}
                  </td>
                  <td>{details[a.id]?.targets?.length || 0}</td>
                  <td className="dim">{a.last_heartbeat ? new Date(a.last_heartbeat).toLocaleString() : 'Never'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selected && (
        <Modal title={`Agent: ${selected.name}`} onClose={() => setSelected(null)}>
          <div className="detail-grid">
            <div className="detail-row">
              <span className="detail-label">ID</span>
              <span className="mono">{selected.id}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Cluster</span>
              <span className="mono">{selected.cluster_name}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Status</span>
              <Status status={selected.status} />
              <button onClick={toggleStatus} className={`btn-toggle ${selected.status === 'ACTIVE' ? 'active' : ''}`}>
                {selected.status === 'ACTIVE' ? 'Deactivate' : 'Activate'}
              </button>
            </div>
          </div>

          <div className="detail-section">
            <h4>Labels</h4>
            <div className="tags">
              {details[selected.id]?.labels?.map((l) => (
                <Tag key={l.id} onRemove={() => removeLabel(l.label)}>{l.label}</Tag>
              ))}
            </div>
            <InlineAdd placeholder="Add label..." onAdd={addLabel} />
          </div>

          <div className="detail-section">
            <h4>Annotations</h4>
            <div className="tags">
              {details[selected.id]?.annotations?.map((a) => (
                <Tag key={a.key} onRemove={() => removeAnnotation(a.key)}>{a.key}={a.value}</Tag>
              ))}
            </div>
            <InlineAdd placeholder="Add annotation..." onAdd={addAnnotation} fields={2} />
          </div>

          <div className="detail-section">
            <h4>Stack Targets</h4>
            <div className="tags">
              {details[selected.id]?.targets?.map((t) => {
                const stack = stacks.find((s) => s.id === t.stack_id);
                return <Tag key={t.stack_id} onRemove={() => removeTarget(t.stack_id)} variant="target">{stack?.name || t.stack_id}</Tag>;
              })}
            </div>
            <select onChange={(e) => e.target.value && addTarget(e.target.value)} value="">
              <option value="">+ Add target...</option>
              {stacks.filter((s) => !details[selected.id]?.targets?.some((t) => t.stack_id === s.id)).map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </div>

          <div className="detail-section">
            <h4>Recent Events</h4>
            {events.length === 0 ? (
              <div className="empty-small">No events</div>
            ) : (
              <div className="events-list">
                {events.slice(0, 10).map((e, i) => (
                  <div key={i} className="event-row">
                    <span className="dim">{new Date(e.created_at).toLocaleTimeString()}</span>
                    <span className="mono">{e.event_type}</span>
                    <Status status={e.status} />
                  </div>
                ))}
              </div>
            )}
          </div>
        </Modal>
      )}
    </div>
  );
};

// ==================== STACKS PANEL ====================
const StacksPanel = ({ generators, agents, onRefresh }) => {
  const [stacks, setStacks] = useState([]);
  const [details, setDetails] = useState({});
  const [selected, setSelected] = useState(null);
  const [deployments, setDeployments] = useState([]);
  const [showCreate, setShowCreate] = useState(false);
  const [showDeploy, setShowDeploy] = useState(false);
  const [showDiagnostic, setShowDiagnostic] = useState(null);
  const [diagnosticResult, setDiagnosticResult] = useState(null);
  const [stackHealth, setStackHealth] = useState(null);
  const [newStack, setNewStack] = useState({ name: '', description: '', generatorId: '' });
  const [yaml, setYaml] = useState('');
  const [isDeletion, setIsDeletion] = useState(false);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.getStacks();
      setStacks(data);
      const d = {};
      await Promise.all(data.map(async (s) => {
        const [labels, annotations] = await Promise.all([api.getStackLabels(s.id), api.getStackAnnotations(s.id)]);
        d[s.id] = { labels, annotations };
      }));
      setDetails(d);
      onRefresh?.(data);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, [onRefresh]);

  useEffect(() => { load(); }, [load]);

  const selectStack = async (stack) => {
    setSelected(stack);
    setStackHealth(null);
    const [deps, health] = await Promise.all([
      api.getStackDeployments(stack.id),
      api.getStackHealth(stack.id).catch(() => null)
    ]);
    setDeployments(deps);
    setStackHealth(health);
  };

  const create = async (e) => {
    e.preventDefault();
    await api.createStack(newStack.name, newStack.description, newStack.generatorId);
    setShowCreate(false);
    setNewStack({ name: '', description: '', generatorId: '' });
    load();
  };

  const deploy = async (e) => {
    e.preventDefault();
    await api.createDeployment(selected.id, yaml, isDeletion);
    setShowDeploy(false);
    setYaml('');
    setIsDeletion(false);
    const deps = await api.getStackDeployments(selected.id);
    setDeployments(deps);
  };

  const addLabel = async (label) => {
    await api.addStackLabel(selected.id, label);
    const labels = await api.getStackLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  const removeLabel = async (label) => {
    await api.removeStackLabel(selected.id, label);
    const labels = await api.getStackLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  const addAnnotation = async (key, value) => {
    await api.addStackAnnotation(selected.id, key, value);
    const annotations = await api.getStackAnnotations(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
  };

  const removeAnnotation = async (key) => {
    await api.removeStackAnnotation(selected.id, key);
    const annotations = await api.getStackAnnotations(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
  };

  const copyDeployment = async (depId) => {
    const dep = await api.getDeployment(depId);
    setYaml(dep.yaml_content);
    setIsDeletion(dep.is_deletion_marker);
    setShowDeploy(true);
  };

  const requestDiagnostic = async (depId, agentId) => {
    try {
      const req = await api.createDiagnostic(depId, agentId, 'ui-slim', 60);
      setDiagnosticResult({ status: 'pending', request: req, id: req.id });
      // Poll for result
      const pollResult = async () => {
        try {
          const res = await api.getDiagnostic(req.id);
          if (res.result) {
            setDiagnosticResult({ status: 'completed', request: res.request, result: res.result });
          } else if (res.request.status === 'claimed') {
            setDiagnosticResult({ status: 'claimed', request: res.request });
            setTimeout(pollResult, 2000);
          } else if (res.request.status === 'pending') {
            setTimeout(pollResult, 2000);
          }
        } catch (e) { console.error(e); }
      };
      setTimeout(pollResult, 2000);
    } catch (e) { console.error(e); }
  };

  if (loading) return <div className="loading">Loading stacks...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>◫ Stacks</h2>
        <div className="panel-actions">
          <button onClick={() => setShowCreate(true)} className="btn-primary">+ New Stack</button>
          <button onClick={load} className="btn-icon">↻</button>
        </div>
      </div>

      {stacks.length === 0 ? (
        <div className="empty">No stacks found</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Description</th>
                <th>Labels</th>
                <th>Deployments</th>
              </tr>
            </thead>
            <tbody>
              {stacks.map((s) => (
                <tr key={s.id} onClick={() => selectStack(s)} className="clickable">
                  <td className="mono">{s.name}</td>
                  <td className="dim">{s.description || '—'}</td>
                  <td>
                    {details[s.id]?.labels?.map((l) => (
                      <Tag key={l.id} variant="label">{l.label}</Tag>
                    ))}
                  </td>
                  <td>—</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showCreate && (
        <Modal title="Create Stack" onClose={() => setShowCreate(false)}>
          <form onSubmit={create} className="form">
            <label>Name<input value={newStack.name} onChange={(e) => setNewStack({ ...newStack, name: e.target.value })} required /></label>
            <label>Description<input value={newStack.description} onChange={(e) => setNewStack({ ...newStack, description: e.target.value })} /></label>
            <label>Generator
              <select value={newStack.generatorId} onChange={(e) => setNewStack({ ...newStack, generatorId: e.target.value })} required>
                <option value="">Select...</option>
                {generators.map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
              </select>
            </label>
            <div className="form-actions">
              <button type="button" onClick={() => setShowCreate(false)}>Cancel</button>
              <button type="submit" className="btn-primary">Create</button>
            </div>
          </form>
        </Modal>
      )}

      {selected && !showDeploy && (
        <Modal title={`Stack: ${selected.name}`} onClose={() => setSelected(null)}>
          <div className="detail-section">
            <h4>Labels</h4>
            <div className="tags">
              {details[selected.id]?.labels?.map((l) => (
                <Tag key={l.id} onRemove={() => removeLabel(l.label)}>{l.label}</Tag>
              ))}
            </div>
            <InlineAdd placeholder="Add label..." onAdd={addLabel} />
          </div>

          <div className="detail-section">
            <h4>Annotations</h4>
            <div className="tags">
              {details[selected.id]?.annotations?.map((a) => (
                <Tag key={a.key} onRemove={() => removeAnnotation(a.key)}>{a.key}={a.value}</Tag>
              ))}
            </div>
            <InlineAdd placeholder="Add annotation..." onAdd={addAnnotation} fields={2} />
          </div>

          <div className="detail-section">
            <div className="section-header-row">
              <h4>Deployment Objects</h4>
              <div className="section-header-actions">
                {stackHealth && <Status status={stackHealth.overall_status} />}
                <button onClick={() => { setYaml(''); setIsDeletion(false); setShowDeploy(true); }} className="btn-small">+ Deploy</button>
              </div>
            </div>
            {deployments.length === 0 ? (
              <div className="empty-small">No deployments</div>
            ) : (
              <div className="deployments-list">
                {deployments.map((d) => {
                  const health = stackHealth?.deployment_objects?.find(h => h.id === d.id);
                  return (
                    <div key={d.id} className="deployment-row">
                      <span className="mono">{d.id.slice(0, 8)}...</span>
                      <Tag variant={d.is_deletion_marker ? 'danger' : 'success'}>{d.is_deletion_marker ? 'DELETE' : 'DEPLOY'}</Tag>
                      {health && <Status status={health.status} />}
                      {health && health.healthy_agents + health.degraded_agents + health.failing_agents > 0 && (
                        <span className="health-counts dim">
                          {health.healthy_agents > 0 && <span className="health-healthy">{health.healthy_agents}✓</span>}
                          {health.degraded_agents > 0 && <span className="health-degraded">{health.degraded_agents}⚠</span>}
                          {health.failing_agents > 0 && <span className="health-failing">{health.failing_agents}✕</span>}
                        </span>
                      )}
                      <span className="dim flex-fill">{new Date(d.created_at).toLocaleString()}</span>
                      <button onClick={() => copyDeployment(d.id)} className="btn-icon" title="Copy as new">⧉</button>
                      <button onClick={() => setShowDiagnostic(d)} className="btn-icon" title="Run diagnostics">🔍</button>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </Modal>
      )}

      {showDiagnostic && (
        <Modal title="Run Diagnostics" onClose={() => { setShowDiagnostic(null); setDiagnosticResult(null); }}>
          {!diagnosticResult ? (
            <div className="form">
              <p className="dim">Select an agent to run diagnostics on deployment object {showDiagnostic.id.slice(0, 8)}...</p>
              <label>Agent
                <select onChange={(e) => e.target.value && requestDiagnostic(showDiagnostic.id, e.target.value)} defaultValue="">
                  <option value="">Select agent...</option>
                  {agents.map((a) => <option key={a.id} value={a.id}>{a.name} ({a.cluster_name})</option>)}
                </select>
              </label>
            </div>
          ) : diagnosticResult.status === 'completed' ? (
            <div className="diagnostic-result">
              <div className="detail-grid">
                <div className="detail-row">
                  <span className="detail-label">Status</span>
                  <Status status="completed" />
                </div>
                <div className="detail-row">
                  <span className="detail-label">Collected</span>
                  <span className="dim">{new Date(diagnosticResult.result.collected_at).toLocaleString()}</span>
                </div>
              </div>
              <div className="detail-section">
                <h4>Pod Statuses</h4>
                <pre className="code-block">{diagnosticResult.result.pod_statuses}</pre>
              </div>
              <div className="detail-section">
                <h4>Events</h4>
                <pre className="code-block">{diagnosticResult.result.events}</pre>
              </div>
              {diagnosticResult.result.log_tails && (
                <div className="detail-section">
                  <h4>Logs</h4>
                  <pre className="code-block">{diagnosticResult.result.log_tails}</pre>
                </div>
              )}
            </div>
          ) : (
            <div className="diagnostic-pending">
              <div className="loading-spinner"></div>
              <p>Diagnostics {diagnosticResult.status}...</p>
              <p className="dim">Request ID: {diagnosticResult.request?.id?.slice(0, 8)}...</p>
            </div>
          )}
        </Modal>
      )}

      {showDeploy && (
        <Modal title="Create Deployment" onClose={() => setShowDeploy(false)}>
          <form onSubmit={deploy} className="form">
            <label>YAML Content
              <textarea value={yaml} onChange={(e) => setYaml(e.target.value)} rows={15} className="mono" required />
            </label>
            <label className="checkbox">
              <input type="checkbox" checked={isDeletion} onChange={(e) => setIsDeletion(e.target.checked)} />
              Mark as deletion
            </label>
            <div className="form-actions">
              <button type="button" onClick={() => setShowDeploy(false)}>Cancel</button>
              <button type="submit" className="btn-primary">Deploy</button>
            </div>
          </form>
        </Modal>
      )}
    </div>
  );
};

// ==================== TEMPLATES PANEL ====================
const TemplatesPanel = ({ stacks }) => {
  const [templates, setTemplates] = useState([]);
  const [details, setDetails] = useState({});
  const [selected, setSelected] = useState(null);
  const [showCreate, setShowCreate] = useState(false);
  const [showInstantiate, setShowInstantiate] = useState(false);
  const [newTemplate, setNewTemplate] = useState({ name: '', description: '', content: '', schema: '{}' });
  const [instantiateForm, setInstantiateForm] = useState({ stackId: '', params: '{}' });
  const [loading, setLoading] = useState(true);

  const defaultContent = `apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}
  namespace: {{ namespace | default(value="default") }}
data:
  key: {{ value }}`;

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.getTemplates();
      setTemplates(data);
      const d = {};
      await Promise.all(data.map(async (t) => {
        const [labels, annotations] = await Promise.all([api.getTemplateLabels(t.id), api.getTemplateAnnotations(t.id)]);
        d[t.id] = { labels, annotations };
      }));
      setDetails(d);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    await api.createTemplate(newTemplate.name, newTemplate.description, newTemplate.content, newTemplate.schema);
    setShowCreate(false);
    setNewTemplate({ name: '', description: '', content: '', schema: '{}' });
    load();
  };

  const instantiate = async (e) => {
    e.preventDefault();
    const params = JSON.parse(instantiateForm.params);
    await api.instantiateTemplate(instantiateForm.stackId, selected.id, params);
    setShowInstantiate(false);
    setInstantiateForm({ stackId: '', params: '{}' });
  };

  const remove = async (id) => {
    if (window.confirm('Delete this template?')) {
      await api.deleteTemplate(id);
      setSelected(null);
      load();
    }
  };

  const addLabel = async (label) => {
    await api.addTemplateLabel(selected.id, label);
    const labels = await api.getTemplateLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  const removeLabel = async (label) => {
    await api.removeTemplateLabel(selected.id, label);
    const labels = await api.getTemplateLabels(selected.id);
    setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
  };

  if (loading) return <div className="loading">Loading templates...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>⬢ Templates</h2>
        <div className="panel-actions">
          <button onClick={() => { setNewTemplate({ ...newTemplate, content: defaultContent }); setShowCreate(true); }} className="btn-primary">+ New Template</button>
          <button onClick={load} className="btn-icon">↻</button>
        </div>
      </div>

      {templates.length === 0 ? (
        <div className="empty">No templates found</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Version</th>
                <th>Description</th>
                <th>Labels</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {templates.map((t) => (
                <tr key={t.id} onClick={() => setSelected(t)} className="clickable">
                  <td className="mono">{t.name}</td>
                  <td><Tag variant="version">v{t.version}</Tag></td>
                  <td className="dim">{t.description || '—'}</td>
                  <td>
                    {details[t.id]?.labels?.map((l) => (
                      <Tag key={l.id} variant="label">{l.label}</Tag>
                    ))}
                  </td>
                  <td>
                    <button onClick={(e) => { e.stopPropagation(); setSelected(t); setShowInstantiate(true); }} className="btn-small">▶ Use</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showCreate && (
        <Modal title="Create Template" onClose={() => setShowCreate(false)}>
          <form onSubmit={create} className="form">
            <label>Name<input value={newTemplate.name} onChange={(e) => setNewTemplate({ ...newTemplate, name: e.target.value })} required /></label>
            <label>Description<input value={newTemplate.description} onChange={(e) => setNewTemplate({ ...newTemplate, description: e.target.value })} /></label>
            <label>Template Content (Tera/Jinja2)
              <textarea value={newTemplate.content} onChange={(e) => setNewTemplate({ ...newTemplate, content: e.target.value })} rows={12} className="mono" required />
            </label>
            <label>Parameters Schema (JSON)
              <textarea value={newTemplate.schema} onChange={(e) => setNewTemplate({ ...newTemplate, schema: e.target.value })} rows={6} className="mono" />
            </label>
            <div className="form-actions">
              <button type="button" onClick={() => setShowCreate(false)}>Cancel</button>
              <button type="submit" className="btn-primary">Create</button>
            </div>
          </form>
        </Modal>
      )}

      {selected && !showInstantiate && (
        <Modal title={`Template: ${selected.name}`} onClose={() => setSelected(null)}>
          <div className="detail-grid">
            <div className="detail-row">
              <span className="detail-label">Version</span>
              <Tag variant="version">v{selected.version}</Tag>
            </div>
            <div className="detail-row">
              <span className="detail-label">Description</span>
              <span>{selected.description || '—'}</span>
            </div>
          </div>

          <div className="detail-section">
            <h4>Labels</h4>
            <div className="tags">
              {details[selected.id]?.labels?.map((l) => (
                <Tag key={l.id} onRemove={() => removeLabel(l.label)}>{l.label}</Tag>
              ))}
            </div>
            <InlineAdd placeholder="Add label..." onAdd={addLabel} />
          </div>

          <div className="detail-section">
            <h4>Template Content</h4>
            <pre className="code-block">{selected.template_content}</pre>
          </div>

          <div className="detail-section">
            <h4>Parameters Schema</h4>
            <pre className="code-block">{selected.parameters_schema}</pre>
          </div>

          <div className="form-actions">
            <button onClick={() => remove(selected.id)} className="btn-danger">Delete</button>
            <button onClick={() => setShowInstantiate(true)} className="btn-primary">▶ Instantiate</button>
          </div>
        </Modal>
      )}

      {showInstantiate && selected && (
        <Modal title={`Instantiate: ${selected.name}`} onClose={() => setShowInstantiate(false)}>
          <form onSubmit={instantiate} className="form">
            <label>Target Stack
              <select value={instantiateForm.stackId} onChange={(e) => setInstantiateForm({ ...instantiateForm, stackId: e.target.value })} required>
                <option value="">Select stack...</option>
                {stacks.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
            </label>
            <label>Parameters (JSON)
              <textarea value={instantiateForm.params} onChange={(e) => setInstantiateForm({ ...instantiateForm, params: e.target.value })} rows={8} className="mono" />
            </label>
            <div className="form-actions">
              <button type="button" onClick={() => setShowInstantiate(false)}>Cancel</button>
              <button type="submit" className="btn-primary">▶ Instantiate</button>
            </div>
          </form>
        </Modal>
      )}
    </div>
  );
};

// ==================== JOBS PANEL ====================
const JobsPanel = ({ agents }) => {
  const [workOrders, setWorkOrders] = useState([]);
  const [workOrderLog, setWorkOrderLog] = useState([]);
  const [showCreate, setShowCreate] = useState(false);
  const [showLog, setShowLog] = useState(false);
  const [selected, setSelected] = useState(null);
  const [form, setForm] = useState({
    workType: 'build',
    yamlContent: '',
    targetAgentIds: [],
    targetLabels: '',
    maxRetries: 3,
    backoffSeconds: 60
  });
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [orders, log] = await Promise.all([
        api.getWorkOrders(),
        api.getWorkOrderLog(null, null, null, 20)
      ]);
      setWorkOrders(orders);
      setWorkOrderLog(log);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    const targeting = {};
    if (form.targetAgentIds.length > 0) targeting.agent_ids = form.targetAgentIds;
    if (form.targetLabels.trim()) targeting.labels = form.targetLabels.split(',').map(l => l.trim()).filter(Boolean);

    await api.createWorkOrder(form.workType, form.yamlContent, targeting, {
      maxRetries: form.maxRetries,
      backoffSeconds: form.backoffSeconds
    });
    setShowCreate(false);
    setForm({ workType: 'build', yamlContent: '', targetAgentIds: [], targetLabels: '', maxRetries: 3, backoffSeconds: 60 });
    load();
  };

  const cancel = async (id) => {
    if (window.confirm('Cancel this work order?')) {
      await api.deleteWorkOrder(id);
      load();
    }
  };

  if (loading) return <div className="loading">Loading jobs...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>⚡ Jobs</h2>
        <div className="panel-actions">
          <button onClick={() => setShowCreate(true)} className="btn-primary">+ New Job</button>
          <button onClick={load} className="btn-icon">↻</button>
        </div>
      </div>

      <Section title="Active Work Orders" icon="▶" count={workOrders.length} defaultOpen>
        {workOrders.length === 0 ? (
          <div className="empty-small">No active work orders</div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Type</th>
                  <th>Status</th>
                  <th>Retries</th>
                  <th>Created</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {workOrders.map((wo) => (
                  <tr key={wo.id} onClick={() => setSelected(wo)} className="clickable">
                    <td className="mono">{wo.id.slice(0, 8)}...</td>
                    <td><Tag variant="info">{wo.work_type}</Tag></td>
                    <td><Status status={wo.status} /></td>
                    <td>{wo.retry_count}/{wo.max_retries}</td>
                    <td className="dim">{new Date(wo.created_at).toLocaleString()}</td>
                    <td>
                      <button onClick={(e) => { e.stopPropagation(); cancel(wo.id); }} className="btn-icon" title="Cancel">✕</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Section>

      <Section title="History" icon="📋" count={workOrderLog.length}>
        {workOrderLog.length === 0 ? (
          <div className="empty-small">No completed work orders</div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Result</th>
                  <th>Agent</th>
                  <th>Completed</th>
                  <th>Message</th>
                </tr>
              </thead>
              <tbody>
                {workOrderLog.map((log) => (
                  <tr key={log.id}>
                    <td><Tag variant="info">{log.work_type}</Tag></td>
                    <td><Status status={log.success ? 'success' : 'failed'} /></td>
                    <td className="mono dim">{log.completed_by?.slice(0, 8) || '—'}...</td>
                    <td className="dim">{new Date(log.completed_at).toLocaleString()}</td>
                    <td className="mono dim">{log.result_message?.slice(0, 30) || '—'}{log.result_message?.length > 30 ? '...' : ''}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Section>

      {showCreate && (
        <Modal title="Create Work Order" onClose={() => setShowCreate(false)}>
          <form onSubmit={create} className="form">
            <label>Work Type
              <select value={form.workType} onChange={(e) => setForm({ ...form, workType: e.target.value })}>
                <option value="build">build</option>
                <option value="test">test</option>
                <option value="backup">backup</option>
                <option value="deploy">deploy</option>
                <option value="custom">custom</option>
              </select>
            </label>
            <label>YAML Content
              <textarea value={form.yamlContent} onChange={(e) => setForm({ ...form, yamlContent: e.target.value })} rows={10} className="mono" required />
            </label>
            <label>Target Agents
              <select multiple value={form.targetAgentIds} onChange={(e) => setForm({ ...form, targetAgentIds: Array.from(e.target.selectedOptions, o => o.value) })}>
                {agents.map((a) => <option key={a.id} value={a.id}>{a.name} ({a.cluster_name})</option>)}
              </select>
            </label>
            <label>Target Labels (comma-separated)
              <input value={form.targetLabels} onChange={(e) => setForm({ ...form, targetLabels: e.target.value })} placeholder="label1, label2" />
            </label>
            <div className="form-row">
              <label>Max Retries
                <input type="number" value={form.maxRetries} onChange={(e) => setForm({ ...form, maxRetries: parseInt(e.target.value) })} min="0" max="10" />
              </label>
              <label>Backoff (seconds)
                <input type="number" value={form.backoffSeconds} onChange={(e) => setForm({ ...form, backoffSeconds: parseInt(e.target.value) })} min="1" />
              </label>
            </div>
            <div className="form-actions">
              <button type="button" onClick={() => setShowCreate(false)}>Cancel</button>
              <button type="submit" className="btn-primary">Create</button>
            </div>
          </form>
        </Modal>
      )}

      {selected && (
        <Modal title="Work Order Details" onClose={() => setSelected(null)}>
          <div className="detail-grid">
            <div className="detail-row">
              <span className="detail-label">ID</span>
              <span className="mono">{selected.id}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Type</span>
              <Tag variant="info">{selected.work_type}</Tag>
            </div>
            <div className="detail-row">
              <span className="detail-label">Status</span>
              <Status status={selected.status} />
            </div>
            <div className="detail-row">
              <span className="detail-label">Retries</span>
              <span>{selected.retry_count} / {selected.max_retries}</span>
            </div>
            {selected.claimed_by && (
              <div className="detail-row">
                <span className="detail-label">Claimed By</span>
                <span className="mono">{selected.claimed_by}</span>
              </div>
            )}
          </div>
          <div className="detail-section">
            <h4>YAML Content</h4>
            <pre className="code-block">{selected.yaml_content}</pre>
          </div>
        </Modal>
      )}
    </div>
  );
};

// ==================== ADMIN PANEL ====================
const AdminPanel = ({ onGeneratorsChange, onAgentsChange }) => {
  const [agents, setAgents] = useState([]);
  const [generators, setGenerators] = useState([]);
  const [showCreate, setShowCreate] = useState(null);
  const [form, setForm] = useState({ name: '', cluster: '', description: '' });
  const [newPak, setNewPak] = useState(null);
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [a, g] = await Promise.all([api.getAgents(), api.getGenerators()]);
      setAgents(a);
      setGenerators(g);
      onAgentsChange?.(a);
      onGeneratorsChange?.(g);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, [onAgentsChange, onGeneratorsChange]);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    try {
      if (showCreate === 'agent') {
        const res = await api.createAgent(form.name, form.cluster);
        setNewPak(res.initial_pak);
      } else {
        const res = await api.createGenerator(form.name, form.description);
        setNewPak(res.pak);
      }
      setCopied(false);
      load();
    } catch (e) { console.error(e); }
  };

  const rotate = async (type, id) => {
    try {
      const res = type === 'agent' ? await api.rotateAgentPak(id) : await api.rotateGeneratorPak(id);
      setNewPak(res.pak);
      setCopied(false);
      setShowCreate(type);
    } catch (e) { console.error(e); }
  };

  const copy = () => {
    navigator.clipboard.writeText(newPak);
    setCopied(true);
  };

  const closeCreate = () => {
    if (newPak && !copied && !window.confirm('PAK not copied. Close anyway?')) return;
    setShowCreate(null);
    setNewPak(null);
    setCopied(false);
    setForm({ name: '', cluster: '', description: '' });
  };

  if (loading) return <div className="loading">Loading...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>⚙ Admin</h2>
      </div>

      <Section title="Agent PAKs" icon="⬡" count={agents.length} defaultOpen>
        <button onClick={() => setShowCreate('agent')} className="btn-primary btn-block">+ Create Agent PAK</button>
        {agents.length === 0 ? (
          <div className="empty-small">No agents</div>
        ) : (
          <div className="pak-list">
            {agents.map((a) => (
              <div key={a.id} className="pak-row">
                <div>
                  <span className="mono">{a.name}</span>
                  <span className="dim"> @ {a.cluster_name}</span>
                </div>
                <div>
                  <Status status={a.status} />
                  <button onClick={() => rotate('agent', a.id)} className="btn-icon" title="Rotate PAK">↻</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </Section>

      <Section title="Generator PAKs" icon="◈" count={generators.length}>
        <button onClick={() => setShowCreate('generator')} className="btn-primary btn-block">+ Create Generator PAK</button>
        {generators.length === 0 ? (
          <div className="empty-small">No generators</div>
        ) : (
          <div className="pak-list">
            {generators.map((g) => (
              <div key={g.id} className="pak-row">
                <div>
                  <span className="mono">{g.name}</span>
                  {g.description && <span className="dim"> — {g.description}</span>}
                </div>
                <button onClick={() => rotate('generator', g.id)} className="btn-icon" title="Rotate PAK">↻</button>
              </div>
            ))}
          </div>
        )}
      </Section>

      {showCreate && (
        <Modal title={`Create ${showCreate === 'agent' ? 'Agent' : 'Generator'} PAK`} onClose={closeCreate}>
          {newPak ? (
            <div className="pak-display">
              <div className="pak-warning">⚠ Copy this PAK now. It won't be shown again.</div>
              <div className="pak-value">
                <code>{newPak}</code>
                <button onClick={copy} className={`btn-icon ${copied ? 'success' : ''}`}>{copied ? '✓' : '⧉'}</button>
              </div>
            </div>
          ) : (
            <form onSubmit={create} className="form">
              <label>Name<input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} required /></label>
              {showCreate === 'agent' ? (
                <label>Cluster Name<input value={form.cluster} onChange={(e) => setForm({ ...form, cluster: e.target.value })} required /></label>
              ) : (
                <label>Description<input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></label>
              )}
              <div className="form-actions">
                <button type="button" onClick={closeCreate}>Cancel</button>
                <button type="submit" className="btn-primary">Create</button>
              </div>
            </form>
          )}
        </Modal>
      )}
    </div>
  );
};

// ==================== MAIN APP ====================
export default function App() {
  const [activePanel, setActivePanel] = useState('agents');
  const [stacks, setStacks] = useState([]);
  const [agents, setAgents] = useState([]);
  const [generators, setGenerators] = useState([]);
  const [toast, setToast] = useState(null);

  useEffect(() => {
    api.getGenerators().then(setGenerators).catch(console.error);
    api.getAgents().then(setAgents).catch(console.error);
  }, []);

  const showToast = (message, type = 'info') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  return (
    <div className="app">
      <header className="header">
        <div className="logo">
          <span className="logo-icon">◆</span>
          <span className="logo-text">BROKKR</span>
        </div>
        <nav className="nav">
          {['agents', 'stacks', 'templates', 'jobs', 'admin'].map((p) => (
            <button key={p} className={`nav-item ${activePanel === p ? 'active' : ''}`} onClick={() => setActivePanel(p)}>
              {p}
            </button>
          ))}
        </nav>
      </header>

      <main className="main">
        {activePanel === 'agents' && <AgentsPanel stacks={stacks} onRefresh={setAgents} />}
        {activePanel === 'stacks' && <StacksPanel generators={generators} agents={agents} onRefresh={setStacks} />}
        {activePanel === 'templates' && <TemplatesPanel stacks={stacks} />}
        {activePanel === 'jobs' && <JobsPanel agents={agents} />}
        {activePanel === 'admin' && <AdminPanel onGeneratorsChange={setGenerators} onAgentsChange={setAgents} />}
      </main>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
