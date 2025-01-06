import axios from 'axios';

const api = axios.create({
  baseURL: process.env.REACT_APP_BROKER_URL,
});

// Set admin PAK from environment variable
const adminPak = process.env.REACT_APP_ADMIN_PAK;
if (adminPak) {
  api.defaults.headers.common['Authorization'] = `Bearer ${adminPak}`;
} else {
  console.error('Admin PAK not found in environment variables');
}

// Add request interceptor for debugging
api.interceptors.request.use(request => {
  console.log('Starting Request:', request);
  return request;
});

// Add response interceptor for debugging
api.interceptors.response.use(response => {
  console.log('Response:', response);
  return response;
}, error => {
  console.error('Response Error:', error);
  return Promise.reject(error);
});

// Helper function to calculate SHA-256 hash
const calculateSHA256 = async (str) => {
  const encoder = new TextEncoder();
  const data = encoder.encode(str);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
};

export const getAgents = async () => {
  try {
    const response = await api.get('/api/v1/agents');
    return response.data;
  } catch (error) {
    console.error('Failed to fetch agents:', error);
    throw error;
  }
};

export const getStacks = async () => {
  try {
    const response = await api.get('/api/v1/stacks');
    return response.data;
  } catch (error) {
    console.error('Failed to fetch stacks:', error);
    throw error;
  }
};

export const createDeploymentObject = async (stackId, yamlContent, isDeletionMarker = false) => {
  try {
    const yamlChecksum = await calculateSHA256(yamlContent);
    const response = await api.post(`/api/v1/stacks/${stackId}/deployment-objects`, {
      yaml_content: yamlContent,
      yaml_checksum: yamlChecksum,
      is_deletion_marker: isDeletionMarker,
      sequence_id: null  // The server will assign this
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create deployment object:', error);
    throw error;
  }
};

export const getGenerators = async () => {
  try {
    const response = await api.get('/api/v1/generators');
    return response.data;
  } catch (error) {
    console.error('Failed to fetch generators:', error);
    throw error;
  }
};

export const createGenerator = async (name, description = null) => {
  try {
    const response = await api.post('/api/v1/generators', {
      name,
      description
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create generator:', error);
    throw error;
  }
};

export const createAgent = async (name, clusterName) => {
  try {
    const response = await api.post('/api/v1/agents', {
      name,
      cluster_name: clusterName
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create agent:', error);
    throw error;
  }
};

export const getAgent = async (id) => {
  try {
    const response = await api.get(`/api/v1/agents/${id}`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch agent ${id}:`, error);
    throw error;
  }
};

export const updateAgent = async (id, data) => {
  try {
    const response = await api.put(`/api/v1/agents/${id}`, data);
    return response.data;
  } catch (error) {
    console.error(`Failed to update agent ${id}:`, error);
    throw error;
  }
};

export const getAgentEvents = async (id, limit = 50) => {
  try {
    const response = await api.get(`/api/v1/agents/${id}/events?limit=${limit}`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch events for agent ${id}:`, error);
    throw error;
  }
};

export const getAgentLabels = async (agentId) => {
  try {
    const response = await api.get(`/api/v1/agents/${agentId}/labels`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch agent labels:', error);
    throw error;
  }
};

export const addAgentLabel = async (agentId, label) => {
  try {
    const response = await api.post(`/api/v1/agents/${agentId}/labels`, {
      agent_id: agentId,
      label: label
    }, {
      headers: {
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Failed to add agent label:', error);
    throw error;
  }
};

export const removeAgentLabel = async (agentId, label) => {
  try {
    await api.delete(`/api/v1/agents/${agentId}/labels/${label}`);
  } catch (error) {
    console.error('Failed to remove agent label:', error);
    throw error;
  }
};

// Agent Annotations
export const getAgentAnnotations = async (agentId) => {
  try {
    const response = await api.get(`/api/v1/agents/${agentId}/annotations`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch agent annotations:', error);
    throw error;
  }
};

export const addAgentAnnotation = async (agentId, key, value) => {
  try {
    const response = await api.post(`/api/v1/agents/${agentId}/annotations`, {
      agent_id: agentId,
      key,
      value
    });
    return response.data;
  } catch (error) {
    console.error('Failed to add agent annotation:', error);
    throw error;
  }
};

export const removeAgentAnnotation = async (agentId, key) => {
  try {
    await api.delete(`/api/v1/agents/${agentId}/annotations/${key}`);
  } catch (error) {
    console.error('Failed to remove agent annotation:', error);
    throw error;
  }
};

export const createStack = async (name, description, generatorId) => {
  try {
    const response = await api.post('/api/v1/stacks', {
      name,
      description,
      generator_id: generatorId
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create stack:', error);
    throw error;
  }
};

export const verifyPak = async (pak) => {
  try {
    const response = await api.post('/api/v1/auth/pak', null, {
      headers: {
        Authorization: `Bearer ${pak}`
      }
    });
    return response.data;
  } catch (error) {
    console.error('Failed to verify PAK:', error);
    throw error;
  }
};

export const createAgentPak = async (name, clusterName) => {
  try {
    const response = await api.post('/api/v1/agents', {
      name,
      cluster_name: clusterName
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create agent PAK:', error);
    throw error;
  }
};

export const createGeneratorPak = async (name, description = null) => {
  try {
    const response = await api.post('/api/v1/generators', {
      name,
      description
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create generator PAK:', error);
    throw error;
  }
};

// Stack Labels
export const getStackLabels = async (stackId) => {
  try {
    const response = await api.get(`/api/v1/stacks/${stackId}/labels`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch stack labels:', error);
    throw error;
  }
};

export const addStackLabel = async (stackId, label) => {
  try {
    const response = await api.post(`/api/v1/stacks/${stackId}/labels`, JSON.stringify(label), {
      headers: {
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Failed to add stack label:', error);
    throw error;
  }
};

export const removeStackLabel = async (stackId, label) => {
  try {
    await api.delete(`/api/v1/stacks/${stackId}/labels/${label}`);
  } catch (error) {
    console.error('Failed to remove stack label:', error);
    throw error;
  }
};

// Stack Annotations
export const getStackAnnotations = async (stackId) => {
  try {
    const response = await api.get(`/api/v1/stacks/${stackId}/annotations`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch stack annotations:', error);
    throw error;
  }
};

export const addStackAnnotation = async (stackId, key, value) => {
  try {
    const response = await api.post(`/api/v1/stacks/${stackId}/annotations`, {
      stack_id: stackId,
      key,
      value
    });
    return response.data;
  } catch (error) {
    console.error('Failed to add stack annotation:', error);
    throw error;
  }
};

export const removeStackAnnotation = async (stackId, key) => {
  try {
    await api.delete(`/api/v1/stacks/${stackId}/annotations/${key}`);
  } catch (error) {
    console.error('Failed to remove stack annotation:', error);
    throw error;
  }
};

// Stack Management
export const getStack = async (id) => {
  try {
    const response = await api.get(`/api/v1/stacks/${id}`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch stack ${id}:`, error);
    throw error;
  }
};

export const updateStack = async (id, data) => {
  try {
    const response = await api.put(`/api/v1/stacks/${id}`, data);
    return response.data;
  } catch (error) {
    console.error(`Failed to update stack ${id}:`, error);
    throw error;
  }
};

// Stack Deployment Objects
export const getStackDeploymentObjects = async (stackId) => {
  try {
    const response = await api.get(`/api/v1/stacks/${stackId}/deployment-objects`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch deployment objects:', error);
    throw error;
  }
};

export const getDeploymentObject = async (id) => {
  try {
    const response = await api.get(`/api/v1/deployment-objects/${id}`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch deployment object ${id}:`, error);
    throw error;
  }
};

export const getDeploymentEvents = async (deploymentObjectId) => {
  try {
    const response = await api.get(`/api/v1/agent-events?deployment_object_id=${deploymentObjectId}`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch events for deployment object ${deploymentObjectId}:`, error);
    throw error;
  }
};

export const getAgentTargets = async (agentId) => {
  try {
    const response = await api.get(`/api/v1/agents/${agentId}/targets`);
    return response.data;
  } catch (error) {
    console.error('Failed to fetch agent targets:', error);
    throw error;
  }
};

export const addAgentTarget = async (agentId, stackId) => {
  try {
    const response = await api.post(`/api/v1/agents/${agentId}/targets`, {
      agent_id: agentId,
      stack_id: stackId
    });
    return response.data;
  } catch (error) {
    console.error('Failed to add agent target:', error);
    throw error;
  }
};

export const removeAgentTarget = async (agentId, stackId) => {
  try {
    await api.delete(`/api/v1/agents/${agentId}/targets/${stackId}`);
  } catch (error) {
    console.error('Failed to remove agent target:', error);
    throw error;
  }
};

export const getAgentApplicableDeploymentObjects = async (agentId) => {
  try {
    const response = await api.get(`/api/v1/agents/${agentId}/applicable-deployment-objects`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch applicable deployment objects for agent ${agentId}:`, error);
    throw error;
  }
};

export const getAgentAssociatedStacks = async (agentId) => {
  try {
    const response = await api.get(`/api/v1/agents/${agentId}/stacks`);
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch associated stacks for agent ${agentId}:`, error);
    throw error;
  }
};
