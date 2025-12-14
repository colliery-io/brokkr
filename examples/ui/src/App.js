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
  CssBaseline,
  Divider
} from '@mui/material';
import {
  Computer as AgentIcon,
  Storage as StackIcon,
  AdminPanelSettings as AdminIcon,
  Description as TemplateIcon
} from '@mui/icons-material';
import Agents from './components/Agents';
import Stacks from './components/Stacks';
import Templates from './components/Templates';
import Admin from './components/Admin';
import DeploymentObjectDetail from './components/DeploymentObjectDetail';

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
              <ListItem button component={Link} to="/stacks">
                <ListItemIcon><StackIcon /></ListItemIcon>
                <ListItemText primary="Stacks" />
              </ListItem>
              <ListItem button component={Link} to="/templates">
                <ListItemIcon><TemplateIcon /></ListItemIcon>
                <ListItemText primary="Templates" />
              </ListItem>
              <Divider sx={{ my: 2 }} />
              <ListItem button component={Link} to="/admin">
                <ListItemIcon><AdminIcon /></ListItemIcon>
                <ListItemText primary="Admin" />
              </ListItem>
            </List>
          </Box>
        </Drawer>
        <Box component="main" sx={{ flexGrow: 1, p: 3 }}>
          <Toolbar />
          <Container>
            <Routes>
              <Route path="/agents" element={<Agents />} />
              <Route path="/stacks" element={<Stacks />} />
              <Route path="/templates" element={<Templates />} />
              <Route path="/admin" element={<Admin />} />
              <Route path="/deployment-objects/:id" element={<DeploymentObjectDetail />} />
              <Route path="/" element={<Agents />} />
            </Routes>
          </Container>
        </Box>
      </Box>
    </Router>
  );
}

export default App;
