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
  FormControlLabel
} from '@mui/material';
import {
  Refresh as RefreshIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Label as LabelIcon,
  Edit as EditIcon
} from '@mui/icons-material';
import {
  getAgents,
  getAgentLabels,
  getAgentAnnotations,
  addAgentLabel,
  removeAgentLabel,
  addAgentAnnotation,
  removeAgentAnnotation
} from '../services/api';

const AUTO_REFRESH_INTERVAL = 10000; // 10 seconds

const Agents = () => {
  const [agents, setAgents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState(null);
  const [agentDetails, setAgentDetails] = useState({ labels: [], annotations: [] });
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [labelDialogOpen, setLabelDialogOpen] = useState(false);
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [newLabel, setNewLabel] = useState('');
  const [newAnnotation, setNewAnnotation] = useState({ key: '', value: '' });

  const fetchAgents = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getAgents();
      setAgents(data);
      setError(null);
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

  const fetchAgentDetails = async (agentId) => {
    setLoadingDetails(true);
    try {
      const [labels, annotations] = await Promise.all([
        getAgentLabels(agentId),
        getAgentAnnotations(agentId)
      ]);
      setAgentDetails({ labels, annotations });
    } catch (err) {
      console.error('Error fetching agent details:', err);
    } finally {
      setLoadingDetails(false);
    }
  };

  const handleRowClick = (agent) => {
    setSelectedAgent(agent);
    fetchAgentDetails(agent.id);
  };

  const handleCloseDialog = () => {
    setSelectedAgent(null);
    setAgentDetails({ labels: [], annotations: [] });
    setLabelDialogOpen(false);
    setAnnotationDialogOpen(false);
  };

  const handleAddLabel = async () => {
    if (!newLabel.trim()) return;
    try {
      await addAgentLabel(selectedAgent.id, newLabel.trim());
      await fetchAgentDetails(selectedAgent.id);
      setNewLabel('');
      setLabelDialogOpen(false);
    } catch (err) {
      console.error('Error adding label:', err);
    }
  };

  const handleRemoveLabel = async (label) => {
    try {
      await removeAgentLabel(selectedAgent.id, label);
      await fetchAgentDetails(selectedAgent.id);
    } catch (err) {
      console.error('Error removing label:', err);
    }
  };

  const handleAddAnnotation = async () => {
    if (!newAnnotation.key.trim() || !newAnnotation.value.trim()) return;
    try {
      await addAgentAnnotation(selectedAgent.id, newAnnotation.key.trim(), newAnnotation.value.trim());
      await fetchAgentDetails(selectedAgent.id);
      setNewAnnotation({ key: '', value: '' });
      setAnnotationDialogOpen(false);
    } catch (err) {
      console.error('Error adding annotation:', err);
    }
  };

  const handleRemoveAnnotation = async (key) => {
    try {
      await removeAgentAnnotation(selectedAgent.id, key);
      await fetchAgentDetails(selectedAgent.id);
    } catch (err) {
      console.error('Error removing annotation:', err);
    }
  };

  if (loading) {
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
                <TableCell>Name</TableCell>
                <TableCell>Cluster</TableCell>
                <TableCell>Labels</TableCell>
                <TableCell>Last Seen</TableCell>
                <TableCell>Status</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {agents.map((agent) => (
                <TableRow
                  key={agent.id}
                  hover
                  onClick={() => handleRowClick(agent)}
                  sx={{ cursor: 'pointer' }}
                >
                  <TableCell>{agent.name}</TableCell>
                  <TableCell>{agent.cluster_name}</TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={1}>
                      {agentDetails.labels
                        .filter(label => label.agent_id === agent.id)
                        .map((label) => (
                          <Chip
                            key={label}
                            label={label}
                            size="small"
                            onDelete={(e) => {
                              e.stopPropagation();
                              handleRemoveLabel(label);
                            }}
                          />
                        ))}
                    </Stack>
                  </TableCell>
                  <TableCell>{new Date(agent.last_seen_at).toLocaleString()}</TableCell>
                  <TableCell>{agent.status}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      <Dialog open={!!selectedAgent} onClose={handleCloseDialog} maxWidth="md" fullWidth>
        <DialogTitle>
          Agent Details: {selectedAgent?.name}
        </DialogTitle>
        <DialogContent>
          {loadingDetails ? (
            <Box display="flex" justifyContent="center" p={3}>
              <CircularProgress />
            </Box>
          ) : (
            <Box>
              <Typography variant="h6" gutterBottom>
                Labels
                <IconButton size="small" onClick={() => setLabelDialogOpen(true)}>
                  <AddIcon />
                </IconButton>
              </Typography>
              <Stack direction="row" spacing={1} mb={3}>
                {agentDetails.labels.map((label) => (
                  <Chip
                    key={label}
                    label={label}
                    onDelete={() => handleRemoveLabel(label)}
                    size="small"
                  />
                ))}
                {agentDetails.labels.length === 0 && (
                  <Typography color="text.secondary">No labels</Typography>
                )}
              </Stack>

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
                    {agentDetails.annotations.map((annotation) => (
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
                    {agentDetails.annotations.length === 0 && (
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
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCloseDialog}>Close</Button>
        </DialogActions>
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
    </Box>
  );
};

export default Agents;
