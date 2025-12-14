"""
Documentation tasks for your project.
"""

import subprocess
import sys
from pathlib import Path
import shutil
import json

import angreal
from utils import docker_up, docker_down

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


def _integrate_openapi():
    """Generate OpenAPI documentation and integrate it with the Hugo documentation site."""
    print("Generating OpenAPI documentation...")

    # Setup paths
    hugo_docs_dir = PROJECT_ROOT / "docs"
    hugo_static_dir = hugo_docs_dir / "static"
    openapi_dir = hugo_static_dir / "openapi"
    openapi_dir.mkdir(parents=True, exist_ok=True)

    # Start the broker using Docker Compose
    try:
        # Start the broker using the utility function
        docker_up()

        # Wait for the broker to be ready
        import time
        import requests
        from requests.exceptions import RequestException

        max_retries = 30
        retry_delay = 2
        for attempt in range(max_retries):
            try:
                response = requests.get("http://localhost:3000/docs/openapi.json")
                response.raise_for_status()
                openapi_spec = response.json()

                # Save the OpenAPI spec
                with open(openapi_dir / "openapi.json", "w") as f:
                    json.dump(openapi_spec, f, indent=2)

                # Create a simple HTML page to display the Swagger UI
                swagger_html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Brokkr API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css">
    <script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js"></script>
</head>
<body>
    <div id="swagger-ui"></div>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/openapi/openapi.json",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
            });
        };
    </script>
</body>
</html>"""

                with open(openapi_dir / "index.html", "w") as f:
                    f.write(swagger_html)

                print("OpenAPI documentation integration complete!")
                return 0

            except RequestException:
                if attempt < max_retries - 1:
                    print(f"Waiting for broker to be ready... (attempt {attempt + 1}/{max_retries})")
                    time.sleep(retry_delay)
                else:
                    print("Failed to connect to broker after maximum retries", file=sys.stderr)
                    return 1

    except Exception as e:
        print(f"Failed to generate OpenAPI documentation: {e}", file=sys.stderr)
        return 1
    finally:
        # Stop the broker using the utility function
        try:
            docker_down()
        except Exception as e:
            print(f"Warning: Failed to stop broker: {e}", file=sys.stderr)


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

    # Then integrate OpenAPI docs
    print("\nIntegrating OpenAPI documentation...")
    openapi_result = _integrate_openapi()
    if openapi_result != 0:
        return openapi_result

    # Then start Hugo server
    print("\n=== Starting Hugo server ===")
    print("Documentation will be available at http://localhost:1313")
    print("API documentation will be available at http://localhost:1313/openapi")
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

    # Then integrate OpenAPI docs
    print("\nIntegrating OpenAPI documentation...")
    openapi_result = _integrate_openapi()
    if openapi_result != 0:
        return openapi_result

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
            print("API documentation available at /openapi")
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Failed to build documentation: {e}", file=sys.stderr)
        return e.returncode
