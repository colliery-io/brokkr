import angreal # type: ignore
import subprocess
from utils import docker_up,docker_down,cwd, docker_clean
import time
test = angreal.command_group(name="tests", about="commands for test suites")




def get_crates():
    """Get all crates in the workspace."""
    return {
        "integration_tests": ["brokkr-agent", "brokkr-broker"],
        "unit_tests": ["brokkr-agent", "brokkr-broker", "brokkr-models", "brokkr-utils"]
    }

def run_unit_tests(crate_name: str = "", test_filter: str = ""):
    """Run unit tests for a specific crate or all crates."""
    cmd = ["cargo", "test", "--lib", "-v"]
    if crate_name:
        cmd.extend(["-p", crate_name])
    cmd.extend(["--", "--test-threads=1"])
    if test_filter:
        cmd.extend(test_filter.split())
    result = subprocess.run(cmd, cwd=cwd)
    return result.returncode

def run_integration_tests(crate_name: str = "", test_filter: str = ""):
    """Run integration tests for a specific crate or all crates."""
    cmd = ["cargo", "test", "--test", "integration"]
    if crate_name:
        cmd.extend(["-p", crate_name])
    if test_filter:
        cmd.extend(["--", test_filter, "--test-threads=1", "--nocapture"])
    else:
        cmd.extend(["--", "--test-threads=1", "--nocapture"])
    result = subprocess.run(cmd, cwd=cwd)
    return result.returncode


CRATES = get_crates()


@test()
@angreal.command(name="unit", about="run unit tests for a specific crate")
@angreal.argument(name="test_filter", required=False, help="Filter for specific tests or modules")
@angreal.argument(name="crate_name", required=True, help= f"Name of the crate to test ({CRATES['unit_tests'] + ['all']})")
def unit_tests(crate_name: str, test_filter: str = ""):
    """Run unit tests for a specific crate."""
    return_codes = []
    rc = None
    if crate_name == "all":
        for crate in CRATES["unit_tests"]:
            return_code = run_unit_tests(crate, test_filter)
            return_codes.append((crate,return_code))
        if any(code != 0 for _, code in return_codes):
            rc =   max(code for _, code in return_codes)
            print(f"Unit tests failed for {crate} with return code {rc}")
    else:
        rc = run_unit_tests(crate_name, test_filter)

    return rc


@test()
@angreal.command(name="integration", about="run integration tests for a specific crate")
@angreal.argument(name="skip_docker", long="skip-docker", required=False, help="Skip docker compose up", takes_value=False, is_flag=True)
@angreal.argument(name="test_filter", required=False, help="Filter for specific tests or modules")
@angreal.argument(name="crate_name", required=True, help= f"Name of the crate to test ({CRATES['integration_tests'] + ['all']})")
def integration_tests(crate_name: str, test_filter: str = "", skip_docker: bool = False):
    """Run integration tests for a specific crate."""
    if not skip_docker:
        docker_clean()
        docker_up()

    print("Sleeping for 30 seconds, waiting for services to stabilize - get some coffee")
    time.sleep(30)

    rc = None
    return_codes = []
    try:
        if crate_name == "all":
            for crate in CRATES["integration_tests"]:
                return_code = run_integration_tests(crate, test_filter)
                return_codes.append((crate,return_code))
            if any(code != 0 for _, code in return_codes):
                rc =   max(code for _, code in return_codes)
                print(f"Integration tests failed for {crate} with return code {rc}")
        else:
            rc = run_integration_tests(crate_name, test_filter)
        if not skip_docker:
            input("Press Enter to shutdown containers and clean up...")
    finally:
        if not skip_docker:
            docker_down()
            docker_clean()
        return rc
