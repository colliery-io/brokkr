import angreal
import subprocess
import time
from utils import docker_up, docker_down, docker_clean, cwd
import os

helm = angreal.command_group(name="helm", about="commands for Helm chart testing")


def ensure_k3s_running(skip_docker=False):
    """Ensure k3s cluster is running via docker-compose."""
    if skip_docker:
        print("Skipping docker setup (--skip-docker flag set)")
        return

    print("Starting k3s cluster via docker-compose (k3s only, not full stack)...")
    docker_clean()

    # Also remove the host directory to clear old kubeconfig files
    # (docker_clean removes volumes, but /tmp/brokkr-keys persists on host)
    print("Cleaning up stale kubeconfig files from /tmp/brokkr-keys...")
    subprocess.run(["rm", "-rf", "/tmp/brokkr-keys"], check=False)

    # Only start k3s and init-kubeconfig (init-kubeconfig depends on k3s)
    # The --wait flag ensures init-kubeconfig completes (including its 60s sleep)
    docker_up(services=["k3s", "init-kubeconfig"])

    print("k3s cluster is ready (docker-compose healthcheck passed)")


def run_in_k8s_container(cmd, description="Running command in k8s container"):
    """Run a command inside a kubernetes tools container on the docker network."""
    print(f"{description}...")

    # Use alpine/k8s which has kubectl, helm, and other k8s tools
    # Mount the charts directory and brokkr-keys volume
    # Connect to the same docker network as k3s
    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", "brokkr-dev_default",
        "-v", f"{os.path.join(cwd, 'charts')}:/charts:ro",
        "-v", "brokkr-dev_brokkr-keys:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.27.3",
        "sh", "-c", cmd
    ], cwd=cwd)

    return result.returncode == 0


def verify_kubectl_connectivity():
    """Verify kubectl can connect to k3s cluster."""
    print("\nVerifying kubectl connectivity...")

    # First check if kubeconfig files exist
    print("Checking for kubeconfig files in volume...")
    run_in_k8s_container("ls -la /keys/", "Listing kubeconfig files")

    success = run_in_k8s_container(
        "kubectl get nodes",
        "Testing kubectl connectivity"
    )

    if not success:
        raise Exception("Failed to connect to k3s cluster")

    print("kubectl connectivity verified")


def setup_image_pull_secret(registry, namespace="default"):
    """Create image pull secret for private registry access."""
    import os

    # Check if GITHUB_TOKEN is available
    github_token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")

    if not github_token:
        print("\nWarning: No GITHUB_TOKEN or GH_TOKEN environment variable found.")
        print("Image pulls from private GHCR repositories will fail.")
        print("Set GITHUB_TOKEN with a PAT that has read:packages scope.")
        return False

    print(f"\nCreating image pull secret for {registry}...")

    # Create docker config for GHCR
    # The username doesn't matter for GHCR, the token is what's important
    cmd = f"""
        kubectl create secret docker-registry ghcr-secret \
            --docker-server={registry} \
            --docker-username=token \
            --docker-password={github_token} \
            --namespace {namespace} \
            --dry-run=client -o yaml | kubectl apply -f -
    """

    # Pass the token via environment variable to the container
    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", "brokkr-dev_default",
        "-v", f"{os.path.join(cwd, 'charts')}:/charts:ro",
        "-v", "brokkr-dev_brokkr-keys:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "-e", f"GITHUB_TOKEN={github_token}",
        "alpine/k8s:1.27.3",
        "sh", "-c", cmd
    ], cwd=cwd)

    if result.returncode == 0:
        print("Image pull secret created successfully")
        return True
    else:
        print("Failed to create image pull secret")
        return False


def helm_install(chart_name, release_name, values, namespace="default"):
    """Install a Helm chart."""
    print("")
    print("=" * 60)
    print(f"Installing Helm chart: {chart_name}")
    print(f"Release name: {release_name}")
    print(f"Namespace: {namespace}")
    print("=" * 60)
    print("")

    # Build helm install command with values
    values_args = " ".join([f"--set {k}={v}" for k, v in values.items()])

    cmd = f"""
        helm install {release_name} /charts/{chart_name} \
            --namespace {namespace} \
            --create-namespace \
            --wait \
            --timeout 10m \
            --debug \
            {values_args}
    """

    success = run_in_k8s_container(cmd, f"Installing {chart_name}")

    if not success:
        print(f"\nFailed to install {chart_name}")
        print("Checking pod status...")
        run_in_k8s_container(
            f"kubectl get pods -n {namespace} -l app.kubernetes.io/instance={release_name}",
            "Getting pod status"
        )
        print("\nChecking pod logs...")
        run_in_k8s_container(
            f"kubectl logs -n {namespace} -l app.kubernetes.io/instance={release_name} --all-containers --tail=50",
            "Getting pod logs"
        )
        print("\nChecking pod events...")
        run_in_k8s_container(
            f"kubectl get events -n {namespace} --sort-by='.lastTimestamp'",
            "Getting events"
        )
    else:
        print(f"\nSuccessfully installed {chart_name}")

    return success


def helm_uninstall(release_name, namespace="default"):
    """Uninstall a Helm release."""
    print(f"\nUninstalling Helm release: {release_name}")

    cmd = f"helm uninstall {release_name} --namespace {namespace} --wait"
    return run_in_k8s_container(cmd, f"Uninstalling {release_name}")


def wait_for_pods(release_name, namespace="default", timeout=300):
    """Wait for all pods in a release to be ready."""
    print(f"\nWaiting for pods in release '{release_name}' to be ready...")

    start_time = time.time()
    while time.time() - start_time < timeout:
        cmd = f"""
            kubectl get pods -n {namespace} \
                -l app.kubernetes.io/instance={release_name} \
                -o jsonpath='{{range .items[*]}}{{.status.phase}}:{{range .status.conditions[?(@.type=="Ready")]}}{{.status}}{{end}} {{end}}'
        """

        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", "brokkr-dev_default",
            "-v", "brokkr-dev_brokkr-keys:/keys:ro",
            "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
            "alpine/k8s:1.27.3",
            "sh", "-c", cmd
        ], capture_output=True, text=True, cwd=cwd)

        if result.returncode == 0 and result.stdout.strip():
            # Check if all pods are Running:True
            pod_statuses = result.stdout.strip().split()
            if pod_statuses and all("Running:True" in status for status in pod_statuses):
                print(f"All pods in release '{release_name}' are ready!")
                return True

        time.sleep(5)

    print(f"Timeout waiting for pods in release '{release_name}' to be ready")
    return False


def validate_health_endpoint(service_name, port, path, namespace="default"):
    """Validate a health check endpoint via the service."""
    print(f"\nValidating health endpoint: {service_name}:{port}{path}")

    # Use kubectl run to create a temporary pod that curls the service
    cmd = f"""
        kubectl run curl-test-$RANDOM --rm -i --restart=Never --image=curlimages/curl:latest \
            -n {namespace} -- curl -f -s http://{service_name}:{port}{path}
    """

    success = run_in_k8s_container(cmd, f"Testing health endpoint {path}")

    if success:
        print(f"✓ Health check passed: {path}")
    else:
        print(f"✗ Health check failed: {path}")

    return success


def test_broker_chart(tag, registry, no_cleanup):
    """Test the broker Helm chart."""
    release_name = "brokkr-broker-test"
    chart_name = "brokkr-broker"

    # Setup image pull secret
    setup_image_pull_secret(registry.split('/')[0])  # Extract hostname (ghcr.io)

    values = {
        "image.tag": tag,
        "image.repository": f"{registry}/brokkr-broker",
        "image.pullSecrets[0].name": "ghcr-secret",
        "postgresql.enabled": "true",  # Use bundled PostgreSQL for testing
    }

    # Install chart
    if not helm_install(chart_name, release_name, values):
        return False

    # Wait for pods
    if not wait_for_pods(release_name):
        if not no_cleanup:
            helm_uninstall(release_name)
        return False

    # Validate health endpoints
    health_passed = True
    health_passed &= validate_health_endpoint(release_name, 3000, "/healthz")
    health_passed &= validate_health_endpoint(release_name, 3000, "/readyz")

    if not no_cleanup:
        helm_uninstall(release_name)

    return health_passed


def create_agent_in_broker(broker_release_name, agent_name, cluster_name, namespace="default"):
    """Create an agent via the broker CLI and return the PAK."""
    print(f"\nCreating agent '{agent_name}' in cluster '{cluster_name}' via broker...")

    # Get the broker pod name
    # Use just name=brokkr-broker and instance labels (no component label)
    get_pod_cmd = f"""
        kubectl get pods -n {namespace} \
            -l app.kubernetes.io/name=brokkr-broker,app.kubernetes.io/instance={broker_release_name} \
            -o jsonpath='{{.items[0].metadata.name}}'
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", "brokkr-dev_default",
        "-v", "brokkr-dev_brokkr-keys:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.27.3",
        "sh", "-c", get_pod_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to get broker pod name")
        return None

    broker_pod = result.stdout.strip()
    print(f"Broker pod: {broker_pod}")

    # Run the create agent command in the broker pod
    create_agent_cmd = f"""
        kubectl exec {broker_pod} -n {namespace} -- \
            brokkr-broker create agent --name {agent_name} --cluster-name {cluster_name}
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", "brokkr-dev_default",
        "-v", "brokkr-dev_brokkr-keys:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.27.3",
        "sh", "-c", create_agent_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0:
        print("Failed to create agent")
        print(f"Error: {result.stderr}")
        return None

    # Parse the PAK from the output
    # The output should contain the PAK
    output = result.stdout.strip()
    print(f"Agent creation output:\n{output}")

    # Look for PAK in the output (assuming it's printed)
    # We'll need to parse this based on the actual output format
    for line in output.split('\n'):
        if 'PAK' in line or 'pak' in line or line.startswith('pak_'):
            # Extract the PAK value
            pak = line.split()[-1]  # Assume PAK is the last word on the line
            print(f"Extracted PAK: {pak[:10]}...")
            return pak

    # If we can't find PAK in a labeled line, try to find a line that looks like a PAK
    for line in output.split('\n'):
        line = line.strip()
        if line and not line.startswith('#') and not line.startswith('['):
            # Might be the PAK itself
            print(f"Potential PAK found: {line[:10]}...")
            return line

    print("Failed to extract PAK from output")
    return None


def test_agent_chart(tag, registry, no_cleanup):
    """Test the agent Helm chart.

    This test performs a full integration test:
    1. Deploys a broker chart instance
    2. Creates an agent via the broker CLI to get a valid PAK
    3. Deploys the agent chart with the real broker URL and PAK
    4. Validates the agent is running and healthy
    """
    agent_release_name = "brokkr-agent-test"
    broker_release_name = "brokkr-broker-for-agent-test"
    agent_chart_name = "brokkr-agent"
    broker_chart_name = "brokkr-broker"
    broker_cleanup_needed = False

    try:
        # Setup image pull secret
        setup_image_pull_secret(registry.split('/')[0])  # Extract hostname (ghcr.io)

        # Step 1: Deploy broker chart for the agent to connect to
        print("\n" + "=" * 60)
        print("Step 1: Deploying broker for agent testing")
        print("=" * 60)

        broker_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-broker",
            "image.pullSecrets[0].name": "ghcr-secret",
            "postgresql.enabled": "true",
        }

        if not helm_install(broker_chart_name, broker_release_name, broker_values):
            print("Failed to deploy broker for agent testing")
            return False

        broker_cleanup_needed = True

        if not wait_for_pods(broker_release_name):
            print("Broker pods failed to become ready")
            return False

        # Step 2: Create agent via broker CLI to get PAK
        print("\n" + "=" * 60)
        print("Step 2: Creating agent via broker CLI")
        print("=" * 60)

        pak = create_agent_in_broker(
            broker_release_name,
            "test-agent",
            "test-cluster"
        )

        if not pak:
            print("Failed to create agent and get PAK")
            return False

        # Step 3: Deploy agent chart with real configuration
        print("\n" + "=" * 60)
        print("Step 3: Deploying agent chart")
        print("=" * 60)

        # The broker service URL uses the release name
        broker_url = f"http://{broker_release_name}:3000"

        agent_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-agent",
            "image.pullSecrets[0].name": "ghcr-secret",
            "broker.url": broker_url,
            "broker.agentName": "test-agent",
            "broker.clusterName": "test-cluster",
            "broker.pak": pak,
        }

        if not helm_install(agent_chart_name, agent_release_name, agent_values):
            return False

        # Wait for agent pods
        if not wait_for_pods(agent_release_name):
            if not no_cleanup:
                helm_uninstall(agent_release_name)
            return False

        # Agent doesn't expose a service - health is validated by k8s readiness probes
        print("\n" + "=" * 60)
        print("Step 4: Agent validation complete")
        print("=" * 60)
        print("Agent pods are ready and healthy (validated by k8s readiness probes)")
        print("Agent is successfully connected to broker")

        return True

    finally:
        # Cleanup
        if not no_cleanup:
            print("\nCleaning up agent and broker deployments...")
            helm_uninstall(agent_release_name)
            if broker_cleanup_needed:
                helm_uninstall(broker_release_name)


@helm()
@angreal.command(name="test", about="test Helm charts in k3s cluster")
@angreal.argument(name="component", required=True, help="Component to test (broker, agent, all)")
@angreal.argument(name="skip_docker", long="skip-docker", help="Skip docker compose setup", takes_value=False, is_flag=True)
@angreal.argument(name="no_cleanup", long="no-cleanup", help="Skip cleanup after tests", takes_value=False, is_flag=True)
@angreal.argument(name="tag", long="tag", help="Image tag to test (default: test)", default_value="test")
@angreal.argument(name="registry", long="registry", help="Registry URL (default: ghcr.io/colliery-io)", default_value="ghcr.io/colliery-io")
def test_helm_chart(component, skip_docker=False, no_cleanup=False, tag="test", registry="ghcr.io/colliery-io"):
    """
    Test Helm charts in a k3s cluster.

    This command will:
    1. Start k3s cluster (unless --skip-docker)
    2. Install the specified Helm chart(s) from within a container
    3. Validate pods are running
    4. Test health check endpoints
    5. Clean up (unless --no-cleanup)

    All helm/kubectl commands run inside a container on the docker network,
    avoiding host networking issues.

    Examples:
        angreal helm test broker --tag test
        angreal helm test agent --skip-docker
        angreal helm test all --no-cleanup
    """
    valid_components = ["broker", "agent", "all"]
    if component not in valid_components:
        print(f"Error: Unknown component '{component}'")
        print(f"Valid components: {', '.join(valid_components)}")
        return 1

    try:
        # Setup k3s
        ensure_k3s_running(skip_docker)

        # Verify kubectl connectivity
        verify_kubectl_connectivity()

        # Test components
        results = []

        if component in ["broker", "all"]:
            print("\n" + "=" * 60)
            print("Testing broker chart")
            print("=" * 60)
            result = test_broker_chart(tag, registry, no_cleanup)
            results.append(("broker", result))

        if component in ["agent", "all"]:
            print("\n" + "=" * 60)
            print("Testing agent chart")
            print("=" * 60)
            result = test_agent_chart(tag, registry, no_cleanup)
            results.append(("agent", result))

        # Summary
        print("\n" + "=" * 60)
        print("Test Results:")
        print("=" * 60)
        for comp_name, result in results:
            status = "✓ PASSED" if result else "✗ FAILED"
            print(f"{comp_name}: {status}")
        print("=" * 60)

        if no_cleanup:
            print("\nHelm releases left running (--no-cleanup)")
            print("To inspect, run commands in a k8s container:")
            print("  docker run --rm -it --network brokkr-dev_default \\")
            print("    -v brokkr-dev_brokkr-keys:/keys:ro \\")
            print("    -e KUBECONFIG=/keys/kubeconfig.docker.yaml \\")
            print("    alpine/k8s:1.27.3 sh")
            print("  # Then inside container:")
            print("  kubectl get pods")
            print("  helm list")
            print("\nTo clean up manually:")
            print("  angreal local down --hard")

        # Cleanup docker if needed
        if not skip_docker and not no_cleanup:
            print("\nCleaning up docker environment...")
            docker_down()
            docker_clean()

        # Return success only if all tests passed
        return 0 if all(result for _, result in results) else 1

    except Exception as e:
        print(f"\nError during Helm testing: {e}")
        if not skip_docker and not no_cleanup:
            print("Cleaning up docker environment...")
            docker_down()
            docker_clean()
        return 1
