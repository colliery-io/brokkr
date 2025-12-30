import React, { useState, useEffect, useCallback } from 'react';
import * as api from './api';
import './styles.css';

// Import shared components from modular components file
import {
  Tag,
  Section,
  InlineAdd,
  Status,
  Pagination,
  usePagination,
  Modal,
  ToastProvider,
  useToast,
  getErrorMessage
} from './components';

// ==================== AGENTS PANEL ====================
const AgentsPanel = ({ stacks, onRefresh }) => {
  const [agents, setAgents] = useState([]);
  const [details, setDetails] = useState({});
  const [selected, setSelected] = useState(null);
  const [events, setEvents] = useState([]);
  const [loading, setLoading] = useState(true);
  const toast = useToast();
  const pagination = usePagination(agents);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.getAgents();
      // Sort agents: ACTIVE first, then by name
      data.sort((a, b) => {
        if (a.status === 'ACTIVE' && b.status !== 'ACTIVE') return -1;
        if (a.status !== 'ACTIVE' && b.status === 'ACTIVE') return 1;
        return a.name.localeCompare(b.name);
      });
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
    } catch (e) {
      toast?.('Failed to load agents: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [onRefresh, toast]);

  useEffect(() => { load(); }, [load]);

  const selectAgent = async (agent) => {
    setSelected(agent);
    try {
      const evts = await api.getAgentEvents(agent.id);
      setEvents(evts);
    } catch (e) {
      toast?.('Failed to load agent events: ' + getErrorMessage(e), 'error');
      setEvents([]);
    }
  };

  const addLabel = async (label) => {
    try {
      await api.addAgentLabel(selected.id, label);
      const labels = await api.getAgentLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
      toast?.('Label added', 'success');
    } catch (e) {
      toast?.('Failed to add label: ' + getErrorMessage(e), 'error');
    }
  };

  const removeLabel = async (label) => {
    try {
      await api.removeAgentLabel(selected.id, label);
      const labels = await api.getAgentLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
    } catch (e) {
      toast?.('Failed to remove label: ' + getErrorMessage(e), 'error');
    }
  };

  const addAnnotation = async (key, value) => {
    try {
      await api.addAgentAnnotation(selected.id, key, value);
      const annotations = await api.getAgentAnnotations(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
      toast?.('Annotation added', 'success');
    } catch (e) {
      toast?.('Failed to add annotation: ' + getErrorMessage(e), 'error');
    }
  };

  const removeAnnotation = async (key) => {
    try {
      await api.removeAgentAnnotation(selected.id, key);
      const annotations = await api.getAgentAnnotations(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
    } catch (e) {
      toast?.('Failed to remove annotation: ' + getErrorMessage(e), 'error');
    }
  };

  const addTarget = async (stackId) => {
    try {
      await api.addAgentTarget(selected.id, stackId);
      const targets = await api.getAgentTargets(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], targets } });
      toast?.('Target added', 'success');
    } catch (e) {
      toast?.('Failed to add target: ' + getErrorMessage(e), 'error');
    }
  };

  const removeTarget = async (stackId) => {
    try {
      await api.removeAgentTarget(selected.id, stackId);
      const targets = await api.getAgentTargets(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], targets } });
    } catch (e) {
      toast?.('Failed to remove target: ' + getErrorMessage(e), 'error');
    }
  };

  const toggleStatus = async () => {
    try {
      const newStatus = selected.status === 'ACTIVE' ? 'INACTIVE' : 'ACTIVE';
      const updated = await api.updateAgent(selected.id, { status: newStatus });
      setSelected(updated);
      setAgents(agents.map(a => a.id === updated.id ? updated : a));
      onRefresh?.(agents.map(a => a.id === updated.id ? updated : a));
      toast?.(`Agent ${newStatus.toLowerCase()}`, 'success');
    } catch (e) {
      toast?.('Failed to update agent status: ' + getErrorMessage(e), 'error');
    }
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
        <>
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
                {pagination.paginatedItems.map((a) => (
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
          {agents.length > 10 && (
            <Pagination
              page={pagination.page}
              totalPages={pagination.totalPages}
              onPageChange={pagination.setPage}
              pageSize={pagination.pageSize}
              onPageSizeChange={pagination.setPageSize}
              total={pagination.total}
            />
          )}
        </>
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
  const toast = useToast();
  const pagination = usePagination(stacks);

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
    } catch (e) {
      toast?.('Failed to load stacks: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [onRefresh, toast]);

  useEffect(() => { load(); }, [load]);

  const selectStack = async (stack) => {
    setSelected(stack);
    setStackHealth(null);
    try {
      const [deps, health] = await Promise.all([
        api.getStackDeployments(stack.id),
        api.getStackHealth(stack.id).catch(() => null)
      ]);
      setDeployments(deps);
      setStackHealth(health);
    } catch (e) {
      toast?.('Failed to load stack details: ' + getErrorMessage(e), 'error');
    }
  };

  const create = async (e) => {
    e.preventDefault();
    try {
      await api.createStack(newStack.name, newStack.description, newStack.generatorId);
      setShowCreate(false);
      setNewStack({ name: '', description: '', generatorId: '' });
      toast?.('Stack created', 'success');
      load();
    } catch (e) {
      toast?.('Failed to create stack: ' + getErrorMessage(e), 'error');
    }
  };

  const deploy = async (e) => {
    e.preventDefault();
    try {
      await api.createDeployment(selected.id, yaml, isDeletion);
      setShowDeploy(false);
      setYaml('');
      setIsDeletion(false);
      const deps = await api.getStackDeployments(selected.id);
      setDeployments(deps);
      toast?.('Deployment created', 'success');
    } catch (e) {
      toast?.('Failed to create deployment: ' + getErrorMessage(e), 'error');
    }
  };

  const addLabel = async (label) => {
    try {
      await api.addStackLabel(selected.id, label);
      const labels = await api.getStackLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
      toast?.('Label added', 'success');
    } catch (e) {
      toast?.('Failed to add label: ' + getErrorMessage(e), 'error');
    }
  };

  const removeLabel = async (label) => {
    try {
      await api.removeStackLabel(selected.id, label);
      const labels = await api.getStackLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
    } catch (e) {
      toast?.('Failed to remove label: ' + getErrorMessage(e), 'error');
    }
  };

  const addAnnotation = async (key, value) => {
    try {
      await api.addStackAnnotation(selected.id, key, value);
      const annotations = await api.getStackAnnotations(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
      toast?.('Annotation added', 'success');
    } catch (e) {
      toast?.('Failed to add annotation: ' + getErrorMessage(e), 'error');
    }
  };

  const removeAnnotation = async (key) => {
    try {
      await api.removeStackAnnotation(selected.id, key);
      const annotations = await api.getStackAnnotations(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], annotations } });
    } catch (e) {
      toast?.('Failed to remove annotation: ' + getErrorMessage(e), 'error');
    }
  };

  const copyDeployment = async (depId) => {
    try {
      const dep = await api.getDeployment(depId);
      setYaml(dep.yaml_content);
      setIsDeletion(dep.is_deletion_marker);
      setShowDeploy(true);
    } catch (e) {
      toast?.('Failed to load deployment: ' + getErrorMessage(e), 'error');
    }
  };

  const requestDiagnostic = async (depId, agentId) => {
    try {
      const req = await api.createDiagnostic(depId, agentId, 'ui-slim', 60);
      setDiagnosticResult({ status: 'pending', request: req, id: req.id });
      toast?.('Diagnostic requested', 'success');
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
        } catch (e) {
          toast?.('Failed to poll diagnostic: ' + getErrorMessage(e), 'error');
        }
      };
      setTimeout(pollResult, 2000);
    } catch (e) {
      toast?.('Failed to request diagnostic: ' + getErrorMessage(e), 'error');
    }
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
        <>
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
                {pagination.paginatedItems.map((s) => (
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
          {stacks.length > 10 && (
            <Pagination
              page={pagination.page}
              totalPages={pagination.totalPages}
              onPageChange={pagination.setPage}
              pageSize={pagination.pageSize}
              onPageSizeChange={pagination.setPageSize}
              total={pagination.total}
            />
          )}
        </>
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
  const toast = useToast();
  const pagination = usePagination(templates);

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
    } catch (e) {
      toast?.('Failed to load templates: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [toast]);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    try {
      await api.createTemplate(newTemplate.name, newTemplate.description, newTemplate.content, newTemplate.schema);
      setShowCreate(false);
      setNewTemplate({ name: '', description: '', content: '', schema: '{}' });
      toast?.('Template created', 'success');
      load();
    } catch (e) {
      toast?.('Failed to create template: ' + getErrorMessage(e), 'error');
    }
  };

  const instantiate = async (e) => {
    e.preventDefault();
    try {
      const params = JSON.parse(instantiateForm.params);
      await api.instantiateTemplate(instantiateForm.stackId, selected.id, params);
      setShowInstantiate(false);
      setInstantiateForm({ stackId: '', params: '{}' });
      toast?.('Template instantiated', 'success');
    } catch (e) {
      toast?.('Failed to instantiate template: ' + getErrorMessage(e), 'error');
    }
  };

  const remove = async (id) => {
    if (window.confirm('Delete this template?')) {
      try {
        await api.deleteTemplate(id);
        setSelected(null);
        toast?.('Template deleted', 'success');
        load();
      } catch (e) {
        toast?.('Failed to delete template: ' + getErrorMessage(e), 'error');
      }
    }
  };

  const addLabel = async (label) => {
    try {
      await api.addTemplateLabel(selected.id, label);
      const labels = await api.getTemplateLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
      toast?.('Label added', 'success');
    } catch (e) {
      toast?.('Failed to add label: ' + getErrorMessage(e), 'error');
    }
  };

  const removeLabel = async (label) => {
    try {
      await api.removeTemplateLabel(selected.id, label);
      const labels = await api.getTemplateLabels(selected.id);
      setDetails({ ...details, [selected.id]: { ...details[selected.id], labels } });
    } catch (e) {
      toast?.('Failed to remove label: ' + getErrorMessage(e), 'error');
    }
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
        <>
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
                {pagination.paginatedItems.map((t) => (
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
          {templates.length > 10 && (
            <Pagination
              page={pagination.page}
              totalPages={pagination.totalPages}
              onPageChange={pagination.setPage}
              pageSize={pagination.pageSize}
              onPageSizeChange={pagination.setPageSize}
              total={pagination.total}
            />
          )}
        </>
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
  const [buildDemoRunning, setBuildDemoRunning] = useState(false);
  const [buildDemoStatus, setBuildDemoStatus] = useState(null);
  const toast = useToast();

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [orders, log] = await Promise.all([
        api.getWorkOrders(),
        api.getWorkOrderLog(null, null, null, 20)
      ]);
      setWorkOrders(orders);
      setWorkOrderLog(log);
    } catch (e) {
      toast?.('Failed to load jobs: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [toast]);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    try {
      const targeting = {};
      if (form.targetAgentIds.length > 0) targeting.agent_ids = form.targetAgentIds;
      if (form.targetLabels.trim()) targeting.labels = form.targetLabels.split(',').map(l => l.trim()).filter(Boolean);

      await api.createWorkOrder(form.workType, form.yamlContent, targeting, {
        maxRetries: form.maxRetries,
        backoffSeconds: form.backoffSeconds
      });
      setShowCreate(false);
      setForm({ workType: 'build', yamlContent: '', targetAgentIds: [], targetLabels: '', maxRetries: 3, backoffSeconds: 60 });
      toast?.('Work order created', 'success');
      load();
    } catch (e) {
      toast?.('Failed to create work order: ' + getErrorMessage(e), 'error');
    }
  };

  const cancel = async (id) => {
    if (window.confirm('Cancel this work order?')) {
      try {
        await api.deleteWorkOrder(id);
        toast?.('Work order cancelled', 'success');
        load();
      } catch (e) {
        toast?.('Failed to cancel work order: ' + getErrorMessage(e), 'error');
      }
    }
  };

  // Build Demo - creates a build work order for the webhook-catcher
  const runBuildDemo = async () => {
    if (buildDemoRunning) return;
    setBuildDemoRunning(true);
    setBuildDemoStatus({ step: 'creating', message: 'Creating build work order...' });

    try {
      // Create the build work order
      const workOrder = await api.createBuildWorkOrder();
      setBuildDemoStatus({ step: 'pending', message: 'Build work order created, waiting for agent to claim...', workOrderId: workOrder.id });
      toast?.('Build work order created', 'success');

      // Poll for completion
      const pollInterval = setInterval(async () => {
        try {
          // Check if still in active work orders
          const orders = await api.getWorkOrders();
          const current = orders.find(o => o.id === workOrder.id);

          if (current) {
            if (current.status === 'CLAIMED') {
              setBuildDemoStatus({ step: 'building', message: 'Agent claimed work order, building...', workOrderId: workOrder.id });
            } else if (current.status === 'RETRY_PENDING') {
              setBuildDemoStatus({ step: 'retrying', message: `Build failed, retrying (${current.retry_count}/${current.max_retries})...`, workOrderId: workOrder.id, error: current.last_error });
            }
          } else {
            // Check work order log for completion
            const log = await api.getWorkOrderLog('build', null, null, 10);
            const completed = log.find(l => l.original_work_order_id === workOrder.id);

            if (completed) {
              clearInterval(pollInterval);
              if (completed.success) {
                setBuildDemoStatus({
                  step: 'completed',
                  message: 'Build completed successfully!',
                  workOrderId: workOrder.id,
                  result: completed.result_message
                });
                toast?.('Build completed successfully!', 'success');
              } else {
                setBuildDemoStatus({
                  step: 'failed',
                  message: 'Build failed',
                  workOrderId: workOrder.id,
                  error: completed.result_message
                });
                toast?.('Build failed: ' + completed.result_message, 'error');
              }
              setBuildDemoRunning(false);
              load();
            }
          }
        } catch (e) {
          console.error('Error polling build status:', e);
        }
      }, 3000);

      // Timeout after 15 minutes
      setTimeout(() => {
        clearInterval(pollInterval);
        if (buildDemoRunning) {
          setBuildDemoRunning(false);
          setBuildDemoStatus({ step: 'timeout', message: 'Build timed out after 15 minutes' });
          toast?.('Build timed out', 'error');
        }
      }, 15 * 60 * 1000);

      load();
    } catch (e) {
      setBuildDemoRunning(false);
      setBuildDemoStatus({ step: 'error', message: 'Failed to create build work order: ' + getErrorMessage(e) });
      toast?.('Failed to create build work order: ' + getErrorMessage(e), 'error');
    }
  };

  // Pre-fill the create form with the webhook-catcher build YAML
  const prefillBuildDemo = () => {
    setForm({
      ...form,
      workType: 'build',
      yamlContent: api.getWebhookCatcherBuildYaml()
    });
    setShowCreate(true);
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

      <Section title="Build Demo" icon="🔨" defaultOpen>
        <p className="dim" style={{ marginBottom: '12px' }}>
          Build the webhook-catcher service from source using Shipwright. This demonstrates the complete build pipeline: Git clone → Container build → Push to registry.
        </p>
        <div className="build-demo-actions">
          <button
            onClick={runBuildDemo}
            className={`btn-primary ${buildDemoRunning ? 'disabled' : ''}`}
            disabled={buildDemoRunning}
          >
            {buildDemoRunning ? '⏳ Building...' : '▶ Run Build'}
          </button>
          <button onClick={prefillBuildDemo} className="btn-secondary">
            📝 View Build YAML
          </button>
        </div>

        {buildDemoStatus && (
          <div className={`build-demo-status build-demo-${buildDemoStatus.step}`}>
            <div className="build-demo-step">
              {buildDemoStatus.step === 'creating' && '⏳'}
              {buildDemoStatus.step === 'pending' && '⏳'}
              {buildDemoStatus.step === 'building' && '🔨'}
              {buildDemoStatus.step === 'retrying' && '🔄'}
              {buildDemoStatus.step === 'completed' && '✅'}
              {buildDemoStatus.step === 'failed' && '❌'}
              {buildDemoStatus.step === 'error' && '❌'}
              {buildDemoStatus.step === 'timeout' && '⏰'}
              <span>{buildDemoStatus.message}</span>
            </div>
            {buildDemoStatus.workOrderId && (
              <div className="build-demo-id dim">
                Work Order: {buildDemoStatus.workOrderId.slice(0, 8)}...
              </div>
            )}
            {buildDemoStatus.result && (
              <div className="build-demo-result">
                <strong>Image Digest:</strong> <code>{buildDemoStatus.result}</code>
              </div>
            )}
            {buildDemoStatus.error && (
              <div className="build-demo-error error-text">
                {buildDemoStatus.error}
              </div>
            )}
          </div>
        )}
      </Section>

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
                  <th>Last Error</th>
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
                    <td className="dim" title={wo.last_error || ''}>
                      {wo.last_error ? (wo.last_error.slice(0, 40) + (wo.last_error.length > 40 ? '...' : '')) : '—'}
                    </td>
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
            {selected.last_error && (
              <div className="detail-row">
                <span className="detail-label">Last Error</span>
                <span className="error-text">{selected.last_error}</span>
              </div>
            )}
            {selected.last_error_at && (
              <div className="detail-row">
                <span className="detail-label">Error Time</span>
                <span className="dim">{new Date(selected.last_error_at).toLocaleString()}</span>
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
  const [demoRunning, setDemoRunning] = useState(false);
  const [demoLogs, setDemoLogs] = useState([]);
  const [demoResults, setDemoResults] = useState(null);
  const toast = useToast();

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [a, g] = await Promise.all([api.getAgents(), api.getGenerators()]);
      setAgents(a);
      setGenerators(g);
      onAgentsChange?.(a);
      onGeneratorsChange?.(g);
    } catch (e) {
      toast?.('Failed to load admin data: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [onAgentsChange, onGeneratorsChange, toast]);

  useEffect(() => { load(); }, [load]);

  const create = async (e) => {
    e.preventDefault();
    try {
      if (showCreate === 'agent') {
        const res = await api.createAgent(form.name, form.cluster);
        setNewPak(res.initial_pak);
        toast?.('Agent created', 'success');
      } else {
        const res = await api.createGenerator(form.name, form.description);
        setNewPak(res.pak);
        toast?.('Generator created', 'success');
      }
      setCopied(false);
      load();
    } catch (e) {
      toast?.('Failed to create: ' + getErrorMessage(e), 'error');
    }
  };

  const rotate = async (type, id) => {
    try {
      const res = type === 'agent' ? await api.rotateAgentPak(id) : await api.rotateGeneratorPak(id);
      setNewPak(res.pak);
      setCopied(false);
      setShowCreate(type);
      toast?.('PAK rotated', 'success');
    } catch (e) {
      toast?.('Failed to rotate PAK: ' + getErrorMessage(e), 'error');
    }
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

  const runDemo = async () => {
    if (demoRunning) return;
    setDemoRunning(true);
    setDemoLogs([]);
    setDemoResults(null);

    try {
      await api.runDemoSetup((progress) => {
        setDemoLogs(logs => [...logs, { time: new Date().toLocaleTimeString(), message: progress.message }]);
        if (progress.done) {
          setDemoRunning(false);
          if (progress.results) {
            setDemoResults(progress.results);
            toast?.('Demo setup complete!', 'success');
            load(); // Refresh data
          } else if (progress.error) {
            toast?.('Demo setup failed: ' + progress.error.message, 'error');
          }
        }
      });
    } catch (e) {
      setDemoRunning(false);
      toast?.('Demo setup failed: ' + getErrorMessage(e), 'error');
    }
  };

  if (loading) return <div className="loading">Loading...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>⚙ Admin</h2>
      </div>

      <Section title="Demo Setup" icon="🚀" defaultOpen>
        <p className="dim" style={{ marginBottom: '12px' }}>
          Demonstrates real functionality: activates the deployed agent, deploys a second agent via K8s, creates stacks with deployments, and captures webhook events.
        </p>
        <button
          onClick={runDemo}
          className={`btn-primary btn-block ${demoRunning ? 'disabled' : ''}`}
          disabled={demoRunning}
        >
          {demoRunning ? '⏳ Running Demo Setup...' : '▶ Run Demo Setup'}
        </button>

        {demoLogs.length > 0 && (
          <div className="demo-log">
            {demoLogs.map((log, i) => (
              <div key={i} className="demo-log-entry">
                <span className="dim">{log.time}</span>
                <span>{log.message}</span>
              </div>
            ))}
          </div>
        )}

        {demoResults && (
          <div className="demo-results">
            {demoResults.realAgent && (
              <div className="demo-agent-info">
                <h4>Real Agent Activated:</h4>
                <div className="demo-agent-detail">
                  <span className="mono">{demoResults.realAgent.name}</span>
                  <Status status="ACTIVE" />
                </div>
              </div>
            )}

            {demoResults.stagingAgent && (
              <div className="demo-agent-info">
                <h4>Staging Agent Deployed:</h4>
                <div className="demo-agent-detail">
                  <span className="mono">{demoResults.stagingAgent.agent?.name || 'demo-staging-agent'}</span>
                  <Tag variant="info">deploying via K8s</Tag>
                </div>
              </div>
            )}

            <h4>Created Resources:</h4>
            <div className="demo-results-grid">
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.generators.length}</span>
                <span className="demo-result-label">Generators</span>
              </div>
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.agents.length}</span>
                <span className="demo-result-label">New Agents</span>
              </div>
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.stacks.length}</span>
                <span className="demo-result-label">Stacks</span>
              </div>
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.deployments.length}</span>
                <span className="demo-result-label">Deployments</span>
              </div>
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.templates.length}</span>
                <span className="demo-result-label">Templates</span>
              </div>
              <div className="demo-result-item">
                <span className="demo-result-count">{demoResults.webhooks.length}</span>
                <span className="demo-result-label">Webhooks</span>
              </div>
            </div>

            {demoResults.receivedWebhooks && demoResults.receivedWebhooks.length > 0 && (
              <div className="webhook-events">
                <h4>Received Webhook Events ({demoResults.receivedWebhooks.length}):</h4>
                <div className="webhook-events-list">
                  {demoResults.receivedWebhooks.slice(0, 10).map((event, i) => (
                    <div key={i} className="webhook-event">
                      <span className="webhook-event-type">{event.event_type}</span>
                      <span className="webhook-event-time">{new Date(event.received_at).toLocaleTimeString()}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </Section>

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

// ==================== WEBHOOKS PANEL ====================
const WebhooksPanel = () => {
  const [webhooks, setWebhooks] = useState([]);
  const [eventTypes, setEventTypes] = useState([]);
  const [selected, setSelected] = useState(null);
  const [deliveries, setDeliveries] = useState([]);
  const [showCreate, setShowCreate] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [form, setForm] = useState({
    name: '',
    url: '',
    eventTypes: [],
    authHeader: '',
    maxRetries: 5,
    timeoutSeconds: 30
  });
  const [loading, setLoading] = useState(true);
  const toast = useToast();
  const pagination = usePagination(webhooks);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [wh, types] = await Promise.all([api.getWebhooks(), api.getWebhookEventTypes()]);
      setWebhooks(wh);
      setEventTypes(types);
    } catch (e) {
      toast?.('Failed to load webhooks: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [toast]);

  useEffect(() => { load(); }, [load]);

  const selectWebhook = async (webhook) => {
    setSelected(webhook);
    try {
      const dels = await api.getWebhookDeliveries(webhook.id, null, 50);
      setDeliveries(dels);
    } catch (e) {
      toast?.('Failed to load deliveries: ' + getErrorMessage(e), 'error');
      setDeliveries([]);
    }
  };

  const create = async (e) => {
    e.preventDefault();
    try {
      await api.createWebhook(
        form.name,
        form.url,
        form.eventTypes,
        form.authHeader || null,
        { maxRetries: form.maxRetries, timeoutSeconds: form.timeoutSeconds }
      );
      setShowCreate(false);
      setForm({ name: '', url: '', eventTypes: [], authHeader: '', maxRetries: 5, timeoutSeconds: 30 });
      toast?.('Webhook created', 'success');
      load();
    } catch (e) {
      toast?.('Failed to create webhook: ' + getErrorMessage(e), 'error');
    }
  };

  const toggleEnabled = async (webhook) => {
    try {
      await api.updateWebhook(webhook.id, { enabled: !webhook.enabled });
      load();
      if (selected?.id === webhook.id) {
        setSelected({ ...selected, enabled: !webhook.enabled });
      }
      toast?.(webhook.enabled ? 'Webhook disabled' : 'Webhook enabled', 'success');
    } catch (e) {
      toast?.('Failed to update webhook: ' + getErrorMessage(e), 'error');
    }
  };

  const remove = async (id) => {
    if (window.confirm('Delete this webhook?')) {
      try {
        await api.deleteWebhook(id);
        setSelected(null);
        toast?.('Webhook deleted', 'success');
        load();
      } catch (e) {
        toast?.('Failed to delete webhook: ' + getErrorMessage(e), 'error');
      }
    }
  };

  const toggleEventType = (type) => {
    if (form.eventTypes.includes(type)) {
      setForm({ ...form, eventTypes: form.eventTypes.filter(t => t !== type) });
    } else {
      setForm({ ...form, eventTypes: [...form.eventTypes, type] });
    }
  };

  if (loading) return <div className="loading">Loading webhooks...</div>;

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>🔔 Webhooks</h2>
        <div className="panel-actions">
          <button onClick={() => setShowCreate(true)} className="btn-primary">+ New Webhook</button>
          <button onClick={load} className="btn-icon">↻</button>
        </div>
      </div>

      {webhooks.length === 0 ? (
        <div className="empty">No webhooks configured</div>
      ) : (
        <>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Status</th>
                  <th>Events</th>
                  <th>URL</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {pagination.paginatedItems.map((w) => (
                  <tr key={w.id} onClick={() => selectWebhook(w)} className="clickable">
                    <td className="mono">{w.name}</td>
                    <td>
                      <span className={`status status-${w.enabled ? 'green' : 'gray'}`}>
                        {w.enabled ? 'enabled' : 'disabled'}
                      </span>
                    </td>
                    <td>
                      {w.event_types?.slice(0, 3).map((e, i) => (
                        <Tag key={i} variant="info">{e}</Tag>
                      ))}
                      {w.event_types?.length > 3 && <span className="dim">+{w.event_types.length - 3}</span>}
                    </td>
                    <td className="dim">{w.has_url ? '••••••••' : '—'}</td>
                    <td>
                      <button
                        onClick={(e) => { e.stopPropagation(); toggleEnabled(w); }}
                        className="btn-icon"
                        title={w.enabled ? 'Disable' : 'Enable'}
                      >
                        {w.enabled ? '⏸' : '▶'}
                      </button>
                      <button
                        onClick={(e) => { e.stopPropagation(); remove(w.id); }}
                        className="btn-icon"
                        title="Delete"
                      >
                        ✕
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {webhooks.length > 10 && (
            <Pagination
              page={pagination.page}
              totalPages={pagination.totalPages}
              onPageChange={pagination.setPage}
              pageSize={pagination.pageSize}
              onPageSizeChange={pagination.setPageSize}
              total={pagination.total}
            />
          )}
        </>
      )}

      {showCreate && (
        <Modal title="Create Webhook" onClose={() => setShowCreate(false)}>
          <form onSubmit={create} className="form">
            <label>Name
              <input
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="My Webhook"
                required
              />
            </label>
            <label>URL
              <input
                value={form.url}
                onChange={(e) => setForm({ ...form, url: e.target.value })}
                placeholder="https://example.com/webhook"
                type="url"
                required
              />
            </label>
            <label>Authorization Header (optional)
              <input
                value={form.authHeader}
                onChange={(e) => setForm({ ...form, authHeader: e.target.value })}
                placeholder="Bearer your-token"
              />
            </label>
            <label>Event Types
              <div className="event-type-grid">
                {eventTypes.map((type) => (
                  <label key={type} className="checkbox-inline">
                    <input
                      type="checkbox"
                      checked={form.eventTypes.includes(type)}
                      onChange={() => toggleEventType(type)}
                    />
                    <span>{type}</span>
                  </label>
                ))}
              </div>
            </label>
            <div className="form-row">
              <label>Max Retries
                <input
                  type="number"
                  value={form.maxRetries}
                  onChange={(e) => setForm({ ...form, maxRetries: parseInt(e.target.value) })}
                  min="0"
                  max="10"
                />
              </label>
              <label>Timeout (seconds)
                <input
                  type="number"
                  value={form.timeoutSeconds}
                  onChange={(e) => setForm({ ...form, timeoutSeconds: parseInt(e.target.value) })}
                  min="1"
                  max="300"
                />
              </label>
            </div>
            <div className="form-actions">
              <button type="button" onClick={() => setShowCreate(false)}>Cancel</button>
              <button type="submit" className="btn-primary" disabled={form.eventTypes.length === 0}>Create</button>
            </div>
          </form>
        </Modal>
      )}

      {selected && (
        <Modal title={`Webhook: ${selected.name}`} onClose={() => setSelected(null)}>
          <div className="detail-grid">
            <div className="detail-row">
              <span className="detail-label">ID</span>
              <span className="mono">{selected.id}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Status</span>
              <span className={`status status-${selected.enabled ? 'green' : 'gray'}`}>
                {selected.enabled ? 'enabled' : 'disabled'}
              </span>
              <button
                onClick={() => toggleEnabled(selected)}
                className={`btn-toggle ${selected.enabled ? 'active' : ''}`}
              >
                {selected.enabled ? 'Disable' : 'Enable'}
              </button>
            </div>
            <div className="detail-row">
              <span className="detail-label">URL</span>
              <span className="dim">{selected.has_url ? '(encrypted)' : '—'}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Auth Header</span>
              <span className="dim">{selected.has_auth_header ? '(configured)' : 'none'}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Max Retries</span>
              <span>{selected.max_retries}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Timeout</span>
              <span>{selected.timeout_seconds}s</span>
            </div>
          </div>

          <div className="detail-section">
            <h4>Event Types</h4>
            <div className="tags">
              {selected.event_types?.map((e, i) => (
                <Tag key={i} variant="info">{e}</Tag>
              ))}
            </div>
          </div>

          <div className="detail-section">
            <h4>Recent Deliveries</h4>
            {deliveries.length === 0 ? (
              <div className="empty-small">No deliveries yet</div>
            ) : (
              <div className="table-wrap">
                <table className="table-compact">
                  <thead>
                    <tr>
                      <th>Event</th>
                      <th>Status</th>
                      <th>Attempts</th>
                      <th>Created</th>
                      <th>Error</th>
                    </tr>
                  </thead>
                  <tbody>
                    {deliveries.map((d) => (
                      <tr key={d.id}>
                        <td><Tag variant="info">{d.event_type}</Tag></td>
                        <td><Status status={d.status} /></td>
                        <td>{d.attempts}</td>
                        <td className="dim">{new Date(d.created_at).toLocaleString()}</td>
                        <td className="dim" title={d.last_error || ''}>
                          {d.last_error ? (d.last_error.slice(0, 30) + (d.last_error.length > 30 ? '...' : '')) : '—'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          <div className="form-actions">
            <button onClick={() => remove(selected.id)} className="btn-danger">Delete</button>
          </div>
        </Modal>
      )}
    </div>
  );
};

// ==================== METRICS PANEL ====================
const MetricsPanel = () => {
  const [metrics, setMetrics] = useState(null);
  const [parsedMetrics, setParsedMetrics] = useState(null);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const toast = useToast();

  const load = useCallback(async () => {
    try {
      const raw = await api.getMetrics();
      setMetrics(raw);
      setParsedMetrics(api.parseMetrics(raw));
    } catch (e) {
      toast?.('Failed to load metrics: ' + getErrorMessage(e), 'error');
    }
    setLoading(false);
  }, [toast]);

  useEffect(() => { load(); }, [load]);

  useEffect(() => {
    if (!autoRefresh) return;
    const interval = setInterval(load, 5000);
    return () => clearInterval(interval);
  }, [autoRefresh, load]);

  // Helper to get metric value
  const getMetricValue = (name, labels = {}) => {
    if (!parsedMetrics || !parsedMetrics[name]) return null;
    const match = parsedMetrics[name].find(m =>
      Object.entries(labels).every(([k, v]) => m.labels[k] === v)
    );
    return match ? match.value : null;
  };

  // Get all values for a metric
  const getMetricValues = (name) => parsedMetrics?.[name] || [];

  // Sum all values for a counter/gauge
  const sumMetric = (name) => {
    const values = getMetricValues(name);
    return values.reduce((sum, m) => sum + m.value, 0);
  };

  if (loading) return <div className="loading">Loading metrics...</div>;

  // Calculate summary stats
  const totalRequests = sumMetric('brokkr_http_requests_total');
  const activeAgents = getMetricValue('brokkr_active_agents') || 0;
  const totalStacks = getMetricValue('brokkr_stacks_total') || 0;
  const totalDeployments = getMetricValue('brokkr_deployment_objects_total') || 0;

  // Get request breakdown by endpoint
  const requestsByEndpoint = {};
  getMetricValues('brokkr_http_requests_total').forEach(m => {
    const endpoint = m.labels.endpoint || 'unknown';
    requestsByEndpoint[endpoint] = (requestsByEndpoint[endpoint] || 0) + m.value;
  });

  // Get agent heartbeat ages
  const heartbeatAges = getMetricValues('brokkr_agent_heartbeat_age_seconds');

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>📊 Metrics</h2>
        <div className="panel-actions">
          <label className="checkbox-inline">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            <span>Auto-refresh</span>
          </label>
          <button onClick={load} className="btn-icon">↻</button>
        </div>
      </div>

      <Section title="System Overview" icon="◈" defaultOpen>
        <div className="metrics-grid">
          <div className="metric-card">
            <div className="metric-value">{totalRequests.toLocaleString()}</div>
            <div className="metric-label">Total HTTP Requests</div>
          </div>
          <div className="metric-card">
            <div className="metric-value">{activeAgents}</div>
            <div className="metric-label">Active Agents</div>
          </div>
          <div className="metric-card">
            <div className="metric-value">{totalStacks}</div>
            <div className="metric-label">Total Stacks</div>
          </div>
          <div className="metric-card">
            <div className="metric-value">{totalDeployments}</div>
            <div className="metric-label">Deployment Objects</div>
          </div>
        </div>
      </Section>

      <Section title="HTTP Request Breakdown" icon="▶" defaultOpen>
        {Object.keys(requestsByEndpoint).length === 0 ? (
          <div className="empty-small">No HTTP requests recorded yet</div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Endpoint</th>
                  <th>Requests</th>
                </tr>
              </thead>
              <tbody>
                {Object.entries(requestsByEndpoint)
                  .sort((a, b) => b[1] - a[1])
                  .slice(0, 15)
                  .map(([endpoint, count]) => (
                    <tr key={endpoint}>
                      <td className="mono">{endpoint}</td>
                      <td>{count.toLocaleString()}</td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>
        )}
      </Section>

      <Section title="Agent Heartbeats" icon="♥">
        {heartbeatAges.length === 0 ? (
          <div className="empty-small">No agent heartbeat data</div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Agent</th>
                  <th>Last Heartbeat Age</th>
                </tr>
              </thead>
              <tbody>
                {heartbeatAges.map((m, i) => (
                  <tr key={i}>
                    <td className="mono">{m.labels.agent_name || m.labels.agent_id}</td>
                    <td>
                      <span className={m.value > 300 ? 'error-text' : m.value > 60 ? 'warning-text' : ''}>
                        {m.value.toFixed(1)}s
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Section>

      <Section title="Raw Metrics" icon="📄">
        <div className="metrics-raw">
          <pre className="code-block">{metrics?.slice(0, 5000)}{metrics?.length > 5000 && '\n... (truncated)'}</pre>
        </div>
      </Section>
    </div>
  );
};

// ==================== MAIN APP ====================
// Inner app component that uses toast context
const AppContent = () => {
  const [activePanel, setActivePanel] = useState('agents');
  const [stacks, setStacks] = useState([]);
  const [agents, setAgents] = useState([]);
  const [generators, setGenerators] = useState([]);
  const showToast = useToast();

  useEffect(() => {
    api.getGenerators().then(setGenerators).catch((e) => {
      showToast('Failed to load generators: ' + getErrorMessage(e), 'error');
    });
    api.getAgents().then(setAgents).catch((e) => {
      showToast('Failed to load agents: ' + getErrorMessage(e), 'error');
    });
  }, [showToast]);

  return (
    <div className="app">
      <header className="header">
        <div className="logo">
          <span className="logo-icon">◆</span>
          <span className="logo-text">BROKKR</span>
        </div>
        <nav className="nav">
          {['agents', 'stacks', 'templates', 'jobs', 'webhooks', 'metrics', 'admin'].map((p) => (
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
        {activePanel === 'webhooks' && <WebhooksPanel />}
        {activePanel === 'metrics' && <MetricsPanel />}
        {activePanel === 'admin' && <AdminPanel onGeneratorsChange={setGenerators} onAgentsChange={setAgents} />}
      </main>
    </div>
  );
};

// Main App with ToastProvider wrapper
export default function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}
