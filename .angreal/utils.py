import angreal # type: ignore
import os
import subprocess



cwd = os.path.join(angreal.get_root(),'..')
DOCKER_COMPOSE_FILE = os.path.join(angreal.get_root(),'files','docker-compose.yaml')


def docker_up(services=None):
    """Start docker compose services.

    Args:
        services: Optional list of specific services to start. If None, starts all services.
    """
    os.makedirs('/tmp/brokkr-keys', exist_ok=True)

    services_str = " ".join(services) if services else ""
    subprocess.run(
        f"docker compose -f {DOCKER_COMPOSE_FILE} up --build -d --wait {services_str}",
        cwd=cwd,
        shell=True
    )

def docker_down():
    subprocess.run([f"docker compose -f {DOCKER_COMPOSE_FILE} down" ]
                    , cwd=cwd, shell=True)


def docker_clean():
    subprocess.run(["docker volume rm brokkr-dev_brokkr-postgres-data brokkr-dev_k3s-data brokkr-dev_brokkr-keys"]
                    , cwd=cwd, shell=True)
