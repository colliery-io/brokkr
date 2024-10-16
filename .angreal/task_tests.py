import angreal # type: ignore
import subprocess
import os
from utils import docker_up,docker_down,cwd, docker_clean
import time
test = angreal.command_group(name="tests", about="commands for testing the"
                             " application and library")



@test()
@angreal.command(name="unit", about="run unit tests")
@angreal.argument(name="test_filter", required=False, help="Filter for specific tests or modules")
def unit_tests(test_filter: str = ""):
    """
    Run unit tests with an optional filter.
    """
    cmd = ["cargo", "test", "-v", "--lib", "--", "--test-threads=1"]
    if test_filter:
        cmd.extend(test_filter.split())
    subprocess.run(cmd, cwd=cwd)


@test()
@angreal.command(name="integration", about="run our integration tests (crates/*/tests/integration.rs)")
@angreal.argument(name="test_filter", required=False, help="Filter for specific tests or modules")
def integration_tests(test_filter: str = ""):
    """
    Run integration tests with an optional filter.
    """
    docker_down()
    docker_clean()
    docker_up()

    time.sleep(5)


    cmd = ["cargo", "test", "--test", "integration"]
    if test_filter:
        cmd.extend(["--", test_filter, "--test-threads=1", "--nocapture"])
    else:
        cmd.extend(["--", "--test-threads=1", "--nocapture"])

    subprocess.run(cmd, cwd=cwd)


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
