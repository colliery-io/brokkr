import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  Tabs,
  Tab,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Alert,
  List,
  ListItem,
  ListItemText,
  Chip,
  IconButton,
  Tooltip,
  CircularProgress,
  Divider
} from '@mui/material';
import {
  Add as AddIcon,
  ContentCopy as CopyIcon,
  Refresh as RefreshIcon
} from '@mui/icons-material';
import { getAgents, getGenerators, createAgentPak, createGeneratorPak } from '../services/api';

const TabPanel = ({ children, value, index }) => (
  <div hidden={value !== index} style={{ padding: '20px 0' }}>
    {value === index && children}
  </div>
);

const Admin = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [agents, setAgents] = useState([]);
  const [generators, setGenerators] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(null);
  const [createDialog, setCreateDialog] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    clusterName: ''
  });
  const [newPak, setNewPak] = useState(null);
  const [pakCopied, setPakCopied] = useState(false);
  const [showConfirmClose, setShowConfirmClose] = useState(false);

  const fetchData = async () => {
    try {
      setLoading(true);
      const [agentsData, generatorsData] = await Promise.all([
        getAgents(),
        getGenerators()
      ]);
      setAgents(agentsData);
      setGenerators(generatorsData);
      setError(null);
    } catch (err) {
      setError('Failed to fetch data. Please check your admin PAK.');
      console.error('Error fetching data:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleCreatePak = () => {
    setCreateDialog(true);
    setFormData({
      name: '',
      description: '',
      clusterName: ''
    });
    setNewPak(null);
    setPakCopied(false);
    setShowConfirmClose(false);
  };

  const handleCloseDialog = () => {
    if (newPak && !pakCopied) {
      setShowConfirmClose(true);
    } else {
      setCreateDialog(false);
      setNewPak(null);
      setPakCopied(false);
      setShowConfirmClose(false);
    }
  };

  const handleSubmit = async () => {
    try {
      setLoading(true);
      let response;

      if (activeTab === 0) { // Agent PAK
        response = await createAgentPak(formData.name, formData.clusterName);
        console.log('Agent PAK response:', response);
        if (!response.initial_pak) {
          throw new Error('No PAK received in response');
        }
        setNewPak(response.initial_pak);
      } else if (activeTab === 1) { // Generator PAK
        response = await createGeneratorPak(formData.name, formData.description);
        console.log('Generator PAK response:', response);
        if (!response.pak) {
          throw new Error('No PAK received in response');
        }
        setNewPak(response.pak);
      }

      setPakCopied(false);
      setSuccess(`${activeTab === 0 ? 'Agent' : 'Generator'} PAK created successfully!`);
      fetchData(); // Refresh the list
    } catch (err) {
      console.error('Error creating PAK:', err);
      setError('Failed to create PAK. Please check your input and try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleCopyPak = (pak) => {
    navigator.clipboard.writeText(pak);
    setPakCopied(true);
    setSuccess('PAK copied to clipboard!');
  };

  const renderPakDialog = () => (
    <>
      <Dialog
        open={createDialog}
        onClose={handleCloseDialog}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          Create New {activeTab === 0 ? 'Agent' : 'Generator'} PAK
        </DialogTitle>
        <DialogContent>
          <Box sx={{ pt: 2, display: 'flex', flexDirection: 'column', gap: 2 }}>
            <TextField
              label="Name"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              fullWidth
              disabled={!!newPak}
            />
            {activeTab === 0 ? (
              <TextField
                label="Cluster Name"
                value={formData.clusterName}
                onChange={(e) => setFormData({ ...formData, clusterName: e.target.value })}
                required
                fullWidth
                disabled={!!newPak}
              />
            ) : (
              <TextField
                label="Description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                multiline
                rows={3}
                fullWidth
                disabled={!!newPak}
              />
            )}
            {newPak && (
              <Box sx={{ mt: 2 }}>
                <Alert
                  severity="warning"
                  sx={{ mb: 2 }}
                >
                  Please copy your PAK now. For security reasons, it will not be shown again.
                </Alert>
                <Paper
                  sx={{
                    p: 2,
                    backgroundColor: (theme) => theme.palette.grey[100],
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between'
                  }}
                >
                  <Typography
                    component="code"
                    sx={{
                      fontFamily: 'monospace',
                      fontSize: '1.1em',
                      wordBreak: 'break-all'
                    }}
                  >
                    {newPak}
                  </Typography>
                  <Tooltip title={pakCopied ? "Copied!" : "Copy to clipboard"}>
                    <IconButton
                      onClick={() => handleCopyPak(newPak)}
                      color={pakCopied ? "success" : "default"}
                    >
                      <CopyIcon />
                    </IconButton>
                  </Tooltip>
                </Paper>
              </Box>
            )}
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCloseDialog} disabled={loading}>
            Close
          </Button>
          {!newPak && (
            <Button
              onClick={handleSubmit}
              variant="contained"
              disabled={loading || !formData.name || (activeTab === 0 && !formData.clusterName)}
            >
              {loading ? <CircularProgress size={24} /> : 'Create'}
            </Button>
          )}
        </DialogActions>
      </Dialog>

      <Dialog
        open={showConfirmClose}
        onClose={() => setShowConfirmClose(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>
          Warning
        </DialogTitle>
        <DialogContent>
          <Typography>
            You haven't copied the PAK yet. This PAK will not be shown again after closing. Are you sure you want to close?
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowConfirmClose(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => {
              setCreateDialog(false);
              setShowConfirmClose(false);
              setNewPak(null);
              setPakCopied(false);
            }}
            color="error"
          >
            Close Anyway
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );

  if (loading && !agents.length && !generators.length) {
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
          PAK Management
        </Typography>
        <Box display="flex" gap={2}>
          <Tooltip title="Refresh">
            <IconButton onClick={fetchData} disabled={loading}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={handleCreatePak}
          >
            Create PAK
          </Button>
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

      <Paper sx={{ mb: 3 }}>
        <Tabs
          value={activeTab}
          onChange={(e, newValue) => setActiveTab(newValue)}
          sx={{ borderBottom: 1, borderColor: 'divider' }}
        >
          <Tab label="Agent PAKs" />
          <Tab label="Generator PAKs" />
        </Tabs>

        <TabPanel value={activeTab} index={0}>
          <List>
            {agents.length === 0 ? (
              <ListItem>
                <ListItemText
                  primary="No agent PAKs found"
                  secondary="Create a new agent PAK to get started"
                />
              </ListItem>
            ) : (
              agents.map((agent) => (
                <React.Fragment key={agent.id}>
                  <ListItem>
                    <ListItemText
                      primary={agent.name}
                      secondary={`Cluster: ${agent.cluster_name}`}
                    />
                    <Chip
                      label={agent.status}
                      color={agent.status === 'ACTIVE' ? 'success' : 'default'}
                      size="small"
                      sx={{ ml: 1 }}
                    />
                  </ListItem>
                  <Divider />
                </React.Fragment>
              ))
            )}
          </List>
        </TabPanel>

        <TabPanel value={activeTab} index={1}>
          <List>
            {generators.length === 0 ? (
              <ListItem>
                <ListItemText
                  primary="No generator PAKs found"
                  secondary="Create a new generator PAK to get started"
                />
              </ListItem>
            ) : (
              generators.map((generator) => (
                <React.Fragment key={generator.id}>
                  <ListItem>
                    <ListItemText
                      primary={generator.name}
                      secondary={generator.description || 'No description'}
                    />
                  </ListItem>
                  <Divider />
                </React.Fragment>
              ))
            )}
          </List>
        </TabPanel>
      </Paper>

      {renderPakDialog()}
    </Box>
  );
};

export default Admin;
