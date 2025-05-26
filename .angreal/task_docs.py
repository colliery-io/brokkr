"""
Documentation tasks for your project.
"""

import subprocess
import sys
from pathlib import Path
import shutil

import angreal

# Project root for accessing docs, etc. (one level up from .angreal)
PROJECT_ROOT = Path(angreal.get_root()).parent

# Define command group
docs = angreal.command_group(name="docs", about="commands for documentation tasks")


def _integrate_rustdoc():
    """Generate rustdoc and integrate it with the Hugo documentation site."""
    print("Generating rustdoc...")

    # Generate rustdoc
    try:
        subprocess.run(
            ["cargo", "doc", "--no-deps"],
            check=True
        )
    except subprocess.CalledProcessError as e:
        print(f"Failed to generate rustdoc: {e}", file=sys.stderr)
        return e.returncode

    # Setup paths
    hugo_docs_dir = PROJECT_ROOT / "docs"
    rustdoc_output_dir = PROJECT_ROOT / "target/doc"
    hugo_api_dir = hugo_docs_dir / "static/api"

    # Create Hugo API directory if it doesn't exist
    hugo_api_dir.mkdir(parents=True, exist_ok=True)

    # Copy rustdoc output to Hugo static directory
    print("Copying rustdoc output to Hugo...")
    try:
        # Remove existing API docs
        if hugo_api_dir.exists():
            shutil.rmtree(hugo_api_dir)
        # Copy new API docs
        shutil.copytree(rustdoc_output_dir, hugo_api_dir)
    except Exception as e:
        print(f"Failed to copy rustdoc output: {e}", file=sys.stderr)
        return 1

    print("Rustdoc integration complete!")
    return 0


@docs()
@angreal.command(name="serve", about="serve the documentation site locally")
def serve():
    """Serve the Hugo documentation site locally with integrated API docs."""
    print("=== Setting up documentation ===")

    # First integrate rustdoc
    print("\nIntegrating API documentation...")
    rustdoc_result = _integrate_rustdoc()
    if rustdoc_result != 0:
        return rustdoc_result

    # Then start Hugo server
    print("\n=== Starting Hugo server ===")
    print("Documentation will be available at http://localhost:1313")
    print("Press Ctrl+C to stop the server")

    try:
        result = subprocess.run(
            ["hugo", "server", "-D"],
            cwd=str(PROJECT_ROOT / "docs"),
            check=True
        )
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Hugo server failed: {e}", file=sys.stderr)
        return e.returncode


@docs()
@angreal.command(name="build", about="build the documentation site")
def build():
    """Build the Hugo documentation site."""
    print("=== Building documentation site ===")

    # First integrate rustdoc
    print("\nIntegrating API documentation...")
    rustdoc_result = _integrate_rustdoc()
    if rustdoc_result != 0:
        return rustdoc_result

    # Then build Hugo site
    print("\nBuilding Hugo site...")
    try:
        result = subprocess.run(
            ["hugo"],
            cwd=str(PROJECT_ROOT / "docs"),
            check=True
        )
        if result.returncode == 0:
            print("\n=== Build complete ===")
            print(f"Documentation site built in {PROJECT_ROOT}/docs/public")
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Failed to build documentation: {e}", file=sys.stderr)
        return e.returncode
