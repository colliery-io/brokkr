#!/bin/sh

# Wait for broker to be ready
echo "Waiting for broker to be ready..."
until $(curl --output /dev/null --silent --fail http://brokkr-broker:3000/readyz); do
    printf '.'
    sleep 1
done
echo "Broker is ready!"

# Create the agent using the admin PAK
echo "Creating agent..."
response=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8" \
    -d '{"name":"brokkr-integration-test-agent","cluster_name":"brokkr-dev-integration-cluster"}' \
    http://brokkr-broker:3000/api/v1/agents)

# Extract the PAK from the response
pak=$(echo $response | jq -r '.initial_pak')

# Save the PAK to a file that can be used by the agent
echo $pak > /shared/agent.pak

echo "Agent initialization complete!"
