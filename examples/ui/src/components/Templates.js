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
  PlayArrow as InstantiateIcon
} from '@mui/icons-material';
import {
  getTemplates,
  createTemplate,
  updateTemplate,
  deleteTemplate,
  getTemplateLabels,
  getTemplateAnnotations,
  addTemplateLabel,
  removeTemplateLabel,
  addTemplateAnnotation,
  removeTemplateAnnotation,
  instantiateTemplate,
  getStacks,
  getStackLabels,
  getStackAnnotations
} from '../services/api';
import Editor from "@monaco-editor/react";

const AUTO_REFRESH_INTERVAL = 10000;

const TabPanel = ({ children, value, index }) => (
  <div hidden={value !== index} style={{ padding: '20px 0' }}>
    {value === index && children}
  </div>
);

const DEFAULT_TEMPLATE_CONTENT = `# Tera template (Jinja2-like syntax)
# Use {{ variable }} to reference parameters
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}
  namespace: {{ namespace | default(value="default") }}
data:
  key: {{ value }}`;

const DEFAULT_PARAMETERS_SCHEMA = `{
  "type": "object",
  "required": ["name", "value"],
  "properties": {
    "name": {
      "type": "string",
      "description": "Name of the ConfigMap"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace",
      "default": "default"
    },
    "value": {
      "type": "string",
      "description": "Value to store in the ConfigMap"
    }
  }
}`;

const Templates = () => {
  const [templates, setTemplates] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selectedTemplate, setSelectedTemplate] = useState(null);
  const [templateDetails, setTemplateDetails] = useState({});
  const [labelDialogOpen, setLabelDialogOpen] = useState(false);
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [instantiateDialogOpen, setInstantiateDialogOpen] = useState(false);
  const [newLabel, setNewLabel] = useState('');
  const [newAnnotation, setNewAnnotation] = useState({ key: '', value: '' });
  const [newTemplate, setNewTemplate] = useState({
    name: '',
    description: '',
    templateContent: DEFAULT_TEMPLATE_CONTENT,
    parametersSchema: DEFAULT_PARAMETERS_SCHEMA
  });
  const [success, setSuccess] = useState(null);
  const [selectedTab, setSelectedTab] = useState(0);
  const [stacks, setStacks] = useState([]);
  const [selectedStackId, setSelectedStackId] = useState('');
  const [instantiateParams, setInstantiateParams] = useState('{}');
  const [stackDetails, setStackDetails] = useState({});

  const fetchTemplateDetails = async (templateId) => {
    try {
      const [labels, annotations] = await Promise.all([
        getTemplateLabels(templateId),
        getTemplateAnnotations(templateId)
      ]);

      setTemplateDetails(prevDetails => ({
        ...prevDetails,
        [templateId]: { labels, annotations }
      }));
    } catch (err) {
      console.error('Error fetching template details:', err);
    }
  };

  const fetchTemplates = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getTemplates();
      setTemplates(data);
      setError(null);

      await Promise.all(data.map(template => fetchTemplateDetails(template.id)));
    } catch (err) {
      setError('Failed to fetch templates. Please check your admin PAK.');
      console.error('Error fetching templates:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTemplates();
  }, [fetchTemplates]);

  useEffect(() => {
    let interval;
    if (autoRefresh) {
      interval = setInterval(fetchTemplates, AUTO_REFRESH_INTERVAL);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [autoRefresh, fetchTemplates]);

  useEffect(() => {
    const fetchStacks = async () => {
      try {
        const stacksData = await getStacks();
        setStacks(stacksData);

        // Fetch labels and annotations for each stack
        const details = {};
        await Promise.all(stacksData.map(async (stack) => {
          const [labels, annotations] = await Promise.all([
            getStackLabels(stack.id),
            getStackAnnotations(stack.id)
          ]);
          details[stack.id] = { labels, annotations };
        }));
        setStackDetails(details);
      } catch (err) {
        console.error('Error fetching stacks:', err);
      }
    };
    fetchStacks();
  }, []);

  const handleRowClick = (template) => {
    setSelectedTemplate(template);
    fetchTemplateDetails(template.id);
  };

  const handleCloseDialog = () => {
    setSelectedTemplate(null);
    setLabelDialogOpen(false);
    setAnnotationDialogOpen(false);
    setSuccess(null);
    setSelectedTab(0);
  };

  const handleCreateTemplate = async () => {
    if (!newTemplate.name.trim() || !newTemplate.templateContent.trim()) return;
    try {
      setLoading(true);
      await createTemplate(
        newTemplate.name.trim(),
        newTemplate.description.trim() || null,
        newTemplate.templateContent,
        newTemplate.parametersSchema
      );
      setNewTemplate({
        name: '',
        description: '',
        templateContent: DEFAULT_TEMPLATE_CONTENT,
        parametersSchema: DEFAULT_PARAMETERS_SCHEMA
      });
      setCreateDialogOpen(false);
      setSuccess('Template created successfully');
      await fetchTemplates();
    } catch (err) {
      setError(err.response?.data?.error || 'Failed to create template. Please check your input.');
      console.error('Error creating template:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateTemplate = async () => {
    if (!selectedTemplate) return;
    try {
      setLoading(true);
      await updateTemplate(
        selectedTemplate.id,
        selectedTemplate.description,
        selectedTemplate.template_content,
        selectedTemplate.parameters_schema
      );
      setSuccess('Template updated (new version created)');
      await fetchTemplates();
      handleCloseDialog();
    } catch (err) {
      setError(err.response?.data?.error || 'Failed to update template.');
      console.error('Error updating template:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteTemplate = async (id) => {
    if (!window.confirm('Are you sure you want to delete this template?')) return;
    try {
      setLoading(true);
      await deleteTemplate(id);
      setSuccess('Template deleted successfully');
      handleCloseDialog();
      await fetchTemplates();
    } catch (err) {
      setError('Failed to delete template');
      console.error('Error deleting template:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddLabel = async () => {
    if (!newLabel.trim()) return;
    try {
      await addTemplateLabel(selectedTemplate.id, newLabel.trim());
      await fetchTemplateDetails(selectedTemplate.id);
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
      await removeTemplateLabel(selectedTemplate.id, label);
      await fetchTemplateDetails(selectedTemplate.id);
      setSuccess('Label removed successfully');
    } catch (err) {
      console.error('Error removing label:', err);
      setError('Failed to remove label');
    }
  };

  const handleAddAnnotation = async () => {
    if (!newAnnotation.key.trim() || !newAnnotation.value.trim()) return;
    try {
      await addTemplateAnnotation(selectedTemplate.id, newAnnotation.key.trim(), newAnnotation.value.trim());
      await fetchTemplateDetails(selectedTemplate.id);
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
      await removeTemplateAnnotation(selectedTemplate.id, key);
      await fetchTemplateDetails(selectedTemplate.id);
      setSuccess('Annotation removed successfully');
    } catch (err) {
      console.error('Error removing annotation:', err);
      setError('Failed to remove annotation');
    }
  };

  const handleInstantiate = async () => {
    if (!selectedStackId || !selectedTemplate) return;
    try {
      setLoading(true);
      const params = JSON.parse(instantiateParams);
      await instantiateTemplate(selectedStackId, selectedTemplate.id, params);
      setSuccess('Template instantiated successfully! Deployment object created.');
      setInstantiateDialogOpen(false);
      setInstantiateParams('{}');
      setSelectedStackId('');
    } catch (err) {
      const errorMessage = err.response?.data?.error ||
        err.response?.data?.validation_errors?.join(', ') ||
        'Failed to instantiate template';
      setError(errorMessage);
      console.error('Error instantiating template:', err);
    } finally {
      setLoading(false);
    }
  };

  const checkLabelMatch = (stackId) => {
    if (!selectedTemplate || !templateDetails[selectedTemplate.id]) return { matches: true, missing: [] };

    const templateLabels = templateDetails[selectedTemplate.id].labels?.map(l => l.label) || [];
    const stackLabelsSet = new Set(stackDetails[stackId]?.labels?.map(l => l.label) || []);

    const missing = templateLabels.filter(label => !stackLabelsSet.has(label));
    return { matches: missing.length === 0, missing };
  };

  if (loading && !templates.length) {
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
          Templates
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
          <Tooltip title="Refresh templates">
            <IconButton onClick={fetchTemplates} size="large">
              <RefreshIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      {!error && templates.length === 0 && (
        <Alert severity="info">
          No templates found. Create a template to get started.
        </Alert>
      )}

      {templates.length > 0 && (
        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Version</TableCell>
                <TableCell>Description</TableCell>
                <TableCell>Labels</TableCell>
                <TableCell>Type</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {templates.map((template) => (
                <TableRow
                  key={template.id}
                  hover
                  onClick={() => handleRowClick(template)}
                  sx={{ cursor: 'pointer' }}
                >
                  <TableCell>{template.name}</TableCell>
                  <TableCell>
                    <Chip label={`v${template.version}`} size="small" color="primary" />
                  </TableCell>
                  <TableCell>{template.description || '-'}</TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={1} flexWrap="wrap">
                      {templateDetails[template.id]?.labels?.map((labelObj) => (
                        <Chip
                          key={labelObj.id}
                          label={labelObj.label}
                          size="small"
                          sx={{ margin: '2px' }}
                        />
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={template.generator_id ? 'Generator' : 'System'}
                      size="small"
                      color={template.generator_id ? 'default' : 'secondary'}
                    />
                  </TableCell>
                  <TableCell>
                    <Tooltip title="Instantiate template">
                      <IconButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedTemplate(template);
                          fetchTemplateDetails(template.id);
                          setInstantiateDialogOpen(true);
                        }}
                      >
                        <InstantiateIcon />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Delete template">
                      <IconButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteTemplate(template.id);
                        }}
                      >
                        <DeleteIcon />
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

      {/* Template Details Dialog */}
      <Dialog open={!!selectedTemplate && !instantiateDialogOpen} onClose={handleCloseDialog} maxWidth="lg" fullWidth>
        <DialogTitle>
          Template: {selectedTemplate?.name} (v{selectedTemplate?.version})
        </DialogTitle>
        <DialogContent>
          <Tabs value={selectedTab} onChange={(e, newValue) => setSelectedTab(newValue)}>
            <Tab label="Details" />
            <Tab label="Template Content" />
            <Tab label="Parameters Schema" />
            <Tab label="Labels" />
            <Tab label="Annotations" />
          </Tabs>

          <TabPanel value={selectedTab} index={0}>
            <Box>
              <Typography><strong>ID:</strong> {selectedTemplate?.id}</Typography>
              <Typography><strong>Name:</strong> {selectedTemplate?.name}</Typography>
              <Typography><strong>Version:</strong> {selectedTemplate?.version}</Typography>
              <Typography><strong>Description:</strong> {selectedTemplate?.description || '-'}</Typography>
              <Typography><strong>Type:</strong> {selectedTemplate?.generator_id ? 'Generator Template' : 'System Template'}</Typography>
              <Typography><strong>Created:</strong> {selectedTemplate?.created_at ? new Date(selectedTemplate.created_at).toLocaleString() : '-'}</Typography>
              <Typography><strong>Checksum:</strong> <code>{selectedTemplate?.checksum}</code></Typography>
            </Box>
          </TabPanel>

          <TabPanel value={selectedTab} index={1}>
            <Typography variant="subtitle2" gutterBottom>
              Tera Template (Jinja2-like syntax)
            </Typography>
            <Editor
              height="400px"
              defaultLanguage="yaml"
              value={selectedTemplate?.template_content || ''}
              onChange={(value) => setSelectedTemplate(prev => ({ ...prev, template_content: value }))}
              options={{
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on"
              }}
            />
          </TabPanel>

          <TabPanel value={selectedTab} index={2}>
            <Typography variant="subtitle2" gutterBottom>
              JSON Schema for Parameters
            </Typography>
            <Editor
              height="400px"
              defaultLanguage="json"
              value={selectedTemplate?.parameters_schema || '{}'}
              onChange={(value) => setSelectedTemplate(prev => ({ ...prev, parameters_schema: value }))}
              options={{
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on"
              }}
            />
          </TabPanel>

          <TabPanel value={selectedTab} index={3}>
            <Box>
              <Typography variant="h6" gutterBottom>
                Labels
                <IconButton size="small" onClick={() => setLabelDialogOpen(true)}>
                  <AddIcon />
                </IconButton>
              </Typography>
              <Typography variant="body2" color="text.secondary" gutterBottom>
                Templates with labels can only be instantiated on stacks that have matching labels.
              </Typography>
              <Stack direction="row" spacing={1} mb={3} flexWrap="wrap">
                {templateDetails[selectedTemplate?.id]?.labels?.map((labelObj) => (
                  <Chip
                    key={labelObj.id}
                    label={labelObj.label}
                    onDelete={() => handleRemoveLabel(labelObj.label)}
                    size="small"
                    sx={{ margin: '2px' }}
                  />
                ))}
                {templateDetails[selectedTemplate?.id]?.labels?.length === 0 && (
                  <Typography color="text.secondary">No labels (template matches all stacks)</Typography>
                )}
              </Stack>
            </Box>
          </TabPanel>

          <TabPanel value={selectedTab} index={4}>
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
                    {templateDetails[selectedTemplate?.id]?.annotations?.map((annotation) => (
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
                    {templateDetails[selectedTemplate?.id]?.annotations?.length === 0 && (
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
        </DialogContent>
        <DialogActions>
          <Button onClick={() => handleDeleteTemplate(selectedTemplate?.id)} color="error">
            Delete
          </Button>
          <Button onClick={handleCloseDialog}>Close</Button>
          <Button onClick={handleUpdateTemplate} variant="contained">
            Save as New Version
          </Button>
        </DialogActions>
      </Dialog>

      {/* Create Template Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="lg" fullWidth>
        <DialogTitle>Create New Template</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 2 }}>
            <TextField
              label="Name"
              value={newTemplate.name}
              onChange={(e) => setNewTemplate({ ...newTemplate, name: e.target.value })}
              fullWidth
              required
              helperText="Template name (must be unique)"
            />
            <TextField
              label="Description"
              value={newTemplate.description}
              onChange={(e) => setNewTemplate({ ...newTemplate, description: e.target.value })}
              fullWidth
              multiline
              rows={2}
            />
            <Typography variant="subtitle2">Template Content (Tera/Jinja2 syntax)</Typography>
            <Editor
              height="250px"
              defaultLanguage="yaml"
              value={newTemplate.templateContent}
              onChange={(value) => setNewTemplate({ ...newTemplate, templateContent: value })}
              options={{
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on"
              }}
            />
            <Typography variant="subtitle2">Parameters Schema (JSON Schema)</Typography>
            <Editor
              height="200px"
              defaultLanguage="json"
              value={newTemplate.parametersSchema}
              onChange={(value) => setNewTemplate({ ...newTemplate, parametersSchema: value })}
              options={{
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on"
              }}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleCreateTemplate}
            variant="contained"
            disabled={!newTemplate.name || !newTemplate.templateContent}
          >
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
            helperText="Labels must not contain whitespace. Templates with labels only match stacks with those labels."
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
          />
          <TextField
            margin="dense"
            label="Value"
            fullWidth
            value={newAnnotation.value}
            onChange={(e) => setNewAnnotation({ ...newAnnotation, value: e.target.value })}
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

      {/* Instantiate Template Dialog */}
      <Dialog open={instantiateDialogOpen} onClose={() => setInstantiateDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Instantiate Template: {selectedTemplate?.name}</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 2 }}>
            <TextField
              select
              label="Target Stack"
              value={selectedStackId}
              onChange={(e) => setSelectedStackId(e.target.value)}
              fullWidth
              required
              helperText="Select a stack to create a deployment object from this template"
            >
              {stacks.map((stack) => {
                const { matches, missing } = checkLabelMatch(stack.id);
                return (
                  <MenuItem
                    key={stack.id}
                    value={stack.id}
                    disabled={!matches}
                  >
                    <Box display="flex" alignItems="center" width="100%">
                      <span>{stack.name}</span>
                      {!matches && (
                        <Chip
                          label={`Missing: ${missing.join(', ')}`}
                          size="small"
                          color="error"
                          sx={{ ml: 1 }}
                        />
                      )}
                      {matches && templateDetails[selectedTemplate?.id]?.labels?.length > 0 && (
                        <Chip
                          label="Labels match"
                          size="small"
                          color="success"
                          sx={{ ml: 1 }}
                        />
                      )}
                    </Box>
                  </MenuItem>
                );
              })}
            </TextField>

            <Typography variant="subtitle2">Parameters (JSON)</Typography>
            <Typography variant="body2" color="text.secondary">
              Provide values for the template parameters as defined in the schema.
            </Typography>
            <Editor
              height="200px"
              defaultLanguage="json"
              value={instantiateParams}
              onChange={(value) => setInstantiateParams(value)}
              options={{
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                wordWrap: "on"
              }}
            />

            {selectedTemplate?.parameters_schema && (
              <Box sx={{ bgcolor: 'grey.100', p: 2, borderRadius: 1 }}>
                <Typography variant="subtitle2" gutterBottom>Expected Parameters (from schema):</Typography>
                <Typography variant="body2" component="pre" sx={{ whiteSpace: 'pre-wrap', fontFamily: 'monospace' }}>
                  {selectedTemplate.parameters_schema}
                </Typography>
              </Box>
            )}
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setInstantiateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleInstantiate}
            variant="contained"
            disabled={!selectedStackId}
            startIcon={<InstantiateIcon />}
          >
            Instantiate
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default Templates;
