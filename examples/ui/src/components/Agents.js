import React, { useState, useEffect, useCallback } from 'react';
import {
  Box,
  Button,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  CircularProgress,
  Alert,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Chip,
  Stack,
  Tooltip,
  Switch,
  FormControlLabel,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Tabs,
  Tab
} from '@mui/material';
import {
  Refresh as RefreshIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Label as LabelIcon,
  Edit as EditIcon,
  Close as CloseIcon
} from '@mui/icons-material';
import {
  getAgents,
  getAgentLabels,
  getAgentAnnotations,
  addAgentLabel,
  removeAgentLabel,
  addAgentAnnotation,
  removeAgentAnnotation,
  updateAgent,
  getAgentTargets,
  addAgentTarget,
  removeAgentTarget,
  getStacks,
  getAgentEvents
} from '../services/api';
import { Link } from 'react-router-dom';

const AUTO_REFRESH_INTERVAL = 10000; // 10 seconds

// Custom TabPanel component
const TabPanel = ({ children, value, index }) => {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`tabpanel-${index}`}
      aria-labelledby={`tab-${index}`}
      style={{ padding: '20px 0' }}
    >
      {value === index && children}
    </div>
  );
};

// Add new EventsTab component
const EventsTab = ({ events, loadingEvents, onClose }) => {
  return (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Timestamp</TableCell>
            <TableCell>Event Type</TableCell>
            <TableCell>Status</TableCell>
            <TableCell>Deployment Object</TableCell>
            <TableCell>Message</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {loadingEvents ? (
            <TableRow>
              <TableCell colSpan={5} align="center">
                <CircularProgress />
              </TableCell>
            </TableRow>
          ) : (
            events.map((event, index) => (
              <TableRow key={index}>
                <TableCell>{new Date(event.created_at).toLocaleString()}</TableCell>
                <TableCell>{event.event_type}</TableCell>
                <TableCell>
                  <Chip
                    label={event.status}
                    size="small"
                    color={
                      event.status === 'SUCCESS' ? 'success' :
                      event.status === 'FAILURE' ? 'error' :
                      event.status === 'IN_PROGRESS' ? 'primary' : 'default'
                    }
                  />
                </TableCell>
                <TableCell>
                  {event.deployment_object_id && (
                    <Link
                      to={`/deployment-objects/${event.deployment_object_id}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        onClose();
                      }}
                      style={{ textDecoration: 'none', color: '#1976d2' }}
                    >
                      {event.deployment_object_id}
                    </Link>
                  )}
                </TableCell>
                <TableCell style={{ whiteSpace: 'pre-wrap', maxWidth: '400px' }}>
                  {event.message || '-'}
                </TableCell>
              </TableRow>
            ))
          )}
          {!loadingEvents && events.length === 0 && (
            <TableRow>
              <TableCell colSpan={5} align="center">
                <Typography color="text.secondary">No events recorded</Typography>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

const Agents = () => {
  const [agents, setAgents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState(null);
  const [agentDetails, setAgentDetails] = useState({});
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [labelDialogOpen, setLabelDialogOpen] = useState(false);
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [newLabel, setNewLabel] = useState('');
  const [newAnnotation, setNewAnnotation] = useState({ key: '', value: '' });
  const [success, setSuccess] = useState(null);
  const [stacks, setStacks] = useState([]);
  const [agentTargets, setAgentTargets] = useState({});
  const [selectedTab, setSelectedTab] = useState(0);
  const [targetDialogOpen, setTargetDialogOpen] = useState(false);
  const [selectedStack, setSelectedStack] = useState('');
  const [events, setEvents] = useState([]);
  const [loadingEvents, setLoadingEvents] = useState(false);

  const fetchAgentDetails = async (agentId) => {
    try {
      const [labels, annotations, targets, eventData] = await Promise.all([
        getAgentLabels(agentId),
        getAgentAnnotations(agentId),
        getAgentTargets(agentId),
        getAgentEvents(agentId)
      ]);
      setAgentDetails(prevDetails => ({
        ...prevDetails,
        [agentId]: { labels, annotations }
      }));
      setAgentTargets(prevTargets => ({
        ...prevTargets,
        [agentId]: targets
      }));
      setEvents(eventData);
    } catch (err) {
      console.error('Error fetching agent details:', err);
    }
  };

  useEffect(() => {
    const fetchAvailableStacks = async () => {
      try {
        const stacksData = await getStacks();
        setStacks(stacksData);
      } catch (err) {
        console.error('Error fetching stacks:', err);
      }
    };
    fetchAvailableStacks();
  }, []);

  const fetchAgents = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getAgents();
      setAgents(data);
      setError(null);

      // Fetch details for all agents
      await Promise.all(data.map(agent => fetchAgentDetails(agent.id)));
    } catch (err) {
      setError('Failed to fetch agents. Please check your admin PAK.');
      console.error('Error fetching agents:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  useEffect(() => {
    let interval;
    if (autoRefresh) {
      interval = setInterval(fetchAgents, AUTO_REFRESH_INTERVAL);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [autoRefresh, fetchAgents]);

  const handleRowClick = (agent) => {
    setSelectedAgent(agent);
    fetchAgentDetails(agent.id);
  };

  const handleCloseDialog = () => {
    setSelectedAgent(null);
    setAgentDetails({});
    setLabelDialogOpen(false);
    setAnnotationDialogOpen(false);
    setSuccess(null);
  };

  const handleAddLabel = async () => {
    if (!newLabel.trim()) return;
    try {
      await addAgentLabel(selectedAgent.id, newLabel.trim());
      await fetchAgentDetails(selectedAgent.id);
      setNewLabel('');
      setLabelDialogOpen(false);
      setSuccess('Label added successfully');
    } catch (err) {
      console.error('Error adding label:', err);
      setError('Failed to add label');
    }
  };

  const handleRemoveLabel = async (label) => {
    try {
      await removeAgentLabel(selectedAgent.id, label);
      await fetchAgentDetails(selectedAgent.id);
      setSuccess('Label removed successfully');
    } catch (err) {
      console.error('Error removing label:', err);
      setError('Failed to remove label');
    }
  };

  const handleAddAnnotation = async () => {
    if (!newAnnotation.key.trim() || !newAnnotation.value.trim()) return;
    try {
      await addAgentAnnotation(selectedAgent.id, newAnnotation.key.trim(), newAnnotation.value.trim());
      await fetchAgentDetails(selectedAgent.id);
      setNewAnnotation({ key: '', value: '' });
      setAnnotationDialogOpen(false);
      setSuccess('Annotation added successfully');
    } catch (err) {
      console.error('Error adding annotation:', err);
      setError('Failed to add annotation');
    }
  };

  const handleRemoveAnnotation = async (key) => {
    try {
      await removeAgentAnnotation(selectedAgent.id, key);
      await fetchAgentDetails(selectedAgent.id);
      setSuccess('Annotation removed successfully');
    } catch (err) {
      console.error('Error removing annotation:', err);
      setError('Failed to remove annotation');
    }
  };

  const handleStatusChange = async (status) => {
    try {
      await updateAgent(selectedAgent.id, { status });
      await fetchAgents();
      setSuccess('Agent status updated successfully');
    } catch (err) {
      console.error('Error updating agent status:', err);
      setError('Failed to update agent status');
    }
  };

  const handleAddTarget = async () => {
    if (!selectedStack) return;
    try {
      await addAgentTarget(selectedAgent.id, selectedStack);
      await fetchAgentDetails(selectedAgent.id);
      setSelectedStack('');
      setTargetDialogOpen(false);
      setSuccess('Stack target added successfully');
    } catch (err) {
      console.error('Error adding stack target:', err);
      setError('Failed to add stack target');
    }
  };

  const handleRemoveTarget = async (stackId) => {
    try {
      await removeAgentTarget(selectedAgent.id, stackId);
      await fetchAgentDetails(selectedAgent.id);
      setSuccess('Stack target removed successfully');
    } catch (err) {
      console.error('Error removing stack target:', err);
      setError('Failed to remove stack target');
    }
  };

  if (loading && !agents.length) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h5">
          Agents
        </Typography>
        <Box display="flex" alignItems="center">
          <FormControlLabel
            control={
              <Switch
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
              />
            }
            label="Auto-refresh"
          />
          <Tooltip title="Refresh agents">
            <IconButton onClick={fetchAgents} size="large">
              <RefreshIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      {!error && agents.length === 0 && (
        <Alert severity="info">
          No agents registered. Register an agent to get started.
        </Alert>
      )}

      {agents.length > 0 && (
        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>ID</TableCell>
                <TableCell>Name</TableCell>
                <TableCell>Cluster</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Labels</TableCell>
                <TableCell>Annotations</TableCell>
                <TableCell>Stack Targets</TableCell>
                <TableCell>Last Seen</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {agents.map((agent) => (
                <TableRow
                  key={agent.id}
                  onClick={() => handleRowClick(agent)}
                  hover
                  style={{ cursor: 'pointer' }}
                >
                  <TableCell>{agent.id}</TableCell>
                  <TableCell>{agent.name}</TableCell>
                  <TableCell>{agent.cluster_name}</TableCell>
                  <TableCell>
                    <Chip
                      label={agent.status}
                      color={agent.status === 'active' ? 'success' : 'error'}
                    />
                  </TableCell>
                  <TableCell>
                    {agentDetails[agent.id]?.labels?.map((labelObj) => (
                      <Chip
                        key={labelObj.id}
                        label={labelObj.label}
                        size="small"
                        style={{ margin: '2px' }}
                      />
                    ))}
                  </TableCell>
                  <TableCell>
                    {agentDetails[agent.id]?.annotations?.map((annotation) => (
                      <Chip
                        key={annotation.key}
                        label={`${annotation.key}=${annotation.value}`}
                        size="small"
                        style={{ margin: '2px' }}
                      />
                    ))}
                  </TableCell>
                  <TableCell>
                    {agentTargets[agent.id]?.map((target) => (
                      <Chip
                        key={target.stack_id}
                        label={stacks.find(s => s.id === target.stack_id)?.name || target.stack_id}
                        component={Link}
                        to={`/stacks/${target.stack_id}`}
                        onClick={(e) => e.stopPropagation()}
                        style={{ margin: '2px' }}
                      />
                    ))}
                  </TableCell>
                  <TableCell>{agent.last_heartbeat ? new Date(agent.last_heartbeat).toLocaleString() : 'Never'}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      <Dialog
        open={!!selectedAgent}
        onClose={handleCloseDialog}
        maxWidth="lg"
        fullWidth
      >
        {selectedAgent && (
          <>
            <DialogTitle>
              Agent Details: {selectedAgent.name}
              <IconButton
                aria-label="close"
                onClick={handleCloseDialog}
                sx={{ position: 'absolute', right: 8, top: 8 }}
              >
                <CloseIcon />
              </IconButton>
            </DialogTitle>
            <DialogContent>
              <Tabs value={selectedTab} onChange={(e, newValue) => setSelectedTab(newValue)}>
                <Tab label="Details" />
                <Tab label="Labels" />
                <Tab label="Annotations" />
                <Tab label="Stack Targets" />
                <Tab label="Events" />
              </Tabs>

              <TabPanel value={selectedTab} index={0}>
                <Box sx={{ mb: 3 }}>
                  <Typography variant="subtitle1" gutterBottom>Basic Information</Typography>
                  <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 2 }}>
                    <Typography><strong>Cluster:</strong> {selectedAgent?.cluster_name}</Typography>
                    <Typography><strong>Last Heartbeat:</strong> {selectedAgent?.last_heartbeat ? new Date(selectedAgent.last_heartbeat).toLocaleString() : 'Never'}</Typography>
                    <FormControl>
                      <InputLabel>Status</InputLabel>
                      <Select
                        value={selectedAgent?.status}
                        label="Status"
                        onChange={(e) => handleStatusChange(e.target.value)}
                        size="small"
                      >
                        <MenuItem value="ACTIVE">Active</MenuItem>
                        <MenuItem value="INACTIVE">Inactive</MenuItem>
                      </Select>
                    </FormControl>
                  </Box>
                </Box>
              </TabPanel>

              <TabPanel value={selectedTab} index={1}>
                <Box>
                  <Typography variant="h6" gutterBottom>
                    Labels
                    <IconButton size="small" onClick={() => setLabelDialogOpen(true)}>
                      <AddIcon />
                    </IconButton>
                  </Typography>
                  <Stack direction="row" spacing={1} mb={3}>
                    {agentDetails[selectedAgent?.id]?.labels?.map((labelObj) => (
                      <Chip
                        key={labelObj.id}
                        label={labelObj.label}
                        onDelete={() => handleRemoveLabel(labelObj.label)}
                        size="small"
                      />
                    ))}
                    {agentDetails[selectedAgent?.id]?.labels?.length === 0 && (
                      <Typography color="text.secondary">No labels</Typography>
                    )}
                  </Stack>
                </Box>
              </TabPanel>

              <TabPanel value={selectedTab} index={2}>
                <Box>
                  <Typography variant="h6" gutterBottom>
                    Annotations
                    <IconButton size="small" onClick={() => setAnnotationDialogOpen(true)}>
                      <AddIcon />
                    </IconButton>
                  </Typography>
                  <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                      <TableHead>
                        <TableRow>
                          <TableCell>Key</TableCell>
                          <TableCell>Value</TableCell>
                          <TableCell width={50}></TableCell>
                        </TableRow>
                      </TableHead>
                      <TableBody>
                        {agentDetails[selectedAgent?.id]?.annotations?.map((annotation) => (
                          <TableRow key={annotation.key}>
                            <TableCell>{annotation.key}</TableCell>
                            <TableCell>{annotation.value}</TableCell>
                            <TableCell>
                              <IconButton
                                size="small"
                                onClick={() => handleRemoveAnnotation(annotation.key)}
                              >
                                <DeleteIcon />
                              </IconButton>
                            </TableCell>
                          </TableRow>
                        ))}
                        {agentDetails[selectedAgent?.id]?.annotations?.length === 0 && (
                          <TableRow>
                            <TableCell colSpan={3} align="center">
                              <Typography color="text.secondary">No annotations</Typography>
                            </TableCell>
                          </TableRow>
                        )}
                      </TableBody>
                    </Table>
                  </TableContainer>
                </Box>
              </TabPanel>

              <TabPanel value={selectedTab} index={3}>
                <Box>
                  <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                    <Typography variant="h6">Stack Targets</Typography>
                    <Button
                      startIcon={<AddIcon />}
                      variant="contained"
                      onClick={() => setTargetDialogOpen(true)}
                    >
                      Add Target
                    </Button>
                  </Box>
                  {agentTargets[selectedAgent.id]?.map((target) => {
                    const stack = stacks.find(s => s.id === target.stack_id);
                    return (
                      <Box key={target.stack_id} mb={1} display="flex" alignItems="center">
                        <Button
                          component={Link}
                          to={`/stacks/${target.stack_id}`}
                          variant="outlined"
                          style={{ marginRight: '8px' }}
                        >
                          {stack?.name || target.stack_id}
                        </Button>
                        <IconButton
                          onClick={() => handleRemoveTarget(target.stack_id)}
                          size="small"
                          color="error"
                        >
                          <DeleteIcon />
                        </IconButton>
                      </Box>
                    );
                  })}
                </Box>
              </TabPanel>

              <TabPanel value={selectedTab} index={4}>
                <EventsTab events={events} loadingEvents={loadingEvents} onClose={handleCloseDialog} />
              </TabPanel>
            </DialogContent>
          </>
        )}
      </Dialog>

      <Dialog open={labelDialogOpen} onClose={() => setLabelDialogOpen(false)}>
        <DialogTitle>Add Label</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Label"
            fullWidth
            value={newLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            helperText="Labels must not contain whitespace"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setLabelDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleAddLabel} disabled={!newLabel.trim()}>Add</Button>
        </DialogActions>
      </Dialog>

      <Dialog open={annotationDialogOpen} onClose={() => setAnnotationDialogOpen(false)}>
        <DialogTitle>Add Annotation</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Key"
            fullWidth
            value={newAnnotation.key}
            onChange={(e) => setNewAnnotation({ ...newAnnotation, key: e.target.value })}
            helperText="Keys must not contain whitespace"
          />
          <TextField
            margin="dense"
            label="Value"
            fullWidth
            value={newAnnotation.value}
            onChange={(e) => setNewAnnotation({ ...newAnnotation, value: e.target.value })}
            helperText="Values must not contain whitespace"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAnnotationDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleAddAnnotation}
            disabled={!newAnnotation.key.trim() || !newAnnotation.value.trim()}
          >
            Add
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={targetDialogOpen} onClose={() => setTargetDialogOpen(false)}>
        <DialogTitle>Add Stack Target</DialogTitle>
        <DialogContent>
          <FormControl fullWidth sx={{ mt: 2 }}>
            <InputLabel>Stack</InputLabel>
            <Select
              value={selectedStack}
              onChange={(e) => setSelectedStack(e.target.value)}
              label="Stack"
            >
              {stacks
                .filter(stack => !agentTargets[selectedAgent?.id]?.some(target => target.stack_id === stack.id))
                .map((stack) => (
                  <MenuItem key={stack.id} value={stack.id}>
                    {stack.name}
                  </MenuItem>
                ))}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setTargetDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleAddTarget} disabled={!selectedStack}>
            Add
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default Agents;
