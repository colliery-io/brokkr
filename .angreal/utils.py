import angreal # type: ignore
import os
import subprocess


cwd = os.path.join(angreal.get_root(),'..')
DOCKER_COMPOSE_FILE = os.path.join(angreal.get_root(),'files','docker-compose.yaml')

# Default project name (matches docker-compose.yaml name field)
DEFAULT_PROJECT = "brokkr-dev"


def docker_up(services=None, project=DEFAULT_PROJECT):
    """Start docker compose services.

    Args:
        services: Optional list of specific services to start. If None, starts all services.
        project: Docker compose project name for isolation.
    """
    os.makedirs('/tmp/brokkr-keys', exist_ok=True)

    services_str = " ".join(services) if services else ""
    subprocess.run(
        f"docker compose -f {DOCKER_COMPOSE_FILE} -p {project} up --build -d --wait {services_str}",
        cwd=cwd,
        shell=True
    )


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
