import angreal # type: ignore
import os
import subprocess



cwd = os.path.join(angreal.get_root(),'..')
DOCKER_COMPOSE_FILE = os.path.join(angreal.get_root(),'files','docker-compose.yaml')


def docker_up():
    os.makedirs('/tmp/brokkr-keys', exist_ok=True)
    subprocess.run(f"docker compose  -f {DOCKER_COMPOSE_FILE} up --build -d --wait"
                    , cwd=cwd, shell=True)

def docker_down():
    subprocess.run([f"docker compose -f {DOCKER_COMPOSE_FILE} down" ]
                    , cwd=cwd, shell=True)


def docker_clean():
    subprocess.run(["docker volume rm brokkr-dev_brokkr-postgres-data brokkr-dev_k3s-data brokkr-dev_brokkr-keys"]
                    , cwd=cwd, shell=True)
