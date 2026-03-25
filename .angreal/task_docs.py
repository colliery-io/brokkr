"""
Documentation tasks for the Brokkr project.

Uses plissken to generate Rust API reference docs from source code,
then builds/serves the mdBook documentation site.
"""

import subprocess
import sys
from pathlib import Path
import shutil

import angreal

# Project root for accessing docs, etc. (one level up from .angreal)
PROJECT_ROOT = Path(angreal.get_root()).parent
DOCS_DIR = PROJECT_ROOT / "docs"
PLISSKEN_STAGING = PROJECT_ROOT / ".plissken" / "output"
API_RUST_DIR = DOCS_DIR / "src" / "api" / "rust"

# Define command group
docs = angreal.command_group(name="docs", about="commands for documentation tasks")


def _integrate_plissken():
    """Generate Rust API docs with plissken and integrate into mdBook source tree."""
    print("Generating Rust API reference with plissken...")

    # Run plissken render
    try:
        subprocess.run(
            ["plissken", "render"],
            cwd=str(PROJECT_ROOT),
            check=True,
        )
    except FileNotFoundError:
        print(
            "plissken not found. Install it with: cargo install plissken",
            file=sys.stderr,
        )
        return 1
    except subprocess.CalledProcessError as e:
        print(f"plissken render failed: {e}", file=sys.stderr)
        return e.returncode

    # Copy generated rust/ content into docs source tree
    src = PLISSKEN_STAGING / "src" / "rust"
    if not src.exists():
        print(f"Expected plissken output not found at {src}", file=sys.stderr)
        return 1

    # Clean and copy
    if API_RUST_DIR.exists():
        shutil.rmtree(API_RUST_DIR)
    shutil.copytree(src, API_RUST_DIR)

    print(f"Rust API docs integrated into {API_RUST_DIR}")
    return 0


@docs()
@angreal.command(name="serve", about="serve the documentation site locally")
def serve():
    """Serve the mdBook documentation site locally with generated API docs."""
    print("=== Setting up documentation ===")

    # Generate and integrate plissken API docs
    print("\nGenerating Rust API reference...")
    result = _integrate_plissken()
    if result != 0:
        return result

    # Start mdBook server
    print("\n=== Starting mdBook server ===")
    print("Documentation will be available at http://localhost:3000")
    print("Press Ctrl+C to stop the server")

    try:
        result = subprocess.run(
            ["mdbook", "serve"],
            cwd=str(DOCS_DIR),
            check=True,
        )
        return result.returncode
    except FileNotFoundError:
        print("mdbook not found. Install it from https://github.com/rust-lang/mdBook", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as e:
        print(f"mdbook serve failed: {e}", file=sys.stderr)
        return e.returncode


@docs()
@angreal.command(name="build", about="build the documentation site")
def build():
    """Build the mdBook documentation site with generated API docs."""
    print("=== Building documentation site ===")

    # Generate and integrate plissken API docs
    print("\nGenerating Rust API reference...")
    result = _integrate_plissken()
    if result != 0:
        return result

    # Build mdBook site
    print("\nBuilding mdBook site...")
    try:
        result = subprocess.run(
            ["mdbook", "build"],
            cwd=str(DOCS_DIR),
            check=True,
        )
        if result.returncode == 0:
            print(f"\n=== Build complete ===")
            print(f"Documentation site built in {DOCS_DIR / 'book'}")
        return result.returncode
    except FileNotFoundError:
        print("mdbook not found. Install it from https://github.com/rust-lang/mdBook", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as e:
        print(f"Failed to build documentation: {e}", file=sys.stderr)
        return e.returncode
