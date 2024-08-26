import angreal # type: ignore
from utils import docker_up,docker_down, docker_clean, cwd
import subprocess


local = angreal.command_group(name="local", about="dev commands for"
                                 " local development")






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
