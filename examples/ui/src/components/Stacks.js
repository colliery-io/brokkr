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
  Fab,
  Tab,
  Tabs,
  MenuItem
} from '@mui/material';
import {
  Refresh as RefreshIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  FileCopy as CopyIcon
} from '@mui/icons-material';
import {
  getStacks,
  createStack,
  getStackLabels,
  getStackAnnotations,
  addStackLabel,
  removeStackLabel,
  addStackAnnotation,
  removeStackAnnotation,
  getStackDeploymentObjects,
  createDeploymentObject,
  getDeploymentObject,
  getGenerators,
  getAgents,
  getAgentTargets
} from '../services/api';
import Editor from "@monaco-editor/react";

const AUTO_REFRESH_INTERVAL = 10000; // 10 seconds

const TabPanel = ({ children, value, index }) => (
  <div hidden={value !== index} style={{ padding: '20px 0' }}>
    {value === index && children}
  </div>
);

const Stacks = () => {
  const [stacks, setStacks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selectedStack, setSelectedStack] = useState(null);
  const [stackDetails, setStackDetails] = useState({});
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [labelDialogOpen, setLabelDialogOpen] = useState(false);
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [deploymentDialogOpen, setDeploymentDialogOpen] = useState(false);
  const [newLabel, setNewLabel] = useState('');
  const [newAnnotation, setNewAnnotation] = useState({ key: '', value: '' });
  const [newStack, setNewStack] = useState({
    name: '',
    description: '',
    generatorId: ''
  });
  const [success, setSuccess] = useState(null);
  const [selectedTab, setSelectedTab] = useState(0);
  const [deploymentObjects, setDeploymentObjects] = useState([]);
  const [yamlContent, setYamlContent] = useState('');
  const [isDeletionMarker, setIsDeletionMarker] = useState(false);
  const [generators, setGenerators] = useState([]);
  const [agents, setAgents] = useState([]);
  const [targetingAgents, setTargetingAgents] = useState([]);

  const fetchStackDetails = async (stackId) => {
    try {
      const [labels, annotations, deployments, agents] = await Promise.all([
        getStackLabels(stackId),
        getStackAnnotations(stackId),
        getStackDeploymentObjects(stackId),
        getAgents()
      ]);

      // Get all agent targets and filter for this stack
      const agentTargets = await Promise.all(
        agents.map(async agent => {
          const targets = await getAgentTargets(agent.id);
          return { agent, targets };
        })
      );

      const targeting = agentTargets
        .filter(({ targets }) => targets.some(target => target.stack_id === stackId))
        .map(({ agent }) => agent);

      setTargetingAgents(targeting);
      setStackDetails(prevDetails => ({
        ...prevDetails,
        [stackId]: { labels, annotations }
      }));
      setDeploymentObjects(deployments);
    } catch (err) {
      console.error('Error fetching stack details:', err);
    }
  };

  const fetchStacks = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getStacks();
      setStacks(data);
      setError(null);

      // Fetch details for all stacks
      await Promise.all(data.map(stack => fetchStackDetails(stack.id)));
    } catch (err) {
      setError('Failed to fetch stacks. Please check your admin PAK.');
      console.error('Error fetching stacks:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStacks();
  }, [fetchStacks]);

  useEffect(() => {
    let interval;
    if (autoRefresh) {
      interval = setInterval(fetchStacks, AUTO_REFRESH_INTERVAL);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [autoRefresh, fetchStacks]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [generatorsData, agentsData] = await Promise.all([
          getGenerators(),
          getAgents()
        ]);
        setGenerators(generatorsData);
        setAgents(agentsData);
        if (generatorsData.length > 0) {
          setNewStack(prev => ({ ...prev, generatorId: generatorsData[0].id }));
        }
      } catch (err) {
        console.error('Error fetching data:', err);
      }
    };
    fetchData();
  }, []);

  const handleRowClick = (stack) => {
    setSelectedStack(stack);
    fetchStackDetails(stack.id);
  };

  const handleCloseDialog = () => {
    setSelectedStack(null);
    setStackDetails({});
    setLabelDialogOpen(false);
    setAnnotationDialogOpen(false);
    setDeploymentDialogOpen(false);
    setSuccess(null);
    setSelectedTab(0);
    setYamlContent('');
    setIsDeletionMarker(false);
  };

  const handleCreateStack = async () => {
    if (!newStack.name.trim() || !newStack.generatorId) return;
    try {
      setLoading(true);
      await createStack(
        newStack.name.trim(),
        newStack.description.trim() || null,
        newStack.generatorId
      );
      setNewStack({
        name: '',
        description: '',
        generatorId: generators[0]?.id || ''
      });
      setCreateDialogOpen(false);
      setSuccess('Stack created successfully');
      await fetchStacks();
    } catch (err) {
      setError('Failed to create stack. Please check your input and try again.');
      console.error('Error creating stack:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddLabel = async () => {
    if (!newLabel.trim()) return;
    try {
      await addStackLabel(selectedStack.id, newLabel.trim());
      await fetchStackDetails(selectedStack.id);
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
      await removeStackLabel(selectedStack.id, label);
      await fetchStackDetails(selectedStack.id);
      setSuccess('Label removed successfully');
    } catch (err) {
      console.error('Error removing label:', err);
      setError('Failed to remove label');
    }
  };

  const handleAddAnnotation = async () => {
    if (!newAnnotation.key.trim() || !newAnnotation.value.trim()) return;
    try {
      await addStackAnnotation(selectedStack.id, newAnnotation.key.trim(), newAnnotation.value.trim());
      await fetchStackDetails(selectedStack.id);
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
      await removeStackAnnotation(selectedStack.id, key);
      await fetchStackDetails(selectedStack.id);
      setSuccess('Annotation removed successfully');
    } catch (err) {
      console.error('Error removing annotation:', err);
      setError('Failed to remove annotation');
    }
  };

  const handleEditLastDeployment = async () => {
    if (deploymentObjects.length === 0) return;

    try {
      const lastDeployment = deploymentObjects[deploymentObjects.length - 1];
      const deploymentDetails = await getDeploymentObject(lastDeployment.id);
      setYamlContent(deploymentDetails.yaml_content);
      setIsDeletionMarker(deploymentDetails.is_deletion_marker);
      setDeploymentDialogOpen(true);
    } catch (err) {
      console.error('Error fetching last deployment:', err);
      setError('Failed to fetch last deployment');
    }
  };

  const validateYaml = (content) => {
    try {
      // Use js-yaml or another YAML parser to validate
      return true;
    } catch (err) {
      return false;
    }
  };

  const handleCreateDeployment = async () => {
    if (!yamlContent.trim()) {
      setError('YAML content cannot be empty');
      return;
    }

    if (!validateYaml(yamlContent)) {
      setError('Invalid YAML content');
      return;
    }

    try {
      setLoading(true);
      await createDeploymentObject(selectedStack.id, yamlContent, isDeletionMarker);
      await fetchStackDetails(selectedStack.id);
      setDeploymentDialogOpen(false);
      setSuccess('Deployment object created successfully');
      setYamlContent('');
      setIsDeletionMarker(false);
    } catch (err) {
      console.error('Error creating deployment:', err);
      setError(err.response?.data?.message || 'Failed to create deployment object');
    } finally {
      setLoading(false);
    }
  };

  if (loading && !stacks.length) {
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
          Stacks
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
          <Tooltip title="Refresh stacks">
            <IconButton onClick={fetchStacks} size="large">
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

      {!error && stacks.length === 0 && (
        <Alert severity="info">
          No stacks found. Create a stack to get started.
        </Alert>
      )}

      {stacks.length > 0 && (
        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Description</TableCell>
                <TableCell>Labels</TableCell>
                <TableCell>Annotations</TableCell>
                <TableCell>Last Deployment</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {stacks.map((stack) => (
                <TableRow
                  key={stack.id}
                  hover
                  onClick={() => handleRowClick(stack)}
                  sx={{ cursor: 'pointer' }}
                >
                  <TableCell>{stack.name}</TableCell>
                  <TableCell>{stack.description || '-'}</TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={1}>
                      {stackDetails[stack.id]?.labels?.map((labelObj) => (
                        <Chip
                          key={labelObj.id}
                          label={labelObj.label}
                          size="small"
                        />
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={1}>
                      {stackDetails[stack.id]?.annotations?.map((annotation) => (
                        <Tooltip
                          key={annotation.key}
                          title={`${annotation.key}: ${annotation.value}`}
                        >
                          <Chip
                            label={annotation.key}
                            size="small"
                          />
                        </Tooltip>
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell>
                    {deploymentObjects.length > 0 ? (
                      <Tooltip title="Click to edit last deployment">
                        <IconButton
                          size="small"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleEditLastDeployment();
                          }}
                        >
                          <EditIcon />
                        </IconButton>
                      </Tooltip>
                    ) : (
                      '-'
                    )}
                  </TableCell>
                  <TableCell>
                    <Tooltip title="Create new deployment">
                      <IconButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedStack(stack);
                          setDeploymentDialogOpen(true);
                        }}
                      >
                        <AddIcon />
                      </IconButton>
                    </Tooltip>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      <Fab
        color="primary"
        aria-label="add"
        onClick={() => setCreateDialogOpen(true)}
        sx={{ position: 'fixed', bottom: 16, right: 16 }}
      >
        <AddIcon />
      </Fab>

      {/* Stack Details Dialog */}
      <Dialog open={!!selectedStack} onClose={handleCloseDialog} maxWidth="md" fullWidth>
        <DialogTitle>
          Stack Details: {selectedStack?.name}
        </DialogTitle>
        <DialogContent>
          <Tabs value={selectedTab} onChange={(e, newValue) => setSelectedTab(newValue)}>
            <Tab label="Details" />
            <Tab label="Labels" />
            <Tab label="Annotations" />
            <Tab label="Deployments" />
            <Tab label="Targeting Agents" />
          </Tabs>

          <TabPanel value={selectedTab} index={0}>
            <Box>
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle1" gutterBottom>Basic Information</Typography>
                <Typography><strong>Description:</strong> {selectedStack?.description || '-'}</Typography>
              </Box>

              <Typography variant="h6" gutterBottom>
                Labels
                <IconButton size="small" onClick={() => setLabelDialogOpen(true)}>
                  <AddIcon />
                </IconButton>
              </Typography>
              <Stack direction="row" spacing={1} mb={3}>
                {stackDetails[selectedStack?.id]?.labels?.map((labelObj) => (
                  <Chip
                    key={labelObj.id}
                    label={labelObj.label}
                    onDelete={() => handleRemoveLabel(labelObj.label)}
                    size="small"
                  />
                ))}
                {stackDetails[selectedStack?.id]?.labels?.length === 0 && (
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
                    {stackDetails[selectedStack?.id]?.annotations?.map((annotation) => (
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
                    {stackDetails[selectedStack?.id]?.annotations?.length === 0 && (
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

          <TabPanel value={selectedTab} index={1}>
            <Box>
              <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Typography variant="h6">Deployment Objects</Typography>
                <Button
                  variant="contained"
                  startIcon={<AddIcon />}
                  onClick={() => setDeploymentDialogOpen(true)}
                >
                  Create Deployment
                </Button>
              </Box>

              <TableContainer component={Paper} variant="outlined">
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Sequence ID</TableCell>
                      <TableCell>Created At</TableCell>
                      <TableCell>Type</TableCell>
                      <TableCell>Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {deploymentObjects.map((deployment) => (
                      <TableRow key={deployment.id}>
                        <TableCell>{deployment.sequence_id}</TableCell>
                        <TableCell>{new Date(deployment.created_at).toLocaleString()}</TableCell>
                        <TableCell>
                          <Chip
                            label={deployment.is_deletion_marker ? 'Deletion' : 'Deployment'}
                            color={deployment.is_deletion_marker ? 'error' : 'primary'}
                            size="small"
                          />
                        </TableCell>
                        <TableCell>
                          <Tooltip title="Copy as new deployment">
                            <IconButton
                              size="small"
                              onClick={async () => {
                                const details = await getDeploymentObject(deployment.id);
                                setYamlContent(details.yaml_content);
                                setIsDeletionMarker(details.is_deletion_marker);
                                setDeploymentDialogOpen(true);
                              }}
                            >
                              <CopyIcon />
                            </IconButton>
                          </Tooltip>
                        </TableCell>
                      </TableRow>
                    ))}
                    {deploymentObjects.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={4} align="center">
                          <Typography color="text.secondary">No deployment objects</Typography>
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          </TabPanel>

          <TabPanel value={selectedTab} index={2}>
            <Box>
              <Typography variant="h6" gutterBottom>
                Agents Targeting This Stack
              </Typography>

              <TableContainer component={Paper} variant="outlined">
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Agent Name</TableCell>
                      <TableCell>Cluster</TableCell>
                      <TableCell>Status</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {targetingAgents.map((agent) => (
                      <TableRow key={agent.id}>
                        <TableCell>{agent.name}</TableCell>
                        <TableCell>{agent.cluster_name}</TableCell>
                        <TableCell>
                          <Chip
                            label={agent.status}
                            color={agent.status === 'ACTIVE' ? 'success' : 'default'}
                            size="small"
                          />
                        </TableCell>
                      </TableRow>
                    ))}
                    {targetingAgents.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          <Typography color="text.secondary">
                            No agents are targeting this stack
                          </Typography>
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          </TabPanel>

          <TabPanel value={selectedTab} index={4}>
            <Box>
              <Typography variant="h6" gutterBottom>
                Agents Targeting This Stack
              </Typography>

              <TableContainer component={Paper} variant="outlined">
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Agent Name</TableCell>
                      <TableCell>Cluster</TableCell>
                      <TableCell>Status</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {targetingAgents.map((agent) => (
                      <TableRow key={agent.id}>
                        <TableCell>{agent.name}</TableCell>
                        <TableCell>{agent.cluster_name}</TableCell>
                        <TableCell>
                          <Chip
                            label={agent.status}
                            color={agent.status === 'ACTIVE' ? 'success' : 'default'}
                            size="small"
                          />
                        </TableCell>
                      </TableRow>
                    ))}
                    {targetingAgents.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          <Typography color="text.secondary">
                            No agents are targeting this stack
                          </Typography>
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          </TabPanel>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCloseDialog}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Create Stack Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)}>
        <DialogTitle>Create New Stack</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 2 }}>
            <TextField
              label="Name"
              value={newStack.name}
              onChange={(e) => setNewStack({ ...newStack, name: e.target.value })}
              fullWidth
              required
            />
            <TextField
              label="Description"
              value={newStack.description}
              onChange={(e) => setNewStack({ ...newStack, description: e.target.value })}
              fullWidth
              multiline
              rows={3}
            />
            <TextField
              select
              label="Generator"
              value={newStack.generatorId}
              onChange={(e) => setNewStack({ ...newStack, generatorId: e.target.value })}
              fullWidth
              required
            >
              {generators.map((generator) => (
                <MenuItem key={generator.id} value={generator.id}>
                  {generator.name}
                </MenuItem>
              ))}
            </TextField>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateStack} variant="contained" disabled={!newStack.name || !newStack.generatorId}>
            Create
          </Button>
        </DialogActions>
      </Dialog>

      {/* Add Label Dialog */}
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

      {/* Add Annotation Dialog */}
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

      {/* Create/Edit Deployment Dialog */}
      <Dialog open={deploymentDialogOpen} onClose={() => setDeploymentDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>
          {yamlContent ? 'Edit Deployment' : 'Create Deployment'}
        </DialogTitle>
        <DialogContent>
          <Editor
            height="400px"
            defaultLanguage="yaml"
            value={yamlContent}
            onChange={setYamlContent}
            options={{
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              wordWrap: "on"
            }}
          />
          <FormControlLabel
            control={
              <Switch
                checked={isDeletionMarker}
                onChange={(e) => setIsDeletionMarker(e.target.checked)}
              />
            }
            label="Mark as deletion"
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeploymentDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleCreateDeployment}
            disabled={!yamlContent.trim()}
            variant="contained"
          >
            Submit
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default Stacks;
