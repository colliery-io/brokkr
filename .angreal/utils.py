import angreal # type: ignore
import os
import subprocess



cwd = os.path.join(angreal.get_root(),'..')
DOCKER_COMPOSE_FILE = os.path.join(angreal.get_root(),'files','docker-compose.yaml')


def docker_up():
    subprocess.run(f"docker compose -f {DOCKER_COMPOSE_FILE} up -d --wait"
                    , cwd=cwd, shell=True)

def docker_down():
    subprocess.run([f"docker compose -f {DOCKER_COMPOSE_FILE} down" ]
                    , cwd=cwd, shell=True)


def docker_clean():
    subprocess.run(["docker volume rm brokkr-dev_brokkr-postgres-data"]
                    , cwd=cwd, shell=True)
