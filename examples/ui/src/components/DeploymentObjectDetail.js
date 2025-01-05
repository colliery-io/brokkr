import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  Chip,
  CircularProgress,
  Alert,
  TableContainer,
  Table,
  TableHead,
  TableBody,
  TableRow,
  TableCell,
  Link as MuiLink
} from '@mui/material';
import { useParams, Link } from 'react-router-dom';
import Editor from "@monaco-editor/react";
import { getDeploymentObject, getDeploymentEvents } from '../services/api';

const DeploymentObjectDetail = () => {
  const { id } = useParams();
  const [deployment, setDeployment] = useState(null);
  const [events, setEvents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        const [deploymentData, eventsData] = await Promise.all([
          getDeploymentObject(id),
          getDeploymentEvents(id)
        ]);
        setDeployment(deploymentData);
        setEvents(eventsData);
      } catch (err) {
        console.error('Error fetching deployment details:', err);
        setError('Failed to fetch deployment details');
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [id]);

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ mb: 2 }}>
        {error}
      </Alert>
    );
  }

  if (!deployment) {
    return (
      <Alert severity="info">
        Deployment object not found
      </Alert>
    );
  }

  return (
    <Box>
      <Box display="flex" alignItems="center" mb={3}>
        <Typography variant="h5" mr={2}>
          Deployment Object Details
        </Typography>
        <Chip
          label={deployment.is_deletion_marker ? 'Deletion' : 'Deployment'}
          color={deployment.is_deletion_marker ? 'error' : 'primary'}
          size="small"
        />
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Box display="flex" flexDirection="column" gap={1} mb={3}>
          <Typography variant="body2">
            <strong>ID:</strong> {deployment.id}
          </Typography>
          <Typography variant="body2">
            <strong>Created:</strong> {new Date(deployment.created_at).toLocaleString()}
          </Typography>
          <Typography variant="body2">
            <strong>Stack:</strong>{' '}
            <MuiLink component={Link} to={`/stacks/${deployment.stack_id}`}>
              {deployment.stack_id}
            </MuiLink>
          </Typography>
        </Box>

        <Typography variant="subtitle1" gutterBottom>
          YAML Content
        </Typography>
        <Editor
          height="300px"
          defaultLanguage="yaml"
          value={deployment.yaml_content}
          options={{
            readOnly: true,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            wordWrap: "on"
          }}
        />
      </Paper>

      <Typography variant="h6" gutterBottom>
        Agent Events
      </Typography>
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Timestamp</TableCell>
              <TableCell>Agent</TableCell>
              <TableCell>Event Type</TableCell>
              <TableCell>Status</TableCell>
              <TableCell>Message</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {events.map((event) => (
              <TableRow key={event.id}>
                <TableCell>{new Date(event.created_at).toLocaleString()}</TableCell>
                <TableCell>
                  <MuiLink component={Link} to={`/agents/${event.agent_id}`}>
                    {event.agent_id}
                  </MuiLink>
                </TableCell>
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
                <TableCell style={{ whiteSpace: 'pre-wrap', maxWidth: '400px' }}>
                  {event.message || '-'}
                </TableCell>
              </TableRow>
            ))}
            {events.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  <Typography color="text.secondary">No events recorded</Typography>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  );
};

export default DeploymentObjectDetail;
