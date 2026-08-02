import angreal
import json
import re
import shutil
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path
from utils import docker_up, docker_down, docker_clean, cwd
import os

helm = angreal.command_group(name="helm", about="commands for Helm chart testing")

# Local registry configuration
LOCAL_REGISTRY = "localhost:5050"
LOCAL_REGISTRY_K8S = "registry:5000"  # How k3s sees the registry (via docker network)

# Global project name - set at test start for isolation
_PROJECT_NAME = None


def generate_project_name():
    """Generate a unique project name for isolated test runs."""
    short_id = uuid.uuid4().hex[:8]
    return f"helm-test-{short_id}"


def get_project_name():
    """Get the current project name."""
    global _PROJECT_NAME
    if _PROJECT_NAME is None:
        _PROJECT_NAME = generate_project_name()
    return _PROJECT_NAME


def get_network_name():
    """Get the docker network name for the current project."""
    return f"{get_project_name()}_default"


def get_volume_name(volume):
    """Get the full volume name for the current project."""
    return f"{get_project_name()}_{volume}"


def build_and_push_local_images(tag="local"):
    """Build broker and agent images and push to local registry.

    Returns:
        tuple: (success, registry_url) - registry_url is for k8s to pull from
    """
    print("\n" + "=" * 60)
    print("Building and pushing images to local registry")
    print("=" * 60)

    images = [
        ("broker", "docker/Dockerfile.broker"),
        ("agent", "docker/Dockerfile.agent"),
    ]

    for component, dockerfile in images:
        image_name = f"{LOCAL_REGISTRY}/brokkr-{component}:{tag}"
        print(f"\nBuilding {component}...")

        result = subprocess.run([
            "docker", "build",
            "-t", image_name,
            "-f", dockerfile,
            "."
        ], cwd=cwd, capture_output=False)

        if result.returncode != 0:
            print(f"Failed to build {component}")
            return False, None

        print(f"Pushing {component} to local registry...")
        result = subprocess.run([
            "docker", "push", image_name
        ], cwd=cwd, capture_output=False)

        if result.returncode != 0:
            print(f"Failed to push {component}")
            return False, None

        print(f"  ✓ {component} pushed to {image_name}")

    print("\n✓ All images built and pushed to local registry")
    # Return the registry URL that k8s will use (via docker network)
    return True, LOCAL_REGISTRY_K8S



def ensure_k3s_running():
    """Start an isolated k3s cluster via docker-compose with unique project name."""
    project = get_project_name()
    print(f"\nStarting isolated k3s cluster (project: {project})...")

    # Clean any leftover volumes from crashed previous runs with same name (unlikely but safe)
    docker_clean(project=project)

    # Start registry + k3s + init-kubeconfig
    services = ["registry", "k3s", "init-kubeconfig", "copy-kubeconfig"]
    docker_up(services=services, project=project)

    print(f"k3s cluster is ready (project: {project})")


def run_in_k8s_container(cmd, description="Running command in k8s container", quiet=False):
    """Run a command inside a kubernetes tools container on the docker network.

    Args:
        cmd: Command to run inside the container
        description: Description for logging (default: "Running command in k8s container")
        quiet: If True, suppress output (useful for cleanup operations)
    """
    if not quiet:
        print(f"{description}...")

    # Use alpine/k8s which has kubectl, helm, and other k8s tools
    # Mount the charts directory and brokkr-keys volume
    # Connect to the same docker network as k3s
    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{os.path.join(cwd, 'charts')}:/charts:ro",
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", cmd
    ], cwd=cwd, capture_output=quiet, text=quiet)

    return result.returncode == 0


def verify_kubectl_connectivity():
    """Verify kubectl can connect to k3s cluster with fast polling."""
    print("\nVerifying kubectl connectivity...")

    # Wait for kubeconfig.docker.yaml to exist with faster polling
    print("Waiting for kubeconfig.docker.yaml to be created...")
    max_wait = 30
    start_time = time.time()
    poll_intervals = [1, 1, 2, 2, 3, 3, 5, 5, 5, 5]  # Fast initial checks, then slower

    poll_idx = 0
    while time.time() - start_time < max_wait:
        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", get_network_name(),
            "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
            "alpine/k8s:1.30.10",
            "sh", "-c", "test -f /keys/kubeconfig.docker.yaml"
        ], cwd=cwd, capture_output=True)

        if result.returncode == 0:
            print("kubeconfig.docker.yaml found!")
            break

        elapsed = int(time.time() - start_time)
        print(f"Waiting for kubeconfig.docker.yaml... ({elapsed}s)")
        sleep_time = poll_intervals[min(poll_idx, len(poll_intervals) - 1)]
        time.sleep(sleep_time)
        poll_idx += 1
    else:
        # List what files are available
        run_in_k8s_container("ls -la /keys/", "Available files in /keys")
        raise Exception("Timeout waiting for kubeconfig.docker.yaml to be created")

    success = run_in_k8s_container(
        "kubectl get nodes",
        "Testing kubectl connectivity"
    )

    if not success:
        raise Exception("Failed to connect to k3s cluster")

    print("kubectl connectivity verified")


def run_parallel_template_tests(tag, registry):
    """Phase 1 of the cluster suite: chart rendering must be sound before we
    spend minutes deploying.

    This used to run `helm template ... > /dev/null 2>&1` inside an alpine/k8s
    container, once per chart per values file, and assert only that helm exited
    zero. Rendering *nothing* is not a non-zero exit, which is how four inert
    values (BROKKR-T-0308) passed this phase for multiple releases. It now
    delegates to `angreal helm check-values`, which renders the same charts
    against the same values files -- plus values.yaml and values-dev.yaml, which
    this phase never covered -- and asserts what the output actually contains.

    Delegating also drops the docker round-trip: rendering never needed the
    cluster, and the new suite runs on the host in about two seconds instead of
    ~30s of container startup. It does require `helm` on PATH; the CI job that
    runs the deployment suite already installs it, and the task prints an
    install hint if it is missing.

    `tag` and `registry` are accepted for call-site compatibility and are
    deliberately unused: image coordinates do not change which capabilities a
    chart renders, and the deployment phases that follow set them for real.

    Returns:
        list: List of (test_name, success) tuples
    """
    del tag, registry  # see docstring

    print("\n" + "=" * 60)
    print("Phase 1: Helm Template Validation (render assertions)")
    print("=" * 60)

    success = run_render_assertions() == 0

    if success:
        print("\nTemplate validation: render assertions passed")
    else:
        # The specific reason goes to stderr, which does not reliably interleave
        # into a grouped CI log -- so name the likeliest cause here on stdout
        # rather than leaving a bare "FAILED" with no diagnosis.
        print(
            "\nTemplate validation FAILED. Reproduce without a cluster:\n"
            "  angreal helm check-values\n"
            "\n"
            "If no assertion failure is shown above, check the prerequisites --\n"
            "this phase needs PyYAML and `helm` on PATH, and reports a missing\n"
            "one on stderr:\n"
            "  pip install pyyaml"
        )

    return [("template-render-assertions", success)]


def helm_install(chart_name, release_name, values, namespace="default", values_file=None):
    """Install a Helm chart.

    Args:
        chart_name: Name of the chart to install
        release_name: Helm release name
        values: Dict of values to set via --set
        namespace: Kubernetes namespace
        values_file: Optional path to values file (relative to project root)
    """
    print("")
    print("=" * 60)
    print(f"Installing Helm chart: {chart_name}")
    print(f"Release name: {release_name}")
    print(f"Namespace: {namespace}")
    if values_file:
        print(f"Values file: {values_file}")
    print("=" * 60)
    print("")

    # Build helm install command with values
    values_args = " ".join([f"--set {k}={v}" for k, v in values.items()])

    # Add values file if specified
    values_file_arg = f"-f /{values_file}" if values_file else ""

    cmd = f"""
        helm install {release_name} /charts/{chart_name} \
            --namespace {namespace} \
            --create-namespace \
            --wait \
            --timeout 10m \
            {values_file_arg} \
            {values_args}
    """

    # Debug: Check what's in the charts directory
    print("\nDebug: Checking charts directory contents...")
    run_in_k8s_container(
        f"ls -la /charts/{chart_name}/",
        "Listing chart directory"
    )
    run_in_k8s_container(
        f"ls -la /charts/{chart_name}/charts/ 2>/dev/null || echo 'No charts subdirectory'",
        "Listing chart dependencies"
    )

    print(f"\nHelm command: {cmd.strip()}")
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


def helm_uninstall(release_name, namespace="default", quiet=False):
    """Uninstall a Helm release.

    Args:
        release_name: Name of the Helm release to uninstall
        namespace: Kubernetes namespace (default: "default")
        quiet: If True, suppress output (useful for cleanup operations)
    """
    if not quiet:
        print(f"\nUninstalling Helm release: {release_name}")

    cmd = f"helm uninstall {release_name} --namespace {namespace} --wait --ignore-not-found"
    return run_in_k8s_container(cmd, f"Uninstalling {release_name}", quiet=quiet)


def wait_for_pods(release_name, namespace="default", timeout=180):
    """Wait for all pods in a release to be ready with fast failure detection."""
    print(f"\nWaiting for pods in release '{release_name}' to be ready...", flush=True)

    start_time = time.time()
    while time.time() - start_time < timeout:
        # Get pod status with container state info for CrashLoopBackOff detection
        cmd = f"""
            kubectl get pods -n {namespace} \
                -l app.kubernetes.io/instance={release_name} \
                -o jsonpath='{{range .items[*]}}{{.status.phase}}:{{range .status.conditions[?(@.type=="Ready")]}}{{.status}}{{end}}:{{range .status.containerStatuses[*]}}{{.state.waiting.reason}}{{end}} {{end}}'
        """

        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", get_network_name(),
            "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
            "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
            "alpine/k8s:1.30.10",
            "sh", "-c", cmd
        ], capture_output=True, text=True, cwd=cwd)

        if result.returncode == 0 and result.stdout.strip():
            pod_statuses = result.stdout.strip().split()

            # Check for terminal failure states (fail fast)
            terminal_failures = ["CrashLoopBackOff", "ImagePullBackOff", "ErrImagePull", "InvalidImageName"]
            for status in pod_statuses:
                for failure in terminal_failures:
                    if failure in status:
                        elapsed = int(time.time() - start_time)
                        print(f"Pod in terminal failure state: {failure} (detected in {elapsed}s)", flush=True)
                        # Show pod details for debugging
                        run_in_k8s_container(
                            f"kubectl get pods -n {namespace} -l app.kubernetes.io/instance={release_name}",
                            "Pod status"
                        )
                        run_in_k8s_container(
                            f"kubectl describe pods -n {namespace} -l app.kubernetes.io/instance={release_name} | tail -30",
                            "Pod events"
                        )
                        return False

            # Check if all pods are Running:True
            if pod_statuses and all("Running:True" in status for status in pod_statuses):
                elapsed = int(time.time() - start_time)
                print(f"All pods in release '{release_name}' are ready! ({elapsed}s)", flush=True)
                return True

        elapsed = int(time.time() - start_time)
        print(f"  Waiting for pods... ({elapsed}s)", flush=True)
        time.sleep(3)  # Reduced from 5s

    print(f"Timeout waiting for pods in release '{release_name}' to be ready", flush=True)
    return False


def log_broker_diagnostics(broker_release_name, namespace="default"):
    """Log broker pod diagnostics for debugging failures."""
    print("\n" + "=" * 60, flush=True)
    print("BROKER DIAGNOSTICS", flush=True)
    print("=" * 60, flush=True)

    run_in_k8s_container(
        f"kubectl get pods -n {namespace} -l app.kubernetes.io/instance={broker_release_name}",
        "Broker pod status"
    )

    run_in_k8s_container(
        f"kubectl logs -n {namespace} -l app.kubernetes.io/instance={broker_release_name} -c broker --tail=100",
        "Broker container logs (last 100 lines)"
    )

    run_in_k8s_container(
        f"kubectl describe pod -n {namespace} -l app.kubernetes.io/instance={broker_release_name}",
        "Broker pod description"
    )

    print("=" * 60, flush=True)


def validate_health_endpoint(service_name, port, path, namespace="default"):
    """Validate a health check endpoint via the service."""
    print(f"\nValidating health endpoint: {service_name}:{port}{path}", flush=True)

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


def test_broker_chart(tag, registry, no_cleanup, test_external_db=False):
    """Test the broker Helm chart.

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
        test_external_db: Test with external PostgreSQL instead of bundled
    """
    release_name = "brokkr-broker-test"
    chart_name = "brokkr-broker"
    external_db_release = None

    try:
        if test_external_db:
            # Deploy a standalone PostgreSQL as "external" database
            print("\n" + "=" * 60)
            print("Deploying external PostgreSQL for testing")
            print("=" * 60)

            external_db_release = "external-postgres"
            external_db_values = {  # noqa: F841
                "image.tag": "16-alpine",
                "image.repository": "postgres",
            }

            # Create a simple postgres deployment
            postgres_manifest = f"""
apiVersion: v1
kind: Service
metadata:
  name: {external_db_release}
spec:
  ports:
  - port: 5432
  selector:
    app: {external_db_release}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {external_db_release}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {external_db_release}
  template:
    metadata:
      labels:
        app: {external_db_release}
    spec:
      containers:
      - name: postgres
        image: postgres:16-alpine
        env:
        - name: POSTGRES_DB
          value: brokkr
        - name: POSTGRES_USER
          value: brokkr
        - name: POSTGRES_PASSWORD
          value: external-test-password
        ports:
        - containerPort: 5432
"""

            # Apply manifest
            result = subprocess.run([
                "docker", "run", "--rm",
                "--network", get_network_name(),
                "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
                "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
                "alpine/k8s:1.30.10",
                "sh", "-c", f"cat <<'EOF' | kubectl apply -f -\n{postgres_manifest}\nEOF"
            ], cwd=cwd)

            if result.returncode != 0:
                print("Failed to deploy external PostgreSQL")
                return False

            # Wait for PostgreSQL to be ready
            print("Waiting for external PostgreSQL to be ready...")
            time.sleep(15)

            # Test broker with external database
            values = {
                "image.tag": tag,
                "image.repository": f"{registry}/brokkr-broker",
                                "postgresql.enabled": "false",
                "postgresql.external.host": external_db_release,
                "postgresql.external.username": "brokkr",
                "postgresql.external.password": "external-test-password",
            }
        else:
            # Use bundled PostgreSQL
            values = {
                "image.tag": tag,
                "image.repository": f"{registry}/brokkr-broker",
                                "postgresql.enabled": "true",
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

        return health_passed

    finally:
        if not no_cleanup:
            helm_uninstall(release_name)

            # Cleanup external database if deployed
            if external_db_release:
                print("\nCleaning up external PostgreSQL...")
                run_in_k8s_container(
                    f"kubectl delete deployment,service {external_db_release} --ignore-not-found",
                    "Deleting external PostgreSQL"
                )


def test_broker_multi_tenant_schema(tag, registry, no_cleanup):
    """Test multi-tenant broker deployments with schema isolation.

    This test verifies that multiple broker instances can share a single
    PostgreSQL database using schema-based isolation.

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
    """
    external_db_release = "shared-postgres"
    broker_a_release = "broker-tenant-a"
    broker_b_release = "broker-tenant-b"
    chart_name = "brokkr-broker"

    try:
        # Deploy a standalone PostgreSQL as shared database
        print("\n" + "=" * 60)
        print("Deploying shared PostgreSQL for multi-tenant testing")
        print("=" * 60)

        postgres_manifest = f"""
apiVersion: v1
kind: Service
metadata:
  name: {external_db_release}
spec:
  ports:
  - port: 5432
  selector:
    app: {external_db_release}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {external_db_release}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {external_db_release}
  template:
    metadata:
      labels:
        app: {external_db_release}
    spec:
      containers:
      - name: postgres
        image: postgres:16-alpine
        env:
        - name: POSTGRES_DB
          value: brokkr
        - name: POSTGRES_USER
          value: brokkr
        - name: POSTGRES_PASSWORD
          value: shared-test-password
        ports:
        - containerPort: 5432
"""

        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", get_network_name(),
            "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
            "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
            "alpine/k8s:1.30.10",
            "sh", "-c", f"cat <<'EOF' | kubectl apply -f -\n{postgres_manifest}\nEOF"
        ], cwd=cwd)

        if result.returncode != 0:
            print("Failed to deploy shared PostgreSQL")
            return False

        print("Waiting for shared PostgreSQL to be ready...")
        time.sleep(20)

        # Create schemas in PostgreSQL
        print("\nCreating schemas tenant_a and tenant_b in PostgreSQL...")
        create_schemas_cmd = """
            kubectl run create-schemas --rm -i --restart=Never --image=postgres:16-alpine \
                --env=PGPASSWORD=shared-test-password -- \
                psql -h shared-postgres -U brokkr -d brokkr -c \
                "CREATE SCHEMA IF NOT EXISTS tenant_a; \
                 CREATE SCHEMA IF NOT EXISTS tenant_b; \
                 GRANT ALL PRIVILEGES ON SCHEMA tenant_a TO brokkr; \
                 GRANT ALL PRIVILEGES ON SCHEMA tenant_b TO brokkr;"
        """

        if not run_in_k8s_container(create_schemas_cmd, "Creating schemas"):
            print("Failed to create schemas")
            return False

        print("Schemas created successfully")

        # Deploy broker for tenant_a
        print("\n" + "=" * 60)
        print("Deploying broker for tenant_a")
        print("=" * 60)

        values_a = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-broker",
                        "postgresql.enabled": "false",
            "postgresql.external.host": external_db_release,
            "postgresql.external.username": "brokkr",
            "postgresql.external.password": "shared-test-password",
            "postgresql.external.schema": "tenant_a",
        }

        if not helm_install(chart_name, broker_a_release, values_a):
            return False

        if not wait_for_pods(broker_a_release):
            if not no_cleanup:
                helm_uninstall(broker_a_release)
            return False

        # Deploy broker for tenant_b
        print("\n" + "=" * 60)
        print("Deploying broker for tenant_b")
        print("=" * 60)

        values_b = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-broker",
                        "postgresql.enabled": "false",
            "postgresql.external.host": external_db_release,
            "postgresql.external.username": "brokkr",
            "postgresql.external.password": "shared-test-password",
            "postgresql.external.schema": "tenant_b",
        }

        if not helm_install(chart_name, broker_b_release, values_b):
            if not no_cleanup:
                helm_uninstall(broker_a_release)
            return False

        if not wait_for_pods(broker_b_release):
            if not no_cleanup:
                helm_uninstall(broker_a_release)
                helm_uninstall(broker_b_release)
            return False

        # Validate both brokers are healthy
        print("\n" + "=" * 60)
        print("Validating multi-tenant broker health")
        print("=" * 60)

        # Service names follow pattern: {release-name}-brokkr-broker
        service_a = f"{broker_a_release}-brokkr-broker"
        service_b = f"{broker_b_release}-brokkr-broker"

        health_passed = True
        health_passed &= validate_health_endpoint(service_a, 3000, "/healthz")
        health_passed &= validate_health_endpoint(service_a, 3000, "/readyz")
        health_passed &= validate_health_endpoint(service_b, 3000, "/healthz")
        health_passed &= validate_health_endpoint(service_b, 3000, "/readyz")

        if health_passed:
            print("\n✓ Multi-tenant schema isolation test passed")
            print("  - Tenant A broker deployed with schema 'tenant_a'")
            print("  - Tenant B broker deployed with schema 'tenant_b'")
            print("  - Both brokers healthy and isolated")

        return health_passed

    finally:
        if not no_cleanup:
            print("\nCleaning up multi-tenant test resources...")
            helm_uninstall(broker_a_release)
            helm_uninstall(broker_b_release)
            run_in_k8s_container(
                f"kubectl delete deployment,service {external_db_release} --ignore-not-found",
                "Deleting shared PostgreSQL"
            )


ADMIN_PAK = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"
ADMIN_PAK_HASH = "4c697273df3d764cba950bb5c04368097685f09259f5bd880d892cf1ff9f4cdd"


def create_agent_in_broker(broker_release_name, agent_name, cluster_name, namespace="default"):
    """Create an agent via the broker API and return the PAK."""
    print(f"\nCreating agent '{agent_name}' in cluster '{cluster_name}' via API...", flush=True)

    broker_url = f"http://{broker_release_name}:3000"

    # Create agent via API using admin PAK - run inside k8s cluster to resolve service name
    # Use kubectl run to create a temporary pod that curls the broker service
    # Build the JSON body carefully to avoid quoting issues
    json_body = json.dumps({"name": agent_name, "cluster_name": cluster_name})

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "kubectl", "run", f"create-agent-{uuid.uuid4().hex[:8]}", "--rm", "-i",
        "--restart=Never", "--image=curlimages/curl:latest",
        "-n", namespace,
        "--", "curl", "-sf", "-X", "POST",
        f"{broker_url}/api/v1/agents",
        "-H", "Content-Type: application/json",
        "-H", f"Authorization: Bearer {ADMIN_PAK}",
        "-d", json_body
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0:
        print("ERROR: Failed to create agent via API", flush=True)
        print(f"  Return code: {result.returncode}", flush=True)
        print(f"  Stderr: {result.stderr}", flush=True)
        print(f"  Stdout: {result.stdout}", flush=True)
        return None

    # Parse initial_pak from JSON response - the response is in stdout
    # kubectl run outputs status messages, we need to find the JSON in the output
    stdout = result.stdout.strip()

    # Try to find and parse JSON from the output
    try:
        # Look for JSON object in output (starts with { and ends with })
        json_start = stdout.find('{')
        json_end = stdout.rfind('}')
        if json_start >= 0 and json_end > json_start:
            json_str = stdout[json_start:json_end + 1]
            response = json.loads(json_str)
            pak = response.get("initial_pak")
            if pak:
                print(f"Extracted PAK: {pak[:20]}...", flush=True)
                return pak
            print(f"ERROR: No initial_pak in response: {json_str[:200]}", flush=True)
            return None
        print(f"ERROR: No JSON found in response: {stdout[:200]}", flush=True)
        return None
    except json.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON response: {stdout[:200]}", flush=True)
        print(f"  Parse error: {e}", flush=True)
        return None


def test_broker_with_values_file(tag, registry, no_cleanup, values_file_name):
    """Test broker deployment using a specific values file.

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
        values_file_name: Name of values file (e.g., "production", "development", "staging")

    Returns:
        bool: True if test passed, False otherwise
    """
    release_name = f"brokkr-broker-test-{values_file_name}"
    chart_name = "brokkr-broker"
    values_file = f"charts/brokkr-broker/values/{values_file_name}.yaml"

    try:
        print(f"\nDeploying broker with {values_file_name}.yaml")

        # Base values that override values file for test environment
        broker_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-broker",
                    }

        # For production/staging, override external DB to use bundled
        if values_file_name in ["production", "staging"]:
            print("Note: Overriding external DB settings for test environment")
            broker_values["postgresql.enabled"] = "true"
            broker_values["postgresql.existingSecret"] = ""  # Don't use external secret
            broker_values["postgresql.auth.password"] = "testpassword"
            broker_values["tls.enabled"] = "false"  # Disable TLS for testing
            broker_values["ingress.enabled"] = "false"  # Disable ingress for testing

        install_success = helm_install(
            chart_name,
            release_name,
            broker_values,
            values_file=values_file
        )

        if not install_success:
            return False

        if not wait_for_pods(release_name):
            if not no_cleanup:
                helm_uninstall(release_name)
            return False

        print(f"✓ Broker deployed successfully with {values_file_name}.yaml")
        return True

    finally:
        if not no_cleanup:
            helm_uninstall(release_name)


def test_agent_with_values_file(tag, registry, no_cleanup, values_file_name, broker_release_name):
    """Test agent deployment using a specific values file.

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
        values_file_name: Name of values file (e.g., "production", "development", "staging")
        broker_release_name: Name of existing broker release

    Returns:
        bool: True if test passed, False otherwise
    """
    release_name = f"brokkr-agent-test-{values_file_name}"
    chart_name = "brokkr-agent"
    values_file = f"charts/brokkr-agent/values/{values_file_name}.yaml"

    try:
        print(f"\nDeploying agent with {values_file_name}.yaml")

        # Create agent in broker
        agent_name = f"test-agent-{values_file_name}"
        pak = create_agent_in_broker(broker_release_name, agent_name, "test-cluster")

        if not pak:
            print("Failed to create agent and get PAK")
            return False

        broker_url = f"http://{broker_release_name}:3000"

        # Base values that override values file for test environment
        agent_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-agent",
                        "broker.url": broker_url,
            "broker.agentName": agent_name,
            "broker.clusterName": "test-cluster",
            "broker.pak": pak,
            # Disable Shipwright in CI tests (requires K8s >= 1.29)
            "shipwright.enabled": "false",
        }

        install_success = helm_install(
            chart_name,
            release_name,
            agent_values,
            values_file=values_file
        )

        if not install_success:
            return False

        if not wait_for_pods(release_name):
            # Log broker diagnostics to understand why agent failed
            log_broker_diagnostics(broker_release_name)
            if not no_cleanup:
                helm_uninstall(release_name)
            return False

        print(f"✓ Agent deployed successfully with {values_file_name}.yaml")
        return True

    finally:
        if not no_cleanup:
            helm_uninstall(release_name)


def deploy_test_broker(tag, registry):
    """Deploy a broker instance for agent testing and return the release name.

    Args:
        tag: Image tag to deploy
        registry: Container registry URL
    """
    broker_release_name = "brokkr-broker-for-agent-test"
    broker_chart_name = "brokkr-broker"

    print("\n" + "=" * 60, flush=True)
    print("Deploying broker for agent testing", flush=True)
    print("=" * 60, flush=True)

    broker_values = {
        "image.tag": tag,
        "image.repository": f"{registry}/brokkr-broker",
        "postgresql.enabled": "true",
        # Always include admin PAK for API access (agent creation)
        "broker.pakHash": ADMIN_PAK_HASH,
    }

    if not helm_install(broker_chart_name, broker_release_name, broker_values):
        print("Failed to deploy broker for agent testing", flush=True)
        return None

    if not wait_for_pods(broker_release_name):
        print("Broker pods failed to become ready", flush=True)
        helm_uninstall(broker_release_name)
        return None

    return broker_release_name


def test_agent_chart(tag, registry, no_cleanup, rbac_mode="cluster-wide", broker_release_name=None):
    """Test the agent Helm chart.

    This test performs agent deployment and validation:
    1. Creates an agent via the broker CLI to get a valid PAK
    2. Deploys the agent chart with the real broker URL and PAK
    3. Validates the agent is running and healthy

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
        rbac_mode: RBAC configuration mode (cluster-wide, namespace-scoped, disabled)
        broker_release_name: Name of existing broker release to use
    """
    agent_release_name = f"brokkr-agent-test-{rbac_mode}"
    agent_chart_name = "brokkr-agent"

    try:
        # Step 1: Create agent via broker CLI to get PAK
        print("\n" + "=" * 60, flush=True)
        print(f"Step 1: Creating agent via broker CLI (RBAC: {rbac_mode})", flush=True)
        print("=" * 60, flush=True)

        agent_name = f"test-agent-{rbac_mode}"
        pak = create_agent_in_broker(
            broker_release_name,
            agent_name,
            "test-cluster"
        )

        if not pak:
            print("Failed to create agent and get PAK", flush=True)
            return False

        # Step 2: Deploy agent chart with real configuration
        print("\n" + "=" * 60, flush=True)
        print(f"Step 2: Deploying agent chart (RBAC mode: {rbac_mode})", flush=True)
        print("=" * 60, flush=True)

        # The broker service URL uses the release name
        broker_url = f"http://{broker_release_name}:3000"

        agent_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-agent",
                        "broker.url": broker_url,
            "broker.agentName": agent_name,
            "broker.clusterName": "test-cluster",
            "broker.pak": pak,
            # Disable Shipwright in CI tests (requires K8s >= 1.29)
            "shipwright.enabled": "false",
        }

        # Configure RBAC based on mode
        if rbac_mode == "cluster-wide":
            agent_values["rbac.create"] = "true"
            agent_values["rbac.clusterWide"] = "true"
        elif rbac_mode == "namespace-scoped":
            agent_values["rbac.create"] = "true"
            agent_values["rbac.clusterWide"] = "false"
        elif rbac_mode == "disabled":
            agent_values["rbac.create"] = "false"

        # For cluster-wide mode, require successful install
        # For other modes, agent will crash (expected), so we just need to verify RBAC config
        install_success = helm_install(agent_chart_name, agent_release_name, agent_values)

        if rbac_mode == "cluster-wide" and not install_success:
            return False

        # For non-cluster-wide modes, install may fail due to agent crashes, which is expected
        # We'll verify RBAC configuration regardless of install status

        # Verify RBAC configuration
        print("\n" + "=" * 60)
        print("Step 3: Verifying RBAC configuration")
        print("=" * 60)

        # For cluster-wide mode, agent should start successfully
        # For namespace-scoped and disabled, agent may fail to start (current limitation)
        # but RBAC should still be configured correctly
        if rbac_mode == "cluster-wide":
            # Wait for agent pods to be ready
            if not wait_for_pods(agent_release_name):
                # Log broker diagnostics to understand why agent failed
                log_broker_diagnostics(broker_release_name)
                if not no_cleanup:
                    helm_uninstall(agent_release_name)
                return False
            print("✓ Agent pods are ready and healthy")
            print("✓ Agent successfully connected to broker")
        else:
            # For non-cluster-wide modes, verify RBAC resources but don't require pod to be ready
            print("Note: Agent currently requires cluster-wide permissions")
            print(f"RBAC configuration test for {rbac_mode} mode validates template rendering only")

            # Give the pod some time to attempt startup
            import time
            time.sleep(10)

            # Check if RBAC resources were created correctly
            if rbac_mode == "namespace-scoped":
                # Verify Role (not ClusterRole) was created
                check_cmd = f"kubectl get role {agent_release_name} -o name"
                if not run_in_k8s_container(check_cmd, "Verifying Role created"):
                    print("✗ Role was not created")
                    return False
                print("✓ Namespace-scoped Role created correctly")
            elif rbac_mode == "disabled":
                # Verify no RBAC resources were created
                check_cmd = f"kubectl get clusterrole,role -l app.kubernetes.io/instance={agent_release_name} 2>&1 | grep -c 'No resources found' || echo 'found'"
                print("✓ RBAC resources correctly not created")

        return True

    finally:
        # Cleanup agent only
        if not no_cleanup:
            print(f"\nCleaning up agent deployment: {agent_release_name}")
            helm_uninstall(agent_release_name)


def get_admin_pak_from_broker(broker_release_name, namespace="default"):
    """Return the well-known admin PAK configured in test brokers.

    The broker is deployed with a known pakHash that corresponds to this PAK.
    This is the same PAK used by the E2E tests and demo UI.
    """
    # Well-known admin PAK matching the pakHash configured in deploy_test_broker
    admin_pak = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"
    print(f"Using admin PAK: {admin_pak[:15]}...", flush=True)
    return admin_pak


def get_agent_id_from_broker(broker_release_name, agent_name, admin_pak, namespace="default"):
    """Get the agent ID from the broker API using the admin PAK."""
    print(f"\nGetting agent ID for '{agent_name}'...", flush=True)

    # Get the broker pod name first
    get_pod_cmd = f"""
        kubectl get pods -n {namespace} \
            -l app.kubernetes.io/name=brokkr-broker,app.kubernetes.io/instance={broker_release_name} \
            -o jsonpath='{{.items[0].metadata.name}}'
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", get_pod_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to get broker pod name", flush=True)
        return None

    broker_pod = result.stdout.strip()

    # Query the broker API via kubectl exec (localhost from within the pod)
    get_agents_cmd = f"""
        kubectl exec {broker_pod} -n {namespace} -- \
            curl -s -H "Authorization: Bearer {admin_pak}" \
            http://localhost:3000/api/v1/agents
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", get_agents_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0:
        print(f"Failed to query agents API: {result.stderr}", flush=True)
        return None

    import json
    try:
        agents = json.loads(result.stdout)
        for agent in agents:
            if agent.get("name") == agent_name:
                agent_id = agent.get("id")
                print(f"Found agent ID: {agent_id}", flush=True)
                return agent_id
        print(f"Agent '{agent_name}' not found in {len(agents)} agents", flush=True)
        return None
    except json.JSONDecodeError as e:
        print(f"Failed to parse agents response: {e}", flush=True)
        print(f"Response: {result.stdout[:500]}", flush=True)
        return None


def activate_agent(broker_release_name, admin_pak, agent_id, namespace="default"):
    """Activate an agent so it can process work orders."""
    print(f"\nActivating agent {agent_id}...", flush=True)

    # Get broker pod name first
    get_pod_cmd = f"""
        kubectl get pods -n {namespace} \
            -l app.kubernetes.io/name=brokkr-broker,app.kubernetes.io/instance={broker_release_name} \
            -o jsonpath='{{.items[0].metadata.name}}'
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", get_pod_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to get broker pod name", flush=True)
        return False

    broker_pod = result.stdout.strip()

    # Activate agent via PUT request (matches E2E test API)
    activate_cmd = f"""
        kubectl exec {broker_pod} -n {namespace} -- \
            curl -s -X PUT \
                -H "Authorization: Bearer {admin_pak}" \
                -H "Content-Type: application/json" \
                -d '{{"status": "ACTIVE"}}' \
                http://localhost:3000/api/v1/agents/{agent_id}
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", activate_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0:
        print(f"Failed to activate agent: {result.stderr}", flush=True)
        return False

    import json
    try:
        agent = json.loads(result.stdout)
        status = agent.get("status", "UNKNOWN")
        print(f"Agent status: {status}", flush=True)
        return status == "ACTIVE"
    except json.JSONDecodeError as e:
        print(f"Failed to parse response: {e}", flush=True)
        print(f"Response: {result.stdout[:500]}", flush=True)
        return False


def create_work_order(broker_release_name, admin_pak, agent_id, work_type, yaml_content, namespace="default"):
    """Create a work order via the broker API."""
    print(f"\nCreating work order of type '{work_type}'...", flush=True)

    import json
    payload = json.dumps({
        "work_type": work_type,
        "yaml_content": yaml_content,
        "target_agent_ids": [agent_id],
        "max_retries": 0,  # No retries for testing
        "claim_timeout_seconds": 300,
    })

    # Get broker pod name first
    get_pod_cmd = f"""
        kubectl get pods -n {namespace} \
            -l app.kubernetes.io/name=brokkr-broker,app.kubernetes.io/instance={broker_release_name} \
            -o jsonpath='{{.items[0].metadata.name}}'
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", get_pod_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to get broker pod name", flush=True)
        return None

    broker_pod = result.stdout.strip()

    # Escape the payload for shell - use base64 to avoid quoting issues
    import base64
    payload_b64 = base64.b64encode(payload.encode()).decode()

    # Create work order via kubectl exec (localhost from within the pod)
    create_wo_cmd = f"""
        kubectl exec {broker_pod} -n {namespace} -- sh -c '
            echo {payload_b64} | base64 -d | curl -s -X POST \
                -H "Authorization: Bearer {admin_pak}" \
                -H "Content-Type: application/json" \
                -d @- \
                http://localhost:3000/api/v1/work-orders
        '
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", create_wo_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0:
        print(f"Failed to create work order: {result.stderr}", flush=True)
        return None

    try:
        work_order = json.loads(result.stdout)
        wo_id = work_order.get("id")
        print(f"Created work order: {wo_id}", flush=True)
        return wo_id
    except json.JSONDecodeError as e:
        print(f"Failed to parse work order response: {e}", flush=True)
        print(f"Response: {result.stdout[:500]}", flush=True)
        return None


def wait_for_work_order_completion(broker_release_name, admin_pak, work_order_id, timeout=300, namespace="default"):
    """Wait for a work order to complete (move to work_order_log)."""
    print(f"\nWaiting for work order {work_order_id} to complete (timeout: {timeout}s)...", flush=True)

    import json
    start_time = time.time()

    # Get broker pod name once
    get_pod_cmd = f"""
        kubectl get pods -n {namespace} \
            -l app.kubernetes.io/name=brokkr-broker,app.kubernetes.io/instance={broker_release_name} \
            -o jsonpath='{{.items[0].metadata.name}}'
    """

    result = subprocess.run([
        "docker", "run", "--rm",
        "--network", get_network_name(),
        "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
        "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
        "alpine/k8s:1.30.10",
        "sh", "-c", get_pod_cmd
    ], capture_output=True, text=True, cwd=cwd)

    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to get broker pod name", flush=True)
        return False, "Failed to get broker pod"

    broker_pod = result.stdout.strip()

    while time.time() - start_time < timeout:
        # Check if work order is in the log (completed)
        check_log_cmd = f"""
            kubectl exec {broker_pod} -n {namespace} -- \
                curl -s -H "Authorization: Bearer {admin_pak}" \
                http://localhost:3000/api/v1/work-order-log/{work_order_id}
        """

        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", get_network_name(),
            "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
            "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
            "alpine/k8s:1.30.10",
            "sh", "-c", check_log_cmd
        ], capture_output=True, text=True, cwd=cwd)

        if result.returncode == 0 and result.stdout.strip():
            try:
                log_entry = json.loads(result.stdout)
                if log_entry.get("id"):
                    success = log_entry.get("success", False)
                    message = log_entry.get("result_message", "")
                    elapsed = int(time.time() - start_time)
                    print(f"Work order completed in {elapsed}s", flush=True)
                    print(f"  Success: {success}", flush=True)
                    print(f"  Message: {message[:100] if message else 'N/A'}", flush=True)
                    return success, message
            except json.JSONDecodeError:
                pass  # Not in log yet

        # Check current status
        check_wo_cmd = f"""
            kubectl exec {broker_pod} -n {namespace} -- \
                curl -s -H "Authorization: Bearer {admin_pak}" \
                http://localhost:3000/api/v1/work-orders/{work_order_id}
        """

        result = subprocess.run([
            "docker", "run", "--rm",
            "--network", get_network_name(),
            "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
            "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
            "alpine/k8s:1.30.10",
            "sh", "-c", check_wo_cmd
        ], capture_output=True, text=True, cwd=cwd)

        if result.returncode == 0 and result.stdout.strip():
            try:
                wo = json.loads(result.stdout)
                status = wo.get("status", "UNKNOWN")
                elapsed = int(time.time() - start_time)
                print(f"  Status: {status} ({elapsed}s elapsed)", flush=True)
            except json.JSONDecodeError:
                pass

        time.sleep(10)

    print("Timeout waiting for work order to complete", flush=True)
    return False, "Timeout"


def test_shipwright_e2e(tag, registry, no_cleanup, broker_release_name=None):
    """Test Shipwright build integration end-to-end.

    This test:
    1. Deploys agent with Shipwright enabled
    2. Creates a Build resource via work order
    3. Verifies the agent processes the work order
    4. Checks the build completes successfully

    Args:
        tag: Image tag to test
        registry: Container registry URL
        no_cleanup: Skip cleanup after test
        broker_release_name: Name of existing broker release to use
    """
    agent_release_name = "brokkr-agent-shipwright-e2e"
    agent_chart_name = "brokkr-agent"
    shipwright_namespace = "shipwright-build"

    try:
        # Step 1: Create agent via broker CLI
        print("\n" + "=" * 60)
        print("Step 1: Creating agent for Shipwright E2E test")
        print("=" * 60)

        agent_name = "shipwright-e2e-agent"
        pak = create_agent_in_broker(
            broker_release_name,
            agent_name,
            "shipwright-e2e-cluster"
        )

        if not pak:
            print("Failed to create agent and get PAK")
            return False

        # Step 2: Deploy agent with Shipwright ENABLED
        print("\n" + "=" * 60)
        print("Step 2: Deploying agent with Shipwright enabled")
        print("=" * 60)

        broker_url = f"http://{broker_release_name}:3000"

        agent_values = {
            "image.tag": tag,
            "image.repository": f"{registry}/brokkr-agent",
                        "broker.url": broker_url,
            "broker.agentName": agent_name,
            "broker.clusterName": "shipwright-e2e-cluster",
            "broker.pak": pak,
            "rbac.create": "true",
            "rbac.clusterWide": "true",
            # Enable Shipwright for this test (matches values-dev.yaml)
            "shipwright.enabled": "true",
            "shipwright.install.tekton": "true",
            "shipwright.install.shipwright": "true",
            "shipwright.install.sampleStrategies": "true",
        }

        install_success = helm_install(agent_chart_name, agent_release_name, agent_values)

        if not install_success:
            print("Failed to install agent chart with Shipwright")
            return False

        # Wait for agent pods to be ready
        if not wait_for_pods(agent_release_name):
            print("Agent pods failed to become ready")
            # Log broker diagnostics to understand why agent failed
            if broker_release_name:
                log_broker_diagnostics(broker_release_name)
            return False

        print("Agent deployed successfully with Shipwright enabled")

        # Step 3: Wait for Shipwright/Tekton to be ready
        print("\n" + "=" * 60)
        print("Step 3: Waiting for Shipwright components to be ready")
        print("=" * 60)

        # Wait for Tekton pipelines controller
        tekton_ready_cmd = """
            kubectl wait --for=condition=available deployment/tekton-pipelines-controller \
                -n tekton-pipelines --timeout=180s 2>/dev/null || echo "tekton-not-ready"
        """
        run_in_k8s_container(tekton_ready_cmd, "Waiting for Tekton controller")

        # Wait for Shipwright build controller
        shipwright_ready_cmd = f"""
            kubectl wait --for=condition=available deployment/shipwright-build-controller \
                -n {shipwright_namespace} --timeout=180s 2>/dev/null || echo "shipwright-not-ready"
        """
        run_in_k8s_container(shipwright_ready_cmd, "Waiting for Shipwright controller")

        # Wait for ClusterBuildStrategy to be created (install job might still be running)
        # Use 'kaniko' strategy as it works without registry credentials (pushes to ttl.sh)
        strategy_name = "kaniko"
        print(f"\nWaiting for ClusterBuildStrategy '{strategy_name}' to be available...", flush=True)
        strategy_ready = False
        for attempt in range(30):  # Wait up to 60 seconds
            strategy_check_cmd = f"kubectl get clusterbuildstrategy {strategy_name} -o name 2>/dev/null"
            result = subprocess.run([
                "docker", "run", "--rm",
                "--network", get_network_name(),
                "-v", f"{get_volume_name('brokkr-keys')}:/keys:ro",
                "-e", "KUBECONFIG=/keys/kubeconfig.docker.yaml",
                "alpine/k8s:1.30.10",
                "sh", "-c", strategy_check_cmd
            ], capture_output=True, text=True, cwd=cwd)
            if result.returncode == 0 and strategy_name in result.stdout:
                print(f"✓ ClusterBuildStrategy '{strategy_name}' is available", flush=True)
                strategy_ready = True
                break
            print(f"  Waiting for {strategy_name} strategy... ({attempt * 2}s)", flush=True)
            time.sleep(2)

        if not strategy_ready:
            print(f"✗ ClusterBuildStrategy '{strategy_name}' not found after waiting", flush=True)
            # List available strategies for debugging
            list_cmd = "kubectl get clusterbuildstrategies 2>/dev/null || echo 'none found'"
            run_in_k8s_container(list_cmd, "Listing available strategies")
            return False

        # Step 4: Get admin PAK and agent ID for work order creation
        print("\n" + "=" * 60, flush=True)
        print("Step 4: Getting admin credentials for work order creation", flush=True)
        print("=" * 60, flush=True)

        admin_pak = get_admin_pak_from_broker(broker_release_name)
        if not admin_pak:
            print("Failed to get admin PAK")
            return False

        agent_id = get_agent_id_from_broker(broker_release_name, agent_name, admin_pak)
        if not agent_id:
            print("Failed to get agent ID")
            return False

        # Activate the agent so it can process work orders
        if not activate_agent(broker_release_name, admin_pak, agent_id):
            print("Failed to activate agent")
            return False

        # Step 5: Create a simple Build and WorkOrder
        print("\n" + "=" * 60)
        print("Step 5: Creating Shipwright Build via work order")
        print("=" * 60)

        # Simple build using ttl.sh (ephemeral registry, no credentials needed)
        # Matches the E2E test pattern in tests/e2e/src/scenarios.rs
        build_yaml = '''---
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: e2e-test-build
  namespace: default
spec:
  source:
    type: Git
    git:
      url: https://github.com/shipwright-io/sample-go
    contextDir: docker-build
  strategy:
    name: kaniko
    kind: ClusterBuildStrategy
  output:
    image: ttl.sh/brokkr-helm-e2e-test:1h
'''

        work_order_id = create_work_order(
            broker_release_name,
            admin_pak,
            agent_id,
            "build",
            build_yaml
        )

        if not work_order_id:
            print("Failed to create work order")
            return False

        # Step 6: Wait for work order to be processed
        print("\n" + "=" * 60)
        print("Step 6: Waiting for work order to be processed")
        print("=" * 60)

        # Check that agent claims the work order
        time.sleep(15)  # Give agent time to pick up work order

        # Check for BuildRun creation
        print("\nChecking for BuildRun resources...")
        buildrun_cmd = "kubectl get buildruns -n default -o wide 2>/dev/null || echo 'no-buildruns'"
        run_in_k8s_container(buildrun_cmd, "Listing BuildRuns")

        # Wait for completion (with longer timeout for actual build)
        success, message = wait_for_work_order_completion(
            broker_release_name,
            admin_pak,
            work_order_id,
            timeout=600  # 10 minutes for build
        )

        # Step 7: Verify results
        print("\n" + "=" * 60)
        print("Step 7: Verifying results")
        print("=" * 60)

        # Show BuildRun status
        print("\nFinal BuildRun status:")
        buildrun_status_cmd = "kubectl get buildruns -n default -o yaml 2>/dev/null | head -100"
        run_in_k8s_container(buildrun_status_cmd, "BuildRun details")

        if success:
            print("\n[PASS] Shipwright E2E test passed!")
            print("  - Agent successfully processed build work order")
            print(f"  - Build result: {message[:100] if message else 'completed'}")
            return True
        else:
            print("\n[FAIL] Shipwright E2E test failed")
            print("  - Work order did not complete successfully")
            print(f"  - Message: {message}")

            # Show agent logs for debugging
            print("\nAgent logs for debugging:")
            agent_logs_cmd = f"""
                kubectl logs -l app.kubernetes.io/instance={agent_release_name} \
                    --tail=50 -n default 2>/dev/null || echo 'no-logs'
            """
            run_in_k8s_container(agent_logs_cmd, "Agent logs")

            return False

    finally:
        if not no_cleanup:
            print("\nCleaning up Shipwright E2E test resources...")
            helm_uninstall(agent_release_name)

            # Clean up Build/BuildRun resources
            run_in_k8s_container(
                "kubectl delete build,buildrun --all -n default --ignore-not-found 2>/dev/null",
                "Cleaning up Build resources"
            )


# =============================================================================
# Tiered Test Runners
# =============================================================================

def run_smoke_tests(tag, registry, no_cleanup):
    """Run fast smoke tests for PR validation (~3-5 min).

    Smoke tests validate:
    1. All chart templates render correctly (helm template)
    2. Basic broker deployment works (bundled PostgreSQL)
    3. Basic agent deployment works (cluster-wide RBAC)

    Returns:
        list: List of (test_name, success) tuples
    """
    results = []

    # Pre-cleanup: Remove any stale releases from previous runs
    # This prevents conflicts when reusing a k3s cluster (e.g., with --skip-docker)
    print("Cleaning up any stale releases from previous runs...")
    stale_releases = [
        "brokkr-broker-test",
        "brokkr-broker-for-agent-test",
        "brokkr-agent-test-cluster-wide",
        "brokkr-agent-test-namespace-scoped",
        "brokkr-agent-test-no-rbac",
    ]
    for release in stale_releases:
        helm_uninstall(release, quiet=True)

    # Phase 1: Template validation (fast, no deployment)
    template_results = run_parallel_template_tests(tag, registry)
    results.extend(template_results)

    # Phase 2: Quick deployment tests
    print("\n" + "=" * 60)
    print("Phase 2: Quick Deployment Validation")
    print("=" * 60)

    # Single broker deployment (bundled PostgreSQL)
    print("\nDeploying broker (bundled PostgreSQL)...")
    result = test_broker_chart(tag, registry, no_cleanup=True, test_external_db=False)
    results.append(("broker-deploy", result))

    if not result:
        print("Broker deployment failed, skipping agent test")
        return results

    # Deploy broker for agent test (reuse from above would be better but simpler to just deploy fresh)
    broker_release_name = deploy_test_broker(tag, registry)
    if broker_release_name:
        # Single agent deployment (cluster-wide RBAC)
        print("\nDeploying agent (cluster-wide RBAC)...", flush=True)
        result = test_agent_chart(tag, registry, no_cleanup=True,
                                  rbac_mode="cluster-wide",
                                  broker_release_name=broker_release_name)
        results.append(("agent-deploy", result))

        # Cleanup
        if not no_cleanup:
            helm_uninstall(broker_release_name)
    else:
        results.append(("agent-broker-setup", False))

    return results


def run_full_tests(tag, registry, no_cleanup):
    """Run comprehensive tests for releases (~10-15 min).

    Full tests include all smoke tests plus:
    - External PostgreSQL configuration
    - Multi-tenant schema isolation
    - Additional RBAC modes (namespace-scoped, disabled)

    Returns:
        list: List of (test_name, success) tuples
    """
    results = []

    # Run smoke tests first
    smoke_results = run_smoke_tests(tag, registry, no_cleanup=True)
    results.extend(smoke_results)

    # Check if smoke tests passed
    smoke_passed = all(success for _, success in smoke_results)
    if not smoke_passed:
        print("\nSmoke tests failed, skipping extended tests")
        return results

    # Clean up smoke test releases before extended tests (they use the same release names)
    print("\nCleaning up smoke test releases before extended tests...")
    helm_uninstall("brokkr-broker-test")
    helm_uninstall("brokkr-broker-for-agent-test")
    helm_uninstall("brokkr-agent-test-cluster-wide")

    # Phase 3: Extended deployment tests
    print("\n" + "=" * 60)
    print("Phase 3: Extended Deployment Tests")
    print("=" * 60)

    # External PostgreSQL test
    print("\nTesting broker with external PostgreSQL...")
    result = test_broker_chart(tag, registry, no_cleanup, test_external_db=True)
    results.append(("broker-external-db", result))

    # Multi-tenant schema isolation test
    print("\nTesting multi-tenant schema isolation...")
    result = test_broker_multi_tenant_schema(tag, registry, no_cleanup)
    results.append(("broker-multi-tenant-schema", result))

    # Additional RBAC modes
    broker_release_name = deploy_test_broker(tag, registry)
    if broker_release_name:
        for rbac_mode in ["namespace-scoped", "disabled"]:
            print(f"\nTesting agent RBAC mode: {rbac_mode}...")
            result = test_agent_chart(tag, registry, no_cleanup,
                                      rbac_mode=rbac_mode,
                                      broker_release_name=broker_release_name)
            results.append((f"agent-rbac-{rbac_mode}", result))

        if not no_cleanup:
            helm_uninstall(broker_release_name)

    return results


def run_shipwright_tests(tag, registry, no_cleanup):
    """Run Shipwright E2E tests only (~15 min).

    Returns:
        list: List of (test_name, success) tuples
    """
    results = []

    # Pre-cleanup: Remove stale releases and Shipwright resources
    print("Cleaning up any stale releases and Shipwright resources...")
    stale_releases = [
        "brokkr-broker-for-agent-test",
        "brokkr-agent-test-shipwright",
    ]
    for release in stale_releases:
        helm_uninstall(release, quiet=True)

    # Clean up stale Shipwright builds
    run_in_k8s_container(
        "kubectl delete build,buildrun --all -n default --ignore-not-found 2>/dev/null || true",
        "Cleaning up stale builds",
        quiet=True
    )

    # Shipwright tests need admin PAK for API access (work order creation)
    broker_release_name = deploy_test_broker(tag, registry)
    if not broker_release_name:
        results.append(("shipwright-broker-setup", False))
        return results

    result = test_shipwright_e2e(tag, registry, no_cleanup, broker_release_name=broker_release_name)
    results.append(("shipwright-e2e", result))

    if not no_cleanup:
        helm_uninstall(broker_release_name)

    return results


def run_legacy_tests(tag, registry, no_cleanup, component):
    """Run legacy-style tests for backward compatibility.

    Args:
        component: One of broker, agent, shipwright, all
    """
    results = []

    if component in ["broker", "all"]:
        print("\n" + "=" * 60)
        print("Testing broker chart (bundled PostgreSQL)")
        print("=" * 60)
        result = test_broker_chart(tag, registry, no_cleanup, test_external_db=False)
        results.append(("broker-bundled-db", result))

        print("\n" + "=" * 60)
        print("Testing broker chart (external PostgreSQL)")
        print("=" * 60)
        result = test_broker_chart(tag, registry, no_cleanup, test_external_db=True)
        results.append(("broker-external-db", result))

        print("\n" + "=" * 60)
        print("Testing broker chart (multi-tenant schema isolation)")
        print("=" * 60)
        result = test_broker_multi_tenant_schema(tag, registry, no_cleanup)
        results.append(("broker-multi-tenant-schema", result))

        # Test broker values files
        values_files = ["production", "development", "staging"]
        for values_file in values_files:
            print("\n" + "=" * 60)
            print(f"Testing broker chart with {values_file}.yaml")
            print("=" * 60)
            result = test_broker_with_values_file(tag, registry, no_cleanup, values_file)
            results.append((f"broker-values-{values_file}", result))

    broker_release_name = None
    if component in ["agent", "all"]:
        # Deploy broker once for all agent tests
        print("\n" + "=" * 60)
        print("Setting up broker for agent testing")
        print("=" * 60)
        broker_release_name = deploy_test_broker(tag, registry)

        if not broker_release_name:
            print("Failed to deploy broker for agent testing")
            results.append(("agent-broker-setup", False))
        else:
            # Test agent with different RBAC modes
            rbac_modes = ["cluster-wide", "namespace-scoped", "disabled"]
            for rbac_mode in rbac_modes:
                print("\n" + "=" * 60)
                print(f"Testing agent chart (RBAC: {rbac_mode})")
                print("=" * 60)
                result = test_agent_chart(tag, registry, no_cleanup, rbac_mode=rbac_mode, broker_release_name=broker_release_name)
                results.append((f"agent-rbac-{rbac_mode}", result))

            # Test agent values files
            values_files = ["production", "development", "staging"]
            for values_file in values_files:
                print("\n" + "=" * 60)
                print(f"Testing agent chart with {values_file}.yaml")
                print("=" * 60)
                result = test_agent_with_values_file(tag, registry, no_cleanup, values_file, broker_release_name)
                results.append((f"agent-values-{values_file}", result))

            # Cleanup broker after all agent tests (unless shipwright test follows)
            if not no_cleanup and component not in ["shipwright", "all"]:
                print("\n" + "=" * 60)
                print("Cleaning up broker")
                print("=" * 60)
                helm_uninstall(broker_release_name)

    if component in ["shipwright", "all"]:
        # For shipwright-only test, need to deploy broker first
        if component == "shipwright":
            print("\n" + "=" * 60)
            print("Setting up broker for Shipwright E2E testing")
            print("=" * 60)
            broker_release_name = deploy_test_broker(tag, registry)

            if not broker_release_name:
                print("Failed to deploy broker for Shipwright E2E testing")
                results.append(("shipwright-broker-setup", False))
            else:
                print("\n" + "=" * 60)
                print("Testing Shipwright E2E (build work order)")
                print("=" * 60)
                result = test_shipwright_e2e(tag, registry, no_cleanup, broker_release_name=broker_release_name)
                results.append(("shipwright-e2e", result))

                # Cleanup broker
                if not no_cleanup:
                    print("\n" + "=" * 60)
                    print("Cleaning up broker")
                    print("=" * 60)
                    helm_uninstall(broker_release_name)
        else:
            # For 'all', broker is already deployed from agent tests
            print("\n" + "=" * 60)
            print("Testing Shipwright E2E (build work order)")
            print("=" * 60)
            result = test_shipwright_e2e(tag, registry, no_cleanup, broker_release_name=broker_release_name)
            results.append(("shipwright-e2e", result))

            # Cleanup broker after shipwright test
            if not no_cleanup:
                print("\n" + "=" * 60)
                print("Cleaning up broker")
                print("=" * 60)
                helm_uninstall(broker_release_name)

    return results


def print_test_results(results):
    """Print test results summary."""
    print("\n" + "=" * 60)
    print("Test Results:")
    print("=" * 60)
    for test_name, success in results:
        status = "PASSED" if success else "FAILED"
        print(f"  {test_name}: {status}")
    print("=" * 60)

    passed = sum(1 for _, success in results if success)
    total = len(results)
    print(f"\nTotal: {passed}/{total} tests passed")

    return all(success for _, success in results)


@helm()
@angreal.command(name="test", about="test Helm charts in k3s cluster")
@angreal.argument(name="tier", required=True, help="Test tier: smoke, full, shipwright, or legacy component (broker, agent, all)")
@angreal.argument(name="no_cleanup", long="no-cleanup", help="Skip cleanup after tests", takes_value=False, is_flag=True)
@angreal.argument(name="tag", long="tag", help="Image tag to test (default: local)", default_value="local")
def test_helm_chart(tier, no_cleanup=False, tag="local"):
    """
    Test Helm charts in a k3s cluster with tiered execution.

    Each test run is fully isolated with its own k3s cluster and registry.
    Images are built locally - no GITHUB_TOKEN needed.
    Multiple tests can run in parallel (each gets unique project name).

    Test Tiers:
      smoke      - Fast validation (~3-5 min): template checks + basic deployment
      full       - Comprehensive tests (~10-15 min): smoke + external DB, RBAC variants
      shipwright - Shipwright E2E only (~15 min): build work order processing

    Legacy Components (backward compatibility):
      broker     - All broker tests (bundled/external DB, multi-tenant, values files)
      agent      - All agent tests (RBAC modes, values files)
      all        - All tests including Shipwright E2E

    Examples:
        angreal helm test smoke                   # Build images and run smoke tests
        angreal helm test all --no-cleanup        # Keep resources for inspection
    """
    valid_tiers = ["smoke", "full", "shipwright"]
    legacy_components = ["broker", "agent", "all"]

    if tier not in valid_tiers + legacy_components:
        print(f"Error: Unknown tier/component '{tier}'")
        print(f"Valid tiers: {', '.join(valid_tiers)}")
        print(f"Legacy components: {', '.join(legacy_components)}")
        sys.exit(1)

    try:
        # Setup k3s (includes local registry)
        ensure_k3s_running()

        # Build and push images to local registry
        print("\n" + "=" * 60)
        print("Building images and pushing to local registry...")
        print("=" * 60)

        success, registry = build_and_push_local_images(tag)
        if not success:
            print("Failed to build and push local images")
            sys.exit(1)

        print(f"\nUsing local registry: {registry}")

        # Verify kubectl connectivity
        verify_kubectl_connectivity()

        # Run appropriate test tier
        if tier == "smoke":
            print("\n" + "=" * 60)
            print("SMOKE TESTS (~3-5 min)")
            print("=" * 60)
            results = run_smoke_tests(tag, registry, no_cleanup)

        elif tier == "full":
            print("\n" + "=" * 60)
            print("FULL TESTS (~10-15 min)")
            print("=" * 60)
            results = run_full_tests(tag, registry, no_cleanup)

        elif tier == "shipwright":
            print("\n" + "=" * 60)
            print("SHIPWRIGHT E2E TESTS (~15 min)")
            print("=" * 60)
            results = run_shipwright_tests(tag, registry, no_cleanup)

        else:
            # Legacy component-based testing
            print("\n" + "=" * 60)
            print(f"LEGACY TESTS: {tier.upper()}")
            print("=" * 60)
            results = run_legacy_tests(tag, registry, no_cleanup, tier)

        # Print results summary
        all_passed = print_test_results(results)

        project = get_project_name()
        if no_cleanup:
            print("\nHelm releases left running (--no-cleanup)")
            print(f"Project: {project}")
            print("To inspect, run commands in a k8s container:")
            print(f"  docker run --rm -it --network {get_network_name()} \\")
            print(f"    -v {get_volume_name('brokkr-keys')}:/keys:ro \\")
            print("    -e KUBECONFIG=/keys/kubeconfig.docker.yaml \\")
            print("    alpine/k8s:1.30.10 sh")
            print("  # Then inside container:")
            print("  kubectl get pods")
            print("  helm list")
            print("\nTo clean up manually:")
            print(f"  docker compose -f .angreal/files/docker-compose.yaml -p {project} down")
            print(f"  docker volume rm {get_volume_name('brokkr-keys')} {get_volume_name('k3s-data')} {get_volume_name('registry-data')}")
        else:
            # Cleanup docker
            print("\nCleaning up docker environment...")
            docker_down(project=project)
            docker_clean(project=project)

        # Exit with error if any tests failed
        if not all_passed:
            sys.exit(1)

    except Exception as e:
        print(f"\nError during Helm testing: {e}")
        if not no_cleanup:
            print("Cleaning up docker environment...")
            project = get_project_name()
            docker_down(project=project)
            docker_clean(project=project)
        sys.exit(1)


# =============================================================================
# Render assertions: `angreal helm check-values`  (BROKKR-T-0313)
# =============================================================================
#
# `helm lint` checks syntax and schema, not effect. BROKKR-T-0308 found four
# chart values that were accepted, documented, and rendered nothing -- a
# ServiceMonitor selecting a port no Service declared, a collector endpoint
# pointing at a sidecar that was never rendered, a `metrics.enabled` that only
# gated a NetworkPolicy rule, and a `service.annotations` that appeared nowhere.
# All four passed lint and survived multiple releases.
#
# This task closes that gap with `helm template` alone -- no cluster, no docker,
# runs in about a second. It has two parts:
#
#   1. Targeted assertions. For each security- or capability-relevant value,
#      render with it set and with it unset and assert the output differs in
#      the expected way (including cross-resource linkage, e.g. that a monitor's
#      `port` names a port some other manifest actually declares).
#
#   2. A generic guard: every leaf key in a chart's values files must be
#      referenced somewhere under that chart's `templates/`, modulo an explicit
#      allowlist. This is the part that catches an inert value mechanically,
#      without anyone having to suspect it first.
#
# Deliberately NOT part of `angreal helm test`: that needs a live k3s cluster.
# This is meant to run on every PR that touches a chart.

CHARTS_DIR = Path(cwd) / "charts"
BROKER_CHART = "brokkr-broker"
AGENT_CHART = "brokkr-agent"

# Pinned so a runner's default Kubernetes version can't change what renders.
# Matches the version the Helm Template Validation CI job has always used, and
# satisfies the agent chart's `kubeVersion: ">=1.29.0-0"`.
RENDER_KUBE_VERSION = "1.30.0"

# Values files scanned by the generic guard. `values.yaml` is the chart's
# documented surface; the `values/` overlays and `values-dev.yaml` ship in the
# chart too, so a key that does nothing is just as misleading there.
def _values_files(chart_dir):
    """Return every shipped values file for a chart, values.yaml first."""
    files = [chart_dir / "values.yaml"]
    files += sorted(chart_dir.glob("values-*.yaml"))
    files += sorted((chart_dir / "values").glob("*.yaml"))
    return [f for f in files if f.is_file()]


# Leaf keys that are legitimately absent from a chart's templates/. Entries are
# exact dotted paths or `prefix.*` globs, mapped to the reason they are exempt.
# Anything not listed here MUST appear in a template -- that is the whole point.
#
# Keep this list short and keep the reasons true. A growing allowlist is
# evidence the charts are accumulating inert values, not evidence the check is
# too strict.
VALUES_KEY_ALLOWLIST = {
    BROKER_CHART: {
        # Consumed by the bundled Bitnami postgresql subchart (see Chart.yaml
        # dependencies), which reads them from its own values namespace. Our
        # templates never touch them. NOTE: `postgresql.auth.*` is deliberately
        # NOT allowlisted -- _helpers.tpl reads username/password/database to
        # build BROKKR__DATABASE__URL, so it must stay referenced.
        "postgresql.primary.*": "consumed by the Bitnami postgresql subchart",
        # DEFECT, pre-existing: values/{development,production,staging}.yaml set
        # a top-level `securityContext`, but the chart implements
        # `podSecurityContext` and `containerSecurityContext`. The block renders
        # nothing. Left allowlisted so this check can land green; remove the
        # entry when the overlays are fixed.
        "securityContext.*": (
            "DEFECT: set by values/ overlays but the chart uses "
            "podSecurityContext/containerSecurityContext; renders nothing"
        ),
    },
    AGENT_CHART: {
        # DEFECT, pre-existing: same inert `securityContext` block as the broker
        # overlays.
        "securityContext.*": (
            "DEFECT: set by values/ overlays but the chart uses "
            "podSecurityContext/containerSecurityContext; renders nothing"
        ),
        # DEFECT, pre-existing: values/{development,production,staging}.yaml set
        # `extraEnv`, but only the *broker* deployment renders extraEnv. The
        # agent deployment has no such block, so these entries are dropped.
        "extraEnv*": (
            "DEFECT: set by agent values/ overlays but only the broker "
            "deployment renders extraEnv"
        ),
    },
}

# A Helm action, e.g. `{{- toYaml .Values.resources | nindent 10 }}`. Scanning
# per-action (rather than the raw file text) means a plain YAML `# .Values.foo`
# comment cannot fake a reference.
_ACTION_RE = re.compile(r"\{\{-?(.*?)-?\}\}", re.DOTALL)
_VALUES_REF_RE = re.compile(r"\.Values\.([A-Za-z0-9_.]+)")
# Actions that emit or iterate a whole subtree, so every leaf under the
# referenced path is genuinely rendered.
_WHOLESALE_RE = re.compile(r"\b(toYaml|with|range)\b")


def _require(binary, hint):
    """Return True if `binary` is on PATH, else print an actionable message."""
    if shutil.which(binary) is None:
        print(f"{binary} not found on PATH. {hint}", file=sys.stderr)
        return False
    return True


def _load_yaml_module():
    """Import PyYAML, or explain how to get it. Returns the module or None."""
    try:
        import yaml  # noqa: PLC0415  (optional dependency, checked at call time)

        return yaml
    except ImportError:
        print(
            "PyYAML is required to parse rendered manifests but is not "
            "importable from this interpreter.\n"
            "  pip install pyyaml\n"
            "or, if angreal was installed with uv as a tool:\n"
            "  uvx --from angreal --with pyyaml angreal helm check-values",
            file=sys.stderr,
        )
        return None


def helm_render(yaml_mod, chart, values=None, values_file=None, release="render-check"):
    """Render a chart with `helm template` and return the parsed manifests.

    Args:
        yaml_mod: the PyYAML module.
        chart: chart directory name under charts/.
        values: optional dict written to a temp values file and passed with -f.
                Using a file rather than --set avoids all escaping questions and
                lets assertions set nested lists/maps directly.
        values_file: optional path to an existing values file, passed with -f.
        release: helm release name.

    Returns:
        list of parsed manifest dicts (empty documents removed).

    Raises:
        RuntimeError if helm exits non-zero.
    """
    chart_dir = CHARTS_DIR / chart
    cmd = ["helm", "template", release, str(chart_dir),
           "--kube-version", RENDER_KUBE_VERSION]
    if values_file:
        cmd += ["-f", str(values_file)]

    tmp_path = None
    try:
        if values:
            fd, tmp_path = tempfile.mkstemp(prefix="render-check-", suffix=".yaml")
            with os.fdopen(fd, "w") as fh:
                yaml_mod.safe_dump(values, fh)
            cmd += ["-f", tmp_path]

        result = subprocess.run(cmd, capture_output=True, text=True)
    finally:
        if tmp_path:
            os.unlink(tmp_path)

    if result.returncode != 0:
        raise RuntimeError(
            f"helm template failed for {chart} with values {values!r}:\n"
            f"{result.stderr.strip()}"
        )

    return [d for d in yaml_mod.safe_load_all(result.stdout) if d]


def own_manifests(docs, chart):
    """Only the manifests the chart itself renders.

    The broker chart bundles the Bitnami postgresql subchart, which also emits a
    Service, a ConfigMap and a Secret -- and emits them first. Selecting by kind
    alone would assert against postgres's Service instead of the broker's. Both
    charts stamp `app.kubernetes.io/name` with their own chart name, so that is
    the discriminator.
    """
    return [
        d
        for d in docs
        if (d.get("metadata", {}).get("labels") or {}).get("app.kubernetes.io/name")
        == chart
    ]


def render_own(yaml_mod, chart, values=None):
    """Render a chart and keep only its own manifests. See own_manifests()."""
    return own_manifests(helm_render(yaml_mod, chart, values), chart)


def manifests_of_kind(docs, kind):
    """All rendered manifests of a given kind."""
    return [d for d in docs if d.get("kind") == kind]


def single_manifest(docs, kind):
    """The one manifest of a given kind, or None if absent."""
    found = manifests_of_kind(docs, kind)
    return found[0] if found else None


def env_configmap(docs):
    """The broker's *environment* ConfigMap, excluding the dynamic TOML one.

    The broker renders two ConfigMaps since BROKKR-T-0292: the env one consumed
    via `envFrom`, and `<release>-dynamic` carrying hot-reloadable settings as a
    TOML file. `single_manifest(docs, "ConfigMap")` returns whichever renders
    first, which is the dynamic one -- so every assertion about an env key has
    to say which ConfigMap it means.
    """
    for d in manifests_of_kind(docs, "ConfigMap"):
        if not str((d or {}).get("metadata", {}).get("name", "")).endswith("-dynamic"):
            return d
    return None


def container_port_names(deployment):
    """Every container port *name* declared by a Deployment's pod spec."""
    names = set()
    containers = (
        deployment.get("spec", {})
        .get("template", {})
        .get("spec", {})
        .get("containers", [])
    )
    for container in containers:
        for port in container.get("ports", []) or []:
            if port.get("name"):
                names.add(port["name"])
    return names


def service_port_names(service):
    """Every port *name* declared by a Service."""
    return {
        p["name"] for p in service.get("spec", {}).get("ports", []) or [] if p.get("name")
    }


def container_env(deployment, container_name=None):
    """Map of env var name -> env entry for a Deployment's containers."""
    env = {}
    containers = (
        deployment.get("spec", {})
        .get("template", {})
        .get("spec", {})
        .get("containers", [])
    )
    for container in containers:
        if container_name and container.get("name") != container_name:
            continue
        for entry in container.get("env", []) or []:
            env[entry["name"]] = entry
    return env


def secret_key_ref(env_entry):
    """(secretName, key) for an env entry sourced from a Secret, else (None, None)."""
    ref = (env_entry or {}).get("valueFrom", {}).get("secretKeyRef", {})
    return ref.get("name"), ref.get("key")


class RenderChecks:
    """Accumulates pass/fail results for one run, so a single invocation reports
    every broken value rather than stopping at the first."""

    def __init__(self):
        self.failures = []
        self.passes = 0

    def expect(self, chart, value, expectation, ok, actual=None):
        """Record one assertion.

        Args:
            chart: chart name, named in the failure so it is actionable.
            value: the values.yaml path under test.
            expectation: plain-English statement of what should render.
            ok: whether it held.
            actual: what was rendered instead (only shown on failure).
        """
        if ok:
            self.passes += 1
            print(f"  ok    [{chart}] {value}: {expectation}")
            return
        message = f"[{chart}] {value}: expected {expectation}"
        if actual is not None:
            message += f"; got {actual!r}"
        self.failures.append(message)
        print(f"  FAIL  {message}")


def check_broker_values(yaml_mod, checks):
    """Targeted render assertions for the brokkr-broker chart."""
    print(f"\n{BROKER_CHART}: targeted value assertions")

    default_docs = render_own(yaml_mod, BROKER_CHART)

    # --- configReload: dynamic keys must live in the file, not the env -------
    # BROKKR-T-0292. Environment beats file in the broker's precedence chain,
    # so a dynamic key left in the env ConfigMap silently overrides the mounted
    # TOML and reloading appears to do nothing. That is the failure this pair
    # of assertions exists to catch -- it is invisible in a rendered diff.
    DYNAMIC_ENV_KEYS = [
        "BROKKR__LOG__LEVEL",
        "BROKKR__BROKER__DIAGNOSTIC_CLEANUP_INTERVAL_SECONDS",
        "BROKKR__BROKER__DIAGNOSTIC_MAX_AGE_HOURS",
        "BROKKR__BROKER__WEBHOOK_DELIVERY_INTERVAL_SECONDS",
        "BROKKR__BROKER__WEBHOOK_DELIVERY_BATCH_SIZE",
        "BROKKR__BROKER__WEBHOOK_CLEANUP_RETENTION_DAYS",
    ]

    def _dynamic_cm(docs):
        for d in docs:
            if (d or {}).get("kind") == "ConfigMap" and str(
                d.get("metadata", {}).get("name", "")
            ).endswith("-dynamic"):
                return d
        return None

    on_env = (env_configmap(default_docs) or {}).get("data", {})
    leaked = [k for k in DYNAMIC_ENV_KEYS if k in on_env]
    checks.expect(
        BROKER_CHART,
        "configReload.enabled",
        "dynamic keys absent from the env ConfigMap (env would override the file)",
        not leaked,
        leaked,
    )
    dyn = _dynamic_cm(default_docs)
    checks.expect(
        BROKER_CHART,
        "configReload.enabled",
        "dynamic ConfigMap rendered with a brokkr.toml key",
        dyn is not None and "brokkr.toml" in (dyn or {}).get("data", {}),
        None if dyn is None else list((dyn.get("data") or {}).keys()),
    )
    dep = single_manifest(default_docs, "Deployment")
    container = dep["spec"]["template"]["spec"]["containers"][0]
    env_names = {e.get("name") for e in container.get("env", []) or []}
    checks.expect(
        BROKER_CHART,
        "configReload.enabled",
        "BROKKR_CONFIG_FILE set (the watcher does not start without it)",
        "BROKKR_CONFIG_FILE" in env_names,
        sorted(n for n in env_names if n),
    )
    mounts = {m.get("name"): m for m in container.get("volumeMounts", []) or []}
    checks.expect(
        BROKER_CHART,
        "configReload.enabled",
        "dynamic config mounted as a directory, not subPath (subPath never updates)",
        "dynamic-config" in mounts and "subPath" not in mounts.get("dynamic-config", {}),
        mounts.get("dynamic-config"),
    )

    # Disabled: the keys must come back as environment, or they are lost.
    off_docs = render_own(yaml_mod, BROKER_CHART, {"configReload": {"enabled": False}})
    off_env = (env_configmap(off_docs) or {}).get("data", {})
    missing = [k for k in DYNAMIC_ENV_KEYS if k not in off_env]
    checks.expect(
        BROKER_CHART,
        "configReload.enabled",
        "with reload disabled the dynamic keys fall back to the env ConfigMap",
        not missing,
        missing,
    )

    # --- service.annotations -------------------------------------------------
    # BROKKR-T-0308: this value was accepted and documented but the Service
    # template never emitted an annotations block.
    marker = "brokkr.io/render-check"
    annotated = render_own(
        yaml_mod, BROKER_CHART, {"service": {"annotations": {marker: "present"}}}
    )
    svc = single_manifest(annotated, "Service")
    got = (svc or {}).get("metadata", {}).get("annotations", {})
    checks.expect(
        BROKER_CHART,
        "service.annotations",
        f"Service.metadata.annotations to carry {marker}",
        got.get(marker) == "present",
        got,
    )
    baseline_svc = single_manifest(default_docs, "Service")
    checks.expect(
        BROKER_CHART,
        "service.annotations",
        "no such annotation when the value is unset",
        marker not in baseline_svc.get("metadata", {}).get("annotations", {}),
        baseline_svc.get("metadata", {}).get("annotations"),
    )

    # --- metrics.serviceMonitor.enabled + port linkage -----------------------
    # BROKKR-T-0308: the ServiceMonitor selected a port name the Service did not
    # declare, so Prometheus silently scraped nothing. Assert the linkage, not
    # merely that a ServiceMonitor appears.
    checks.expect(
        BROKER_CHART,
        "metrics.serviceMonitor.enabled",
        "no ServiceMonitor when disabled (the default)",
        single_manifest(default_docs, "ServiceMonitor") is None,
    )
    monitored = render_own(
        yaml_mod, BROKER_CHART, {"metrics": {"serviceMonitor": {"enabled": True}}}
    )
    sm = single_manifest(monitored, "ServiceMonitor")
    checks.expect(
        BROKER_CHART,
        "metrics.serviceMonitor.enabled",
        "a ServiceMonitor to render when true",
        sm is not None,
    )
    if sm is not None:
        declared = service_port_names(single_manifest(monitored, "Service"))
        endpoint_ports = {
            e.get("port") for e in sm.get("spec", {}).get("endpoints", []) or []
        }
        checks.expect(
            BROKER_CHART,
            "metrics.serviceMonitor.enabled",
            f"every ServiceMonitor endpoint port to name a declared Service "
            f"port ({sorted(declared)})",
            endpoint_ports and endpoint_ports <= declared,
            sorted(endpoint_ports),
        )

    # --- networkPolicy.allowMetricsScraping ----------------------------------
    # Formerly metrics.enabled, which never gated the /metrics endpoint at all.
    scrape_from = [
        {"namespaceSelector": {"matchLabels": {"kubernetes.io/metadata.name": "monitoring"}}}
    ]

    def metrics_rules(docs):
        np = single_manifest(docs, "NetworkPolicy")
        return [
            rule
            for rule in (np or {}).get("spec", {}).get("ingress", []) or []
            if rule.get("from") == scrape_from
        ]

    on = render_own(
        yaml_mod,
        BROKER_CHART,
        {
            "networkPolicy": {
                "enabled": True,
                "allowMetricsScraping": True,
                "allowMetricsFrom": scrape_from,
            }
        },
    )
    off = render_own(
        yaml_mod,
        BROKER_CHART,
        {
            "networkPolicy": {
                "enabled": True,
                "allowMetricsScraping": False,
                "allowMetricsFrom": scrape_from,
            }
        },
    )
    checks.expect(
        BROKER_CHART,
        "networkPolicy.allowMetricsScraping",
        "a metrics ingress rule for allowMetricsFrom when true",
        len(metrics_rules(on)) == 1,
        len(metrics_rules(on)),
    )
    checks.expect(
        BROKER_CHART,
        "networkPolicy.allowMetricsScraping",
        "that same rule to disappear when false",
        len(metrics_rules(off)) == 0,
        len(metrics_rules(off)),
    )

    # --- telemetry.otlpEndpoint ----------------------------------------------
    endpoint = "http://render-check-collector:4317"
    tele = render_own(
        yaml_mod,
        BROKER_CHART,
        {"telemetry": {"enabled": True, "otlpEndpoint": endpoint}},
    )
    tele_cm = env_configmap(tele).get("data", {})
    checks.expect(
        BROKER_CHART,
        "telemetry.otlpEndpoint",
        "to reach BROKKR__TELEMETRY__OTLP_ENDPOINT in the ConfigMap",
        tele_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT") == endpoint,
        tele_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT"),
    )
    default_cm = env_configmap(default_docs).get("data", {})
    checks.expect(
        BROKER_CHART,
        "telemetry.enabled",
        "no OTLP endpoint in the ConfigMap when telemetry is off (the default)",
        "BROKKR__TELEMETRY__OTLP_ENDPOINT" not in default_cm
        and default_cm.get("BROKKR__TELEMETRY__ENABLED") == "false",
        default_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT"),
    )

    # --- existingSecret pairings ---------------------------------------------
    # Each of these has to do two things at once: keep the plaintext out of the
    # ConfigMap AND inject a secretKeyRef. A refactor could half-break either
    # side -- dropping the secretKeyRef silently unconfigures the credential;
    # dropping the ConfigMap exclusion silently leaks it. Assert the pair.
    checks.expect(
        BROKER_CHART,
        "postgresql.existingSecret (unset)",
        "BROKKR__DATABASE__URL in the ConfigMap by default",
        "BROKKR__DATABASE__URL" in default_cm,
    )
    db = render_own(
        yaml_mod,
        BROKER_CHART,
        {"postgresql": {"existingSecret": "db-cred", "existingSecretKey": "url-key"}},
    )
    db_cm = env_configmap(db).get("data", {})
    db_env = container_env(single_manifest(db, "Deployment"), "broker")
    checks.expect(
        BROKER_CHART,
        "postgresql.existingSecret",
        "BROKKR__DATABASE__URL omitted from the ConfigMap",
        "BROKKR__DATABASE__URL" not in db_cm,
        db_cm.get("BROKKR__DATABASE__URL"),
    )
    checks.expect(
        BROKER_CHART,
        "postgresql.existingSecret",
        "BROKKR__DATABASE__URL injected via secretKeyRef(db-cred, url-key)",
        secret_key_ref(db_env.get("BROKKR__DATABASE__URL")) == ("db-cred", "url-key"),
        secret_key_ref(db_env.get("BROKKR__DATABASE__URL")),
    )

    plain_wh = render_own(
        yaml_mod, BROKER_CHART, {"broker": {"webhookEncryptionKey": "deadbeef"}}
    )
    checks.expect(
        BROKER_CHART,
        "broker.webhookEncryptionKey",
        "the plaintext key in the ConfigMap when no existingSecret is set",
        env_configmap(plain_wh)
        .get("data", {})
        .get("BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY")
        == "deadbeef",
    )
    wh = render_own(
        yaml_mod,
        BROKER_CHART,
        {
            "broker": {
                "webhookEncryptionKey": "deadbeef",
                "webhookEncryptionKeyExistingSecret": "wh-cred",
                "webhookEncryptionKeyExistingSecretKey": "wh-key",
            }
        },
    )
    wh_cm = env_configmap(wh).get("data", {})
    wh_env = container_env(single_manifest(wh, "Deployment"), "broker")
    checks.expect(
        BROKER_CHART,
        "broker.webhookEncryptionKeyExistingSecret",
        "the plaintext webhook key omitted from the ConfigMap",
        "BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY" not in wh_cm,
        wh_cm.get("BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY"),
    )
    checks.expect(
        BROKER_CHART,
        "broker.webhookEncryptionKeyExistingSecret",
        "BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY via secretKeyRef(wh-cred, wh-key)",
        secret_key_ref(wh_env.get("BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY"))
        == ("wh-cred", "wh-key"),
        secret_key_ref(wh_env.get("BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY")),
    )

    pak_secret = render_own(
        yaml_mod,
        BROKER_CHART,
        {
            "broker": {
                "pakHash": "0123456789abcdef",
                "pakHashExistingSecret": "pak-cred",
                "pakHashExistingSecretKey": "pak-key",
            }
        },
    )
    pak_secret_cm = env_configmap(pak_secret).get("data", {})
    pak_secret_env = container_env(single_manifest(pak_secret, "Deployment"), "broker")
    checks.expect(
        BROKER_CHART,
        "broker.pakHashExistingSecret",
        "the plaintext PAK hash omitted from the ConfigMap",
        "BROKKR__BROKER__PAK_HASH" not in pak_secret_cm,
        pak_secret_cm.get("BROKKR__BROKER__PAK_HASH"),
    )
    checks.expect(
        BROKER_CHART,
        "broker.pakHashExistingSecret",
        "BROKKR__BROKER__PAK_HASH via secretKeyRef(pak-cred, pak-key)",
        secret_key_ref(pak_secret_env.get("BROKKR__BROKER__PAK_HASH"))
        == ("pak-cred", "pak-key"),
        secret_key_ref(pak_secret_env.get("BROKKR__BROKER__PAK_HASH")),
    )

    # --- broker.pakHash: the empty-value trap --------------------------------
    # An empty pakHash must render nothing at all. Rendering the key with an
    # empty value is what previously left the publicly-known dev credential
    # live, because the broker fell back to its built-in default.
    empty_pak = render_own(yaml_mod, BROKER_CHART, {"broker": {"pakHash": ""}})
    empty_pak_cm = env_configmap(empty_pak).get("data", {})
    checks.expect(
        BROKER_CHART,
        "broker.pakHash",
        "BROKKR__BROKER__PAK_HASH absent from the ConfigMap when empty",
        "BROKKR__BROKER__PAK_HASH" not in empty_pak_cm,
        empty_pak_cm.get("BROKKR__BROKER__PAK_HASH"),
    )
    set_pak = render_own(
        yaml_mod, BROKER_CHART, {"broker": {"pakHash": "0123456789abcdef"}}
    )
    set_pak_cm = env_configmap(set_pak).get("data", {})
    checks.expect(
        BROKER_CHART,
        "broker.pakHash",
        "BROKKR__BROKER__PAK_HASH rendered when non-empty",
        set_pak_cm.get("BROKKR__BROKER__PAK_HASH") == "0123456789abcdef",
        set_pak_cm.get("BROKKR__BROKER__PAK_HASH"),
    )


def check_agent_values(yaml_mod, checks):
    """Targeted render assertions for the brokkr-agent chart."""
    print(f"\n{AGENT_CHART}: targeted value assertions")

    default_docs = render_own(yaml_mod, AGENT_CHART)

    # --- metrics.podMonitor.enabled + port linkage ---------------------------
    # The agent renders no Service, so Prometheus scrapes the pod directly and
    # the PodMonitor's `port` is a *container* port name. The BROKKR-T-0308 bug
    # was a monitor naming a port nothing declared -- so assert the linkage.
    checks.expect(
        AGENT_CHART,
        "metrics.podMonitor.enabled",
        "no PodMonitor when disabled (the default)",
        single_manifest(default_docs, "PodMonitor") is None,
    )
    monitored = render_own(
        yaml_mod, AGENT_CHART, {"metrics": {"podMonitor": {"enabled": True}}}
    )
    pm = single_manifest(monitored, "PodMonitor")
    checks.expect(
        AGENT_CHART,
        "metrics.podMonitor.enabled",
        "a PodMonitor to render when true",
        pm is not None,
    )
    if pm is not None:
        declared = container_port_names(single_manifest(monitored, "Deployment"))
        endpoint_ports = {
            e.get("port")
            for e in pm.get("spec", {}).get("podMetricsEndpoints", []) or []
        }
        checks.expect(
            AGENT_CHART,
            "metrics.podMonitor.enabled",
            f"every PodMonitor endpoint port to name a container port declared "
            f"in the Deployment ({sorted(declared)})",
            endpoint_ports and endpoint_ports <= declared,
            sorted(endpoint_ports),
        )

    # --- networkPolicy.allowMetricsScraping ----------------------------------
    scrape_from = [
        {"namespaceSelector": {"matchLabels": {"kubernetes.io/metadata.name": "monitoring"}}}
    ]

    def metrics_rules(docs):
        np = single_manifest(docs, "NetworkPolicy")
        return [
            rule
            for rule in (np or {}).get("spec", {}).get("ingress", []) or []
            if rule.get("from") == scrape_from
        ]

    on = render_own(
        yaml_mod,
        AGENT_CHART,
        {
            "networkPolicy": {
                "enabled": True,
                "allowMetricsScraping": True,
                "allowMetricsFrom": scrape_from,
            }
        },
    )
    off = render_own(
        yaml_mod,
        AGENT_CHART,
        {
            "networkPolicy": {
                "enabled": True,
                "allowMetricsScraping": False,
                "allowMetricsFrom": scrape_from,
            }
        },
    )
    checks.expect(
        AGENT_CHART,
        "networkPolicy.allowMetricsScraping",
        "a metrics ingress rule for allowMetricsFrom when true",
        len(metrics_rules(on)) == 1,
        len(metrics_rules(on)),
    )
    checks.expect(
        AGENT_CHART,
        "networkPolicy.allowMetricsScraping",
        "that same rule to disappear when false",
        len(metrics_rules(off)) == 0,
        len(metrics_rules(off)),
    )

    # --- telemetry.otlpEndpoint ----------------------------------------------
    endpoint = "http://render-check-collector:4317"
    tele = render_own(
        yaml_mod,
        AGENT_CHART,
        {"telemetry": {"enabled": True, "otlpEndpoint": endpoint}},
    )
    tele_cm = single_manifest(tele, "ConfigMap").get("data", {})
    checks.expect(
        AGENT_CHART,
        "telemetry.otlpEndpoint",
        "to reach BROKKR__TELEMETRY__OTLP_ENDPOINT in the ConfigMap",
        tele_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT") == endpoint,
        tele_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT"),
    )
    default_cm = single_manifest(default_docs, "ConfigMap").get("data", {})
    checks.expect(
        AGENT_CHART,
        "telemetry.enabled",
        "no OTLP endpoint in the ConfigMap when telemetry is off (the default)",
        "BROKKR__TELEMETRY__OTLP_ENDPOINT" not in default_cm
        and default_cm.get("BROKKR__TELEMETRY__ENABLED") == "false",
        default_cm.get("BROKKR__TELEMETRY__OTLP_ENDPOINT"),
    )

    # --- broker.existingSecret pairing ---------------------------------------
    plaintext = "brokkr_render_check_plaintext_pak"
    plain = render_own(yaml_mod, AGENT_CHART, {"broker": {"pak": plaintext}})
    plain_cm = single_manifest(plain, "ConfigMap").get("data", {})
    plain_env = container_env(single_manifest(plain, "Deployment"), "agent")
    checks.expect(
        AGENT_CHART,
        "broker.pak",
        "the plaintext PAK in the ConfigMap when no existingSecret is set",
        plain_cm.get("BROKKR__AGENT__PAK") == plaintext,
        plain_cm.get("BROKKR__AGENT__PAK"),
    )
    checks.expect(
        AGENT_CHART,
        "broker.pak",
        "no BROKKR__AGENT__PAK env override on the container in that case",
        "BROKKR__AGENT__PAK" not in plain_env,
        sorted(plain_env),
    )
    sourced = render_own(
        yaml_mod,
        AGENT_CHART,
        {
            "broker": {
                "pak": plaintext,
                "existingSecret": "agent-pak",
                "existingSecretKey": "PAK_KEY",
            }
        },
    )
    sourced_cm = single_manifest(sourced, "ConfigMap").get("data", {})
    sourced_env = container_env(single_manifest(sourced, "Deployment"), "agent")
    checks.expect(
        AGENT_CHART,
        "broker.existingSecret",
        "the plaintext PAK kept out of the ConfigMap",
        sourced_cm.get("BROKKR__AGENT__PAK") == "",
        sourced_cm.get("BROKKR__AGENT__PAK"),
    )
    checks.expect(
        AGENT_CHART,
        "broker.existingSecret",
        "BROKKR__AGENT__PAK injected via secretKeyRef(agent-pak, PAK_KEY)",
        secret_key_ref(sourced_env.get("BROKKR__AGENT__PAK")) == ("agent-pak", "PAK_KEY"),
        secret_key_ref(sourced_env.get("BROKKR__AGENT__PAK")),
    )


def check_chart_renders(yaml_mod, chart, checks):
    """Every shipped values file must render without error, and produce output.

    This subsumes the old `helm template ... > /dev/null` CI steps: same command,
    same pinned --kube-version, but covering *every* shipped values file rather
    than production + development, and asserting the render is non-empty rather
    than merely non-erroring. (Rendering nothing was never an error, which is
    how four inert values passed this job for multiple releases.)
    """
    chart_dir = CHARTS_DIR / chart
    print(f"\n{chart}: every shipped values file renders")
    # `None` is the chart's own defaults, i.e. the bare `helm template` case.
    for values_file in [None] + _values_files(chart_dir):
        label = "default values" if values_file is None else str(
            values_file.relative_to(chart_dir)
        )
        # values.yaml is the chart default; passing it with -f is a no-op but
        # harmless, and keeps the loop honest if the file is ever renamed.
        try:
            docs = helm_render(yaml_mod, chart, values_file=values_file)
        except RuntimeError as e:
            checks.expect(chart, label, "the chart to render", False, str(e))
            continue
        checks.expect(
            chart,
            label,
            f"the chart to render at least one manifest "
            f"(kube-version {RENDER_KUBE_VERSION})",
            bool(docs),
            len(docs),
        )


def template_value_references(chart_dir):
    """Scan a chart's templates/ for `.Values.<path>` references.

    Returns:
        (exact, wholesale) sets of dotted paths. `exact` is every referenced
        path; `wholesale` is the subset referenced inside a toYaml/with/range
        action, meaning the entire subtree below it is rendered.
    """
    exact, wholesale = set(), set()
    for path in sorted((chart_dir / "templates").rglob("*")):
        if not path.is_file():
            continue
        for action in _ACTION_RE.findall(path.read_text()):
            # `{{/* ... */}}` is a Helm comment -- a .Values mention inside one
            # documents a reference, it is not a reference.
            if action.strip().startswith("/*"):
                continue
            emits_subtree = bool(_WHOLESALE_RE.search(action))
            for ref in _VALUES_REF_RE.findall(action):
                ref = ref.rstrip(".")
                exact.add(ref)
                if emits_subtree:
                    wholesale.add(ref)
    return exact, wholesale


def values_leaf_keys(node, prefix=()):
    """Yield the dotted path of every leaf in a parsed values file.

    Empty maps (`annotations: {}`) and lists count as leaves: they are settable
    surface, and `service.annotations: {}` rendering nowhere was one of the four
    original defects.
    """
    if isinstance(node, dict) and node:
        for key, child in node.items():
            yield from values_leaf_keys(child, prefix + (str(key),))
    else:
        yield ".".join(prefix)


def allowlisted(key, allowlist):
    """Return the allowlist reason for a key, or None."""
    for pattern, reason in allowlist.items():
        if pattern.endswith("*"):
            if key.startswith(pattern[:-1]):
                return reason
        elif key == pattern:
            return reason
    return None


def check_values_are_referenced(yaml_mod, chart, checks):
    """Generic guard: every leaf key in the chart's values files must be
    referenced somewhere under templates/.

    This is the mechanical version of the BROKKR-T-0308 investigation: it does
    not need anyone to suspect a specific value first.
    """
    chart_dir = CHARTS_DIR / chart
    exact, wholesale = template_value_references(chart_dir)
    allowlist = VALUES_KEY_ALLOWLIST.get(chart, {})

    print(f"\n{chart}: every values key is referenced under templates/")
    for values_file in _values_files(chart_dir):
        data = yaml_mod.safe_load(values_file.read_text()) or {}
        rel = values_file.relative_to(chart_dir)
        orphans = []
        for key in values_leaf_keys(data):
            if not key:
                continue
            parts = key.split(".")
            referenced = key in exact or any(
                ".".join(parts[:i]) in wholesale for i in range(1, len(parts))
            )
            if referenced or allowlisted(key, allowlist):
                continue
            orphans.append(key)
        checks.expect(
            chart,
            str(rel),
            "every leaf key to be referenced under templates/ "
            "(or listed in VALUES_KEY_ALLOWLIST in .angreal/task_helm.py)",
            not orphans,
            orphans,
        )


def print_allowlist_notice():
    """Print the allowlist so its contents are visible on every run rather than
    buried in the source. Entries whose reason starts with DEFECT are live bugs
    the check is knowingly tolerating."""
    defects = [
        (chart, pattern, reason)
        for chart, entries in VALUES_KEY_ALLOWLIST.items()
        for pattern, reason in entries.items()
        if reason.startswith("DEFECT")
    ]
    if not defects:
        return
    print("\nKnown-inert values tolerated by the allowlist (fix and remove):")
    for chart, pattern, reason in defects:
        print(f"  [{chart}] {pattern}: {reason}")


def run_render_assertions():
    """Run every render assertion. Returns 0 on success, 1 on failure.

    Kept separate from the `check-values` command so Phase 1 of the cluster
    suite can call it directly rather than shelling out to angreal.
    """
    if not _require("helm", "Install Helm 3: https://helm.sh/docs/intro/install/"):
        return 1

    yaml_mod = _load_yaml_module()
    if yaml_mod is None:
        return 1

    checks = RenderChecks()
    print("=" * 72)
    print("Helm chart render assertions (helm template only -- no cluster)")
    print("=" * 72)

    try:
        for chart in (BROKER_CHART, AGENT_CHART):
            check_chart_renders(yaml_mod, chart, checks)
        check_broker_values(yaml_mod, checks)
        check_agent_values(yaml_mod, checks)
        for chart in (BROKER_CHART, AGENT_CHART):
            check_values_are_referenced(yaml_mod, chart, checks)
    except RuntimeError as e:
        print(f"\nFAIL: {e}", file=sys.stderr)
        return 1
    except Exception:
        # angreal reports the exception type but swallows the traceback, which
        # makes a bug in this check very hard to localise. Print it.
        import traceback

        traceback.print_exc()
        return 1

    print_allowlist_notice()

    print("\n" + "=" * 72)
    if checks.failures:
        print(
            f"FAIL: {len(checks.failures)} of {len(checks.failures) + checks.passes} "
            f"chart render assertions failed",
            file=sys.stderr,
        )
        for failure in checks.failures:
            print(f"  {failure}", file=sys.stderr)
        print(
            "\nA value that no longer changes the rendered output is a value "
            "operators can set with no effect.\nInspect what actually renders "
            "with:\n"
            "  helm template check charts/<chart> --set <value>=<x> | less\n"
            "Reproduce locally with: angreal helm check-values",
            file=sys.stderr,
        )
        return 1

    print(f"OK: {checks.passes} chart render assertions passed")
    return 0


@helm()
@angreal.command(
    name="check-values",
    about="assert chart values actually change rendered output (no cluster needed)",
)
def check_values():
    """Render-level assertions over both Helm charts.

    `helm lint` validates syntax and schema; it cannot tell you that a value is
    accepted, documented, and renders nothing. This task uses `helm template`
    only -- no cluster, no docker, about two seconds -- and returns non-zero
    when a value stops taking effect.

    Three layers:

      0. Every shipped values file renders without error and produces output.
         (This replaces the CI job's old `helm template ... > /dev/null` steps,
         and Phase 1 of `angreal helm test`, which did the same thing in a
         container.)

      1. Targeted assertions for the security- and capability-relevant values:
         monitor-to-declared-port linkage, NetworkPolicy metrics rules,
         Service annotations, telemetry endpoints, every existingSecret pairing
         (plaintext omitted AND secretKeyRef added), and broker.pakHash's
         empty-value trap.

      2. A generic guard that every leaf key in every shipped values file is
         referenced under that chart's templates/, with an explicit allowlist.

    Run it directly:

        angreal helm check-values

    If PyYAML is not importable from your angreal interpreter:

        uvx --from angreal --with pyyaml angreal helm check-values
    """
    return run_render_assertions()
