"""
Documentation tasks for the Brokkr project.

The docs are an Astro Starlight site in docs/, with the Colliery theme
starlight-aurora (BROKKR-T-0350). They need Node 22 or later. plissken writes
the Rust and Python API pages into docs/src/content/docs/reference/api/
before each build (plissken.toml).
"""

import shutil
import subprocess
import sys
from pathlib import Path

import angreal

# Project root for accessing docs, etc. (one level up from .angreal)
PROJECT_ROOT = Path(angreal.get_root()).parent
DOCS_DIR = PROJECT_ROOT / "docs"

docs = angreal.command_group(name="docs", about="commands for documentation tasks")


def _plissken():
    """Write the API pages with plissken."""
    print("Writing the API pages with plissken...")
    plissken = shutil.which("plissken")
    if not plissken:
        print(
            "plissken not found. Install it with: cargo install plissken --version 0.2.1",
            file=sys.stderr,
        )
        return 1
    return subprocess.run([plissken, "render"], cwd=str(PROJECT_ROOT)).returncode


def _npm(*args):
    npm = shutil.which("npm")
    if not npm:
        print("npm not found: install Node 22 or later", file=sys.stderr)
        return 1
    if not (DOCS_DIR / "node_modules").exists():
        result = subprocess.run([npm, "ci", "--no-audit", "--no-fund"], cwd=str(DOCS_DIR))
        if result.returncode != 0:
            return result.returncode
    return subprocess.run([npm, *args], cwd=str(DOCS_DIR)).returncode


@docs()
@angreal.command(
    name="build",
    about="build the documentation site into docs/dist",
    tool=angreal.ToolDescription(
        "Writes the API pages with plissken, then builds the Starlight docs site "
        "in docs/ into docs/dist. Installs the npm dependencies on the first run. "
        "Fails when plissken or a page fails.",
        risk_level="safe",
    ),
)
def build():
    result = _plissken()
    if result != 0:
        return result
    return _npm("run", "build")


@docs()
@angreal.command(
    name="serve",
    about="serve the documentation site with live reload",
    tool=angreal.ToolDescription(
        "Writes the API pages with plissken, then serves the docs site at "
        "http://localhost:4321 and reloads on each change. In a terminal it runs "
        "until Ctrl+C. From a script or an agent, Astro runs it in the background: "
        "stop it with `npx astro dev stop` in docs/.",
        risk_level="safe",
    ),
)
def serve():
    result = _plissken()
    if result != 0:
        return result
    return _npm("run", "dev")
