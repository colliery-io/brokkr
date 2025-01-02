import React from 'react';
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import {
  AppBar,
  Toolbar,
  Typography,
  Container,
  Box,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  CssBaseline
} from '@mui/material';
import {
  Computer as AgentIcon,
  Cloud as ClusterIcon,
  Storage as DeploymentIcon
} from '@mui/icons-material';

// Placeholder components
const Agents = () => <div>Agents Dashboard</div>;
const Clusters = () => <div>Clusters Dashboard</div>;
const Deployments = () => <div>Deployments Dashboard</div>;

const drawerWidth = 240;

function App() {
  return (
    <Router>
      <Box sx={{ display: 'flex' }}>
        <CssBaseline />
        <AppBar position="fixed" sx={{ zIndex: (theme) => theme.zIndex.drawer + 1 }}>
          <Toolbar>
            <Typography variant="h6" noWrap component="div">
              Brokkr Dashboard
            </Typography>
          </Toolbar>
        </AppBar>
        <Drawer
          variant="permanent"
          sx={{
            width: drawerWidth,
            flexShrink: 0,
            '& .MuiDrawer-paper': {
              width: drawerWidth,
              boxSizing: 'border-box',
            },
          }}
        >
          <Toolbar />
          <Box sx={{ overflow: 'auto' }}>
            <List>
              <ListItem button component={Link} to="/agents">
                <ListItemIcon><AgentIcon /></ListItemIcon>
                <ListItemText primary="Agents" />
              </ListItem>
              <ListItem button component={Link} to="/clusters">
                <ListItemIcon><ClusterIcon /></ListItemIcon>
                <ListItemText primary="Clusters" />
              </ListItem>
              <ListItem button component={Link} to="/deployments">
                <ListItemIcon><DeploymentIcon /></ListItemIcon>
                <ListItemText primary="Deployments" />
              </ListItem>
            </List>
          </Box>
        </Drawer>
        <Box component="main" sx={{ flexGrow: 1, p: 3 }}>
          <Toolbar />
          <Container>
            <Routes>
              <Route path="/agents" element={<Agents />} />
              <Route path="/clusters" element={<Clusters />} />
              <Route path="/deployments" element={<Deployments />} />
              <Route path="/" element={<Agents />} />
            </Routes>
          </Container>
        </Box>
      </Box>
    </Router>
  );
}

export default App;
