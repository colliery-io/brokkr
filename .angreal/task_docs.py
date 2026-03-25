"""
Documentation tasks for the Brokkr project.

Uses plissken to generate Rust API reference docs from source code,
then builds/serves the mdBook documentation site.
"""

import re
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
SUMMARY_PATH = DOCS_DIR / "src" / "SUMMARY.md"

# Markers in SUMMARY.md that delimit the auto-generated API section
API_SECTION_START = "# API Documentation"

# Define command group
docs = angreal.command_group(name="docs", about="commands for documentation tasks")


def _rewrite_summary_api_section():
    """Rewrite the API Documentation section in SUMMARY.md from plissken's generated SUMMARY."""
    plissken_summary = PLISSKEN_STAGING / "src" / "SUMMARY.md"
    if not plissken_summary.exists():
        print(f"Plissken SUMMARY.md not found at {plissken_summary}", file=sys.stderr)
        return 1

    # Read plissken's generated entries (skip the header lines)
    with open(plissken_summary) as f:
        plissken_lines = f.readlines()

    # Extract just the entry lines (skip "# Summary", blank lines, "# Rust" header)
    entries = []
    for line in plissken_lines:
        if line.strip().startswith("- [") or line.strip().startswith("- ["):
            # Prefix paths: rust/foo.md -> ./api/rust/foo.md
            entry = re.sub(r'\]\(rust/', '](./api/rust/', line)
            # Add two spaces of indentation to nest under "Rust API Reference"
            entries.append("  " + entry)
        elif entries and line.strip() == "":
            continue  # skip blanks between entries

    if not entries:
        print("No entries found in plissken SUMMARY.md", file=sys.stderr)
        return 1

    # Read current SUMMARY.md
    with open(SUMMARY_PATH) as f:
        summary = f.read()

    # Find the API Documentation section and replace everything after it
    marker = API_SECTION_START
    idx = summary.find(marker)
    if idx == -1:
        print(f"Could not find '{marker}' in SUMMARY.md", file=sys.stderr)
        return 1

    # Build the new API section
    api_section = f"{marker}\n\n- [Rust API Reference](./api/README.md)\n"
    api_section += "".join(entries)

    # Replace from the marker to end of file
    new_summary = summary[:idx] + api_section
    with open(SUMMARY_PATH, "w") as f:
        f.write(new_summary)

    print(f"Updated SUMMARY.md with {len(entries)} API entries")
    return 0


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

    # Update SUMMARY.md with the full module hierarchy
    result = _rewrite_summary_api_section()
    if result != 0:
        return result

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
