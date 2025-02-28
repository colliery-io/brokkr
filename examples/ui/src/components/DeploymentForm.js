import React, { useState, useEffect } from 'react';
import {
  Box,
  Button,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  TextField,
  Typography,
  Paper,
  Alert,
  FormControlLabel,
  Switch,
  CircularProgress
} from '@mui/material';
import { getStacks, createDeploymentObject } from '../services/api';

const DeploymentForm = () => {
  const [stacks, setStacks] = useState([]);
  const [selectedStack, setSelectedStack] = useState('');
  const [yamlContent, setYamlContent] = useState('');
  const [isDeletionMarker, setIsDeletionMarker] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    const fetchStacks = async () => {
      try {
        const data = await getStacks();
        setStacks(data);
      } catch (err) {
        setError('Failed to fetch stacks. Please check your admin PAK.');
      }
    };
    fetchStacks();
  }, []);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(false);

    try {
      await createDeploymentObject(selectedStack, yamlContent, isDeletionMarker);
      setSuccess(true);
      setYamlContent('');
      setIsDeletionMarker(false);
    } catch (err) {
      setError('Failed to create deployment object. Please check your input and try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box>
      <Typography variant="h5" gutterBottom>
        Submit Deployment
      </Typography>

      <Paper sx={{ p: 3, mt: 2 }}>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {success && (
          <Alert severity="success" sx={{ mb: 2 }}>
            Deployment object created successfully!
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <FormControl fullWidth sx={{ mb: 2 }}>
            <InputLabel>Select Stack</InputLabel>
            <Select
              value={selectedStack}
              label="Select Stack"
              onChange={(e) => setSelectedStack(e.target.value)}
              required
            >
              {stacks.map((stack) => (
                <MenuItem key={stack.id} value={stack.id}>
                  {stack.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          <TextField
            fullWidth
            multiline
            rows={10}
            label="YAML Content"
            value={yamlContent}
            onChange={(e) => setYamlContent(e.target.value)}
            required
            sx={{ mb: 2 }}
            placeholder="Enter your Kubernetes YAML content here..."
          />

          <FormControlLabel
            control={
              <Switch
                checked={isDeletionMarker}
                onChange={(e) => setIsDeletionMarker(e.target.checked)}
              />
            }
            label="Mark as deletion"
            sx={{ mb: 2 }}
          />

          <Button
            type="submit"
            variant="contained"
            color="primary"
            disabled={loading || !selectedStack || !yamlContent.trim()}
            sx={{ mt: 2 }}
          >
            {loading ? <CircularProgress size={24} /> : 'Submit Deployment'}
          </Button>
        </form>
      </Paper>
    </Box>
  );
};

export default DeploymentForm;
