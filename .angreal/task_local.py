import angreal # type: ignore
from utils import docker_up,docker_down, docker_clean, cwd, DOCKER_COMPOSE_FILE
import subprocess


local = angreal.command_group(name="local", about="commands for"
                                 " local development environment")






@local()
@angreal.command(name="up", about="bring up backing services")
def up():
    docker_up()


@local()
@angreal.command(name="down", about="bring down backing services")
@angreal.argument(name="hard", long="hard", help="hard down removal of volumes", takes_value=False, is_flag=True)
def down(hard=None):
    docker_down()
    if hard:
        docker_clean()


@local()
@angreal.command(name="reset", about="reset backing services")
@angreal.argument(name="hard", long="hard", help="hard down removal of volumes", takes_value=False, is_flag=True)
def reset(hard=None):
    docker_down()
    if hard:
        docker_clean()
    docker_up()


@local()
@angreal.command(name="clean", about="reset backing services")
def clean():
    docker_down()
    docker_clean()


@local()
@angreal.command(name="docs", about="cargo docs")
def docs():

    subprocess.run(
        [
            "cargo doc --open --no-deps --document-private-items"
        ], cwd=cwd, shell=True
    )


@local()
@angreal.command(name="rebuild", about="rebuild a specific service")
@angreal.argument(name="service", help="service to rebuild (broker, agent, ui)", required=True)
def rebuild(service):
    services = {
        "broker": "brokkr-broker",
        "agent": "brokkr-agent",
        "ui": "brokkr-ui"
    }

    if service not in services:
        print(f"Error: Unknown service '{service}'. Available services: {', '.join(services.keys())}")
        return

    docker_service = services[service]
    subprocess.run(
        [f"docker compose -f {DOCKER_COMPOSE_FILE} up -d --build {docker_service}"],
        cwd=cwd,
        shell=True
    )
