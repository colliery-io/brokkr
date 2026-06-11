import angreal # type: ignore
import subprocess
from utils import docker_up, docker_down, cwd, docker_clean
import time
test = angreal.command_group(name="tests", about="commands for test suites")




def get_crates():
    """Get all crates in the workspace."""
    return {
        "integration_tests": ["brokkr-agent", "brokkr-broker"],
        "unit_tests": ["brokkr-agent", "brokkr-broker", "brokkr-models", "brokkr-utils", "brokkr-wire"]
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
    """Run integration tests on the host, connecting to dockerized services."""
    import os
    env = os.environ.copy()
    env["BROKKR__DATABASE__URL"] = "postgres://brokkr:brokkr@localhost:5433/brokkr"
    env["KUBECONFIG"] = "/tmp/brokkr-keys/kubeconfig.local.yaml"

    cmd = ["cargo", "test", "--test", "integration"]
    if crate_name:
        cmd.extend(["-p", crate_name])
    if test_filter:
        cmd.extend(["--", test_filter, "--test-threads=1", "--nocapture"])
    else:
        cmd.extend(["--", "--test-threads=1", "--nocapture"])
    result = subprocess.run(cmd, cwd=cwd, env=env)
    return result.returncode

def run_e2e_tests(scenario: str = ""):
    """Run the holistic E2E test suite, optionally filtered to a single scenario.

    When ``scenario`` is empty, runs the full demo-walkthrough suite (default).
    When ``scenario`` is set (e.g. ``ws-smoke``), only that scenario runs — the
    Rust binary branches on the ``E2E_SCENARIO`` env var.
    """
    import os

    # Build the E2E test binary
    print("Building E2E test suite...")
    build_result = subprocess.run(
        ["cargo", "build", "--release", "--manifest-path", "tests/e2e/Cargo.toml"],
        cwd=cwd
    )
    if build_result.returncode != 0:
        return build_result.returncode

    # Run the E2E binary
    if scenario:
        print(f"Running E2E scenario: {scenario}")
    else:
        print("Running E2E tests...")
    env = os.environ.copy()
    env["BROKER_URL"] = "http://localhost:3000"
    env["ADMIN_PAK"] = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"
    if scenario:
        env["E2E_SCENARIO"] = scenario
        # Scenarios like ws-smoke shell out to docker; tell them which compose
        # file the angreal task brought up so they can stop/start services.
        env["E2E_COMPOSE_FILE"] = os.path.join(cwd, ".angreal", "files", "docker-compose.yaml")

    result = subprocess.run(
        ["./tests/e2e/target/release/brokkr-e2e"],
        cwd=cwd,
        env=env
    )
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


def _sdk_contract_env():
    import os
    env = os.environ.copy()
    env.setdefault("BROKER_URL", "http://localhost:3000")
    env.setdefault(
        "ADMIN_PAK", "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"
    )
    return env


def run_sdk_contract_rust():
    """Build and run the Rust SDK contract suite."""
    env = _sdk_contract_env()

    print("Building Rust SDK contract suite...")
    build = subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            "tests/sdk-contract/rust/Cargo.toml",
        ],
        cwd=cwd,
    )
    if build.returncode != 0:
        return build.returncode

    print("Running Rust SDK contract suite...")
    result = subprocess.run(
        ["./tests/sdk-contract/rust/target/release/brokkr-sdk-contract-rust"],
        cwd=cwd,
        env=env,
    )
    return result.returncode


def run_sdk_contract_python():
    """Install and run the Python SDK contract suite via pytest.

    Uses `uv` to create an isolated venv under the suite directory and to
    install the locally-generated Python SDK + pytest. Falls back to
    `python -m pip` if `uv` is unavailable.
    """
    import shutil
    import sys

    env = _sdk_contract_env()
    suite_dir = "tests/sdk-contract/python"
    venv_dir = f"{suite_dir}/.venv"

    if shutil.which("uv"):
        print("Provisioning Python SDK contract suite venv via uv...")
        steps = [
            ["uv", "venv", "--python", "3.12", venv_dir],
            [
                "uv",
                "pip",
                "install",
                "--python",
                f"{venv_dir}/bin/python",
                "--quiet",
                "-e",
                "sdks/python/brokkr-client",
                "-e",
                "sdks/python/brokkr",
                "pytest>=8.0",
            ],
        ]
        for step in steps:
            r = subprocess.run(step, cwd=cwd)
            if r.returncode != 0:
                return r.returncode
        py = f"{venv_dir}/bin/python"
    else:
        print("Installing Python SDK contract suite dependencies (pip)...")
        install = subprocess.run(
            [
                sys.executable,
                "-m",
                "pip",
                "install",
                "--quiet",
                "-e",
                "sdks/python/brokkr-client",
                "-e",
                "sdks/python/brokkr",
                "pytest>=8.0",
            ],
            cwd=cwd,
        )
        if install.returncode != 0:
            return install.returncode
        py = sys.executable

    print("Running Python SDK contract suite...")
    result = subprocess.run(
        [py, "-m", "pytest", suite_dir, "-v"],
        cwd=cwd,
        env=env,
    )
    return result.returncode


def run_sdk_contract_typescript():
    """Install and run the TypeScript SDK contract suite via vitest."""
    env = _sdk_contract_env()
    suite_dir = "tests/sdk-contract/typescript"

    # The local TS SDK needs its own deps installed before `tsc` can resolve
    # `openapi-fetch` etc. Locally a populated `node_modules/` masks this; in
    # fresh CI clones it fails the build. Install, then build.
    print("Installing TypeScript SDK dependencies...")
    sdk_install = subprocess.run(
        ["npm", "--prefix", "sdks/typescript/brokkr-client", "install"],
        cwd=cwd,
    )
    if sdk_install.returncode != 0:
        return sdk_install.returncode

    print("Building TypeScript SDK (dist/)...")
    sdk_build = subprocess.run(
        ["npm", "--prefix", "sdks/typescript/brokkr-client", "run", "build"],
        cwd=cwd,
    )
    if sdk_build.returncode != 0:
        return sdk_build.returncode

    print("Installing TypeScript SDK contract suite dependencies...")
    install = subprocess.run(
        ["npm", "--prefix", suite_dir, "install"],
        cwd=cwd,
    )
    if install.returncode != 0:
        return install.returncode

    print("Running TypeScript SDK contract suite...")
    result = subprocess.run(
        ["npm", "--prefix", suite_dir, "test"],
        cwd=cwd,
        env=env,
    )
    return result.returncode


SDK_CONTRACT_LANGUAGES = {
    "rust": run_sdk_contract_rust,
    "python": run_sdk_contract_python,
    "typescript": run_sdk_contract_typescript,
}


@test()
@angreal.command(
    name="sdk-contract",
    about="run SDK contract tests for a given language (or all)",
)
@angreal.argument(
    name="skip_docker",
    long="skip-docker",
    required=False,
    help="Skip docker compose up",
    takes_value=False,
    is_flag=True,
)
@angreal.argument(
    name="language",
    required=True,
    help=f"Language to test ({list(SDK_CONTRACT_LANGUAGES) + ['all']})",
)
def sdk_contract_tests(language: str, skip_docker: bool = False):
    """Run SDK contract tests that exercise the generated SDK against a running broker.

    These tests close the gap that lets consumer-visible drift (status codes,
    content types, auth scopes) escape spec-drift CI — see BROKKR-T-0154.
    """
    if not skip_docker:
        docker_clean()
        docker_up()
        print("Sleeping for 30 seconds, waiting for services to stabilize - get some coffee")
        time.sleep(30)

    rc = 0
    try:
        if language == "all":
            return_codes = []
            for lang, runner in SDK_CONTRACT_LANGUAGES.items():
                print(f"\n=== SDK contract: {lang} ===\n")
                return_codes.append((lang, runner()))
            failed = [(l, c) for l, c in return_codes if c != 0]
            if failed:
                for l, c in failed:
                    print(f"sdk-contract:{l} failed with return code {c}")
                rc = max(c for _, c in failed)
        else:
            runner = SDK_CONTRACT_LANGUAGES.get(language)
            if runner is None:
                print(
                    f"unknown sdk-contract language '{language}' (expected one of "
                    f"{list(SDK_CONTRACT_LANGUAGES) + ['all']})"
                )
                rc = 2
            else:
                rc = runner()

        if not skip_docker:
            input("Press Enter to shutdown containers and clean up...")
    finally:
        if not skip_docker:
            docker_down()
            docker_clean()
        return rc


@test()
@angreal.command(name="e2e", about="run holistic E2E tests (mirrors UI demo walkthrough)")
@angreal.argument(name="skip_docker", long="skip-docker", required=False, help="Skip docker compose up", takes_value=False, is_flag=True)
@angreal.argument(name="scenario", long="scenario", required=False, help="Run only a single scenario (e.g. 'ws-smoke'). Omit for full suite.")
def e2e_tests(skip_docker: bool = False, scenario: str = ""):
    """Run holistic E2E tests that exercise the entire Brokkr system.

    These tests mirror the UI demo walkthrough and test the complete
    system from end to end: broker, agent, database, and kubernetes.

    Pass ``--scenario <name>`` to run only a single targeted scenario (e.g.
    ``ws-smoke``, which exercises the I-0019 WS channel against a real
    broker/agent docker-compose stack with a stop/start cycle).
    """
    if not skip_docker:
        docker_clean()
        docker_up()

    print("Sleeping for 30 seconds, waiting for services to stabilize - get some coffee")
    time.sleep(30)

    rc = None
    try:
        rc = run_e2e_tests(scenario=scenario)
        if not skip_docker:
            input("Press Enter to shutdown containers and clean up...")
    finally:
        if not skip_docker:
            docker_down()
            docker_clean()
        return rc
