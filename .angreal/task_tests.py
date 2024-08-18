import angreal # type: ignore
import subprocess
import os
from utils import docker_up,docker_down,cwd, docker_clean

test = angreal.command_group(name="tests", about="commands for testing the"
                             " application and library")



@test()
@angreal.command(name="unit", about="run unit tests")
def unit_tests():
    """
    """
    subprocess.run(
        [
            "cargo test -v --lib -- --test-threads=1",
        ], cwd=cwd, shell=True
    )

@test()
@angreal.command(name="functional", about="run our functional "
                 "tests (crates/*/tests/functional.rs)")
def functional_tests():
    """
    """
    docker_up()
    subprocess.run(
        [
            "cargo test --test functional ",
        ], cwd=cwd, shell=True
    )
    docker_down()

@test()
@angreal.command(name="integration", about="run our integration "
                 "tests (crates/*/tests/integration.rs)")
def integration_tests():
    """
    """
    docker_up()
    subprocess.run(
        [
            "cargo test --test integration",
        ], cwd=cwd, shell=True
    )
    docker_down()


@test()
@angreal.command(name="migrations", about="run all migrations + redo to ensure"
                 " up and down work as intended. ")
def migration_tests():
    """
    """
    brokkr_models_dir = os.path.join(
        angreal.get_root(),
        '..',
        "crates",
        "brokkr-models"
        )
    docker_down()
    docker_clean()
    docker_up()

    subprocess.run(
        [
            "diesel migration run && diesel migration redo -a"
        ], cwd=brokkr_models_dir, shell=True
    )

    docker_down()
    docker_clean()
