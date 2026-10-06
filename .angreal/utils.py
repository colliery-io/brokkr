import angreal # type: ignore
import json
import os
import subprocess
import sys


cwd = os.path.join(angreal.get_root(),'..')
DOCKER_COMPOSE_FILE = os.path.join(angreal.get_root(),'files','docker-compose.yaml')

# Default project name (matches docker-compose.yaml name field)
DEFAULT_PROJECT = "brokkr-dev"


# The dev broker runs the embedded default admin hash (crates/brokkr-utils/
# default.toml), so this publicly known PAK opens it. Dev only.
DEV_ADMIN_PAK = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"

# The webhook catcher's host port. 8090 is a common port; set
# BROKKR_DEV_WEBHOOK_PORT when another service holds it.
WEBHOOK_PORT = os.environ.get("BROKKR_DEV_WEBHOOK_PORT", "8090")


def _say(line=""):
    """Print and flush: the task runs in an embedded interpreter, and an
    unflushed line is lost when the task exits."""
    print(line, flush=True)


def print_ready_banner():
    """What a person needs after `angreal local up`: the URLs, the admin
    credential, and where the dev agent's key is (BROKKR-T-0330). The compose
    `ready` service prints its own banner, but `up -d` runs detached, so nobody
    sees it."""
    _say("")
    _say("Brokkr dev environment is up.")
    _say("")
    _say("  Broker API:         http://localhost:3000/api/v1")
    _say("  Operator console:   http://localhost:3000/")
    _say("  Demo UI (ui-slim):  http://localhost:3001")
    _say(f"  Webhook catcher:    http://localhost:{WEBHOOK_PORT}")
    _say("  PostgreSQL:         localhost:5433")
    _say("")
    _say("  Admin PAK (dev only, publicly known):")
    _say(f"    export ADMIN_PAK={DEV_ADMIN_PAK}")
    _say("")
    _say("  Dev agent:   brokkr-integration-test-agent, cluster brokkr-dev-integration-cluster.")
    _say("               It starts INACTIVE: activate it before it applies anything.")
    _say("  Agent PAK:   /tmp/brokkr-keys/agent.pak")
    _say("  Kubeconfig:  /tmp/brokkr-keys/kubeconfig.local.yaml")
    _say("")
    _say("  Next: docs/src/getting-started/evaluate.md, or `angreal docs serve`.")
    _say("")


def _compose_rows(project=DEFAULT_PROJECT, service=""):
    """The rows of `docker compose ps --all --format json`, for one service or
    for the project."""
    out = subprocess.run(
        f"docker compose -f {DOCKER_COMPOSE_FILE} -p {project} ps --all --format json {service}",
        cwd=cwd,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    rows = []
    for chunk in ([out] if out.startswith("[") else out.splitlines()):
        try:
            parsed = json.loads(chunk)
            rows.extend(parsed if isinstance(parsed, list) else [parsed])
        except json.JSONDecodeError:
            pass
    return rows


def _ready_state(project=DEFAULT_PROJECT):
    """`(state, exit_code)` of the compose `ready` service, or `(None, None)`."""
    for r in _compose_rows(project, "ready"):
        return r.get("State"), r.get("ExitCode")
    return None, None


def wait_for_stack(project=DEFAULT_PROJECT, timeout_s=300):
    """Wait until every container of the project is running (and healthy,
    where it has a health check) or has exited 0. False when one exited with
    another code, is unhealthy, or the time is up. For a subset of services,
    where there is no `ready` service to ask (BROKKR-T-0330)."""
    import time
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        rows = _compose_rows(project)
        settled = bool(rows)
        for r in rows:
            state, health, code = r.get("State"), r.get("Health", ""), r.get("ExitCode", 0)
            if state == "exited" and code != 0:
                return False
            if health == "unhealthy":
                return False
            if state == "exited" and code == 0:
                continue
            if state == "running" and health in ("", "healthy"):
                continue
            settled = False
        if settled:
            return True
        time.sleep(5)
    return False


def wait_for_ready(project=DEFAULT_PROJECT, timeout_s=900):
    """Wait until the compose `ready` service has run to the end, and say
    whether it exited 0.

    `docker compose up --wait` returns as soon as any one-shot container
    exits, even with code 0 and even before `ready` has started, so its exit
    code cannot say whether the stack is up. The `ready` service runs last,
    after every health check, and its exit code can (BROKKR-T-0330)."""
    import time
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        state, code = _ready_state(project)
        if state == "exited":
            return code == 0
        time.sleep(5)
    return False


def docker_up(services=None, project=DEFAULT_PROJECT):
    """Start docker compose services.

    Args:
        services: Optional list of specific services to start. If None, starts all services.
        project: Docker compose project name for isolation.

    Exits with the compose exit code when compose fails, so a broken stack is
    not reported as success (BROKKR-T-0330).
    """
    os.makedirs('/tmp/brokkr-keys', exist_ok=True)

    services_str = " ".join(services) if services else ""
    result = subprocess.run(
        f"docker compose -f {DOCKER_COMPOSE_FILE} -p {project} up --build -d --wait {services_str}",
        cwd=cwd,
        shell=True
    )
    # `--wait` returns 1 as soon as a one-shot service exits, so the verdict
    # is the `ready` service's exit code, once it has run.
    if not services:
        _say("Waiting for the ready service (health checks, Tekton, Shipwright)...")
        ok = wait_for_ready(project)
    else:
        _say("Waiting for the services to be healthy...")
        ok = wait_for_stack(project)
    if not ok:
        _say("")
        _say(f"docker compose up failed (exit {result.returncode}). The services and their states:")
        _say("")
        subprocess.run(
            f"docker compose -f {DOCKER_COMPOSE_FILE} -p {project} ps --all --format 'table {{{{.Service}}}}\t{{{{.Status}}}}'",
            cwd=cwd,
            shell=True
        )
        _say("")
        _say(f"Logs of one service: docker compose -f {DOCKER_COMPOSE_FILE} -p {project} logs --tail 50 <service>")
        _say("A port that is already allocated: stop the other process, or set BROKKR_DEV_WEBHOOK_PORT for the webhook catcher.")
        sys.exit(result.returncode or 1)
    if not services:
        print_ready_banner()


def docker_down(project=DEFAULT_PROJECT):
    """Stop and remove docker compose services."""
    subprocess.run(
        f"docker compose -f {DOCKER_COMPOSE_FILE} -p {project} down",
        cwd=cwd,
        shell=True
    )


def docker_clean(project=DEFAULT_PROJECT):
    """Remove docker volumes for the project."""
    volumes = [
        f"{project}_brokkr-postgres-data",
        f"{project}_k3s-data",
        f"{project}_brokkr-keys",
        f"{project}_registry-data",
    ]
    subprocess.run(
        f"docker volume rm {' '.join(volumes)} 2>/dev/null || true",
        cwd=cwd,
        shell=True
    )
