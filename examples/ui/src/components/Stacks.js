import React, { useState, useEffect } from 'react';
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
  Fab
} from '@mui/material';
import {
  Refresh as RefreshIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Label as LabelIcon,
  Edit as EditIcon
} from '@mui/icons-material';
import {
  getStacks,
  createStack,
  getStackLabels,
  getStackAnnotations,
  addStackLabel,
  removeStackLabel,
  addStackAnnotation,
  removeStackAnnotation
} from '../services/api';

const Stacks = () => {
  const [stacks, setStacks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedStack, setSelectedStack] = useState(null);
  const [stackDetails, setStackDetails] = useState({ labels: [], annotations: [] });
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [labelDialogOpen, setLabelDialogOpen] = useState(false);
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [newStack, setNewStack] = useState({ name: '', description: '' });
  const [newLabel, setNewLabel] = useState('');
  const [newAnnotation, setNewAnnotation] = useState({ key: '', value: '' });

  const fetchStacks = async () => {
    try {
      setLoading(true);
      const data = await getStacks();
      setStacks(data);
      setError(null);
    } catch (err) {
      setError('Failed to fetch stacks. Please check your admin PAK.');
      console.error('Error fetching stacks:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStacks();
  }, []);

  const fetchStackDetails = async (stackId) => {
    setLoadingDetails(true);
    try {
      const [labels, annotations] = await Promise.all([
        getStackLabels(stackId),
        getStackAnnotations(stackId)
      ]);
      setStackDetails({ labels, annotations });
    } catch (err) {
      console.error('Error fetching stack details:', err);
    } finally {
      setLoadingDetails(false);
    }
  };

  const handleRowClick = (stack) => {
    setSelectedStack(stack);
    fetchStackDetails(stack.id);
  };

  const handleCloseDialog = () => {
    setSelectedStack(null);
    setStackDetails({ labels: [], annotations: [] });
    setLabelDialogOpen(false);
    setAnnotationDialogOpen(false);
  };

  const handleCreateStack = async () => {
    if (!newStack.name.trim()) return;
    try {
      setLoading(true);
      await createStack(newStack.name.trim(), newStack.description.trim() || null);
      setNewStack({ name: '', description: '' });
      setCreateDialogOpen(false);
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
    } catch (err) {
      console.error('Error adding label:', err);
    }
  };

  const handleRemoveLabel = async (label) => {
    try {
      await removeStackLabel(selectedStack.id, label);
      await fetchStackDetails(selectedStack.id);
    } catch (err) {
      console.error('Error removing label:', err);
    }
  };

  const handleAddAnnotation = async () => {
    if (!newAnnotation.key.trim() || !newAnnotation.value.trim()) return;
    try {
      await addStackAnnotation(selectedStack.id, newAnnotation.key.trim(), newAnnotation.value.trim());
      await fetchStackDetails(selectedStack.id);
      setNewAnnotation({ key: '', value: '' });
      setAnnotationDialogOpen(false);
    } catch (err) {
      console.error('Error adding annotation:', err);
    }
  };

  const handleRemoveAnnotation = async (key) => {
    try {
      await removeStackAnnotation(selectedStack.id, key);
      await fetchStackDetails(selectedStack.id);
    } catch (err) {
      console.error('Error removing annotation:', err);
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
        <Box>
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
                <TableCell>Created</TableCell>
                <TableCell>Last Updated</TableCell>
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
                      {stackDetails.labels
                        .filter(label => label.stack_id === stack.id)
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
                  <TableCell>{new Date(stack.created_at).toLocaleString()}</TableCell>
                  <TableCell>{new Date(stack.updated_at).toLocaleString()}</TableCell>
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

      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)}>
        <DialogTitle>Create New Stack</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Stack Name"
            fullWidth
            value={newStack.name}
            onChange={(e) => setNewStack({ ...newStack, name: e.target.value })}
            required
          />
          <TextField
            margin="dense"
            label="Description"
            fullWidth
            value={newStack.description}
            onChange={(e) => setNewStack({ ...newStack, description: e.target.value })}
            multiline
            rows={3}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateStack} disabled={!newStack.name.trim()}>
            Create
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={!!selectedStack} onClose={handleCloseDialog} maxWidth="md" fullWidth>
        <DialogTitle>
          Stack Details: {selectedStack?.name}
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
                {stackDetails.labels.map((label) => (
                  <Chip
                    key={label}
                    label={label}
                    onDelete={() => handleRemoveLabel(label)}
                    size="small"
                  />
                ))}
                {stackDetails.labels.length === 0 && (
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
                    {stackDetails.annotations.map((annotation) => (
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
                    {stackDetails.annotations.length === 0 && (
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

export default Stacks;
