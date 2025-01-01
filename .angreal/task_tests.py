import angreal # type: ignore
import subprocess
import os
from utils import docker_up,docker_down,cwd, docker_clean
import time
import glob
test = angreal.command_group(name="tests", about="commands for test suites")




def get_crates():
    """Get all crates in the workspace."""
    crates_path = os.path.join(cwd, "crates", "*")
    return [os.path.basename(p) for p in glob.glob(crates_path) if os.path.isdir(p)]

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
@angreal.argument(name="crate_name", required=True, help= f"Name of the crate to test ({CRATES + ['all']})")
def unit_tests(crate_name: str, test_filter: str = ""):
    """Run unit tests for a specific crate."""
    return_codes = []
    if crate_name == "all":
        for crate in CRATES:
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
@angreal.argument(name="test_filter", required=False, help="Filter for specific tests or modules")
@angreal.argument(name="crate_name", required=True, help= f"Name of the crate to test ({CRATES + ['all']})")
@angreal.option(name="skip_docker", required=False, help="Skip docker compose up")
def integration_tests(crate_name: str, test_filter: str = "", skip_docker: bool = False):
    """Run integration tests for a specific crate."""
    if not skip_docker:
        docker_clean()
        docker_up()
    print("Waiting for applications to come up and be stable, this may take a while...grab a coffee!")
    time.sleep(180)

    return_codes = []
    try:
        if crate_name == "all":
            for crate in CRATES:
                return_code = run_integration_tests(crate, test_filter)
                return_codes.append((crate,return_code))
            if any(code != 0 for _, code in return_codes):
                rc =   max(code for _, code in return_codes)
                print(f"Integration tests failed for {crate} with return code {rc}")
        else:
            rc = run_integration_tests(crate_name, test_filter)

        input("Press Enter to shutdown containers and clean up...")
    finally:
        if not skip_docker:
            docker_down()
            docker_clean()
        return rc
