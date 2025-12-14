import angreal
import subprocess
from utils import cwd
import platform

build = angreal.command_group(name="build", about="commands for building container images")


def ensure_buildx_builder(builder_name="brokkr-builder"):
    """Ensure Docker Buildx builder exists and is active."""
    result = subprocess.run(
        ["docker", "buildx", "ls"],
        cwd=cwd,
        capture_output=True,
        text=True
    )

    if builder_name not in result.stdout:
        print(f"Creating Docker Buildx builder: {builder_name}")
        subprocess.run(
            ["docker", "buildx", "create", "--name", builder_name, "--use"],
            cwd=cwd,
            check=True
        )
    else:
        print(f"Using existing Docker Buildx builder: {builder_name}")
        subprocess.run(
            ["docker", "buildx", "use", builder_name],
            cwd=cwd,
            check=True
        )


def build_multiarch_image(component, dockerfile, platforms, tag, registry, push):
    """Build a multi-architecture image using Docker Buildx."""
    image_name = f"{registry}/brokkr-{component}:{tag}"

    print("")
    print("=" * 60)
    print(f"Building: brokkr-{component}")
    print(f"Dockerfile: {dockerfile}")
    print(f"Tag: {tag}")
    print(f"Platforms: {platforms}")
    print(f"Image: {image_name}")
    print("=" * 60)
    print("")

    cmd = [
        "docker", "buildx", "build",
        "--platform", platforms,
        "-f", dockerfile,
        "-t", image_name,
    ]

    if push:
        cmd.append("--push")
    else:
        cmd.append("--load")

    cmd.append(".")

    result = subprocess.run(cmd, cwd=cwd)

    if result.returncode == 0:
        print("")
        print(f"Successfully built: {image_name}")
    else:
        print("")
        print(f"Failed to build: {image_name}")

    return result.returncode


@build()
@angreal.command(name="multi-arch", about="build multi-architecture container images")
@angreal.argument(name="component", required=True, help="Component to build (broker, agent, ui, all)")
@angreal.argument(name="push", long="push", help="Push images to registry", takes_value=False, is_flag=True)
@angreal.argument(name="tag", long="tag", help="Image tag (default: dev)", default_value="dev")
@angreal.argument(name="platforms", long="platforms", help="Platforms to build (default: linux/amd64,linux/arm64)", default_value="linux/amd64,linux/arm64")
@angreal.argument(name="registry", long="registry", help="Registry URL (default: ghcr.io/colliery-io)", default_value="ghcr.io/colliery-io")
def multi_arch(component, push=False, tag="dev", platforms="linux/amd64,linux/arm64", registry="ghcr.io/colliery-io"):
    """
    Build multi-architecture container images using Docker Buildx.

    Examples:
        angreal build multi-arch broker --tag latest
        angreal build multi-arch all --push --tag v1.0.0
        angreal build multi-arch agent --platforms linux/arm64
    """
    # Validate component
    valid_components = ["broker", "agent", "ui", "all"]
    if component not in valid_components:
        print(f"Error: Unknown component '{component}'")
        print(f"Valid components: {', '.join(valid_components)}")
        return 1

    # If loading locally, only support single platform
    if not push and "," in platforms:
        print("Warning: --load only supports single platform. Building for current platform only.")
        arch = platform.machine()
        if arch in ["arm64", "aarch64"]:
            platforms = "linux/arm64"
        else:
            platforms = "linux/amd64"
        print(f"Using platform: {platforms}")

    # Ensure buildx builder exists
    ensure_buildx_builder()

    # Build components
    components_to_build = []
    if component == "all":
        components_to_build = [
            ("broker", "docker/Dockerfile.broker"),
            ("agent", "docker/Dockerfile.agent"),
            ("ui", "docker/Dockerfile.ui"),
        ]
    else:
        dockerfile = f"docker/Dockerfile.{component}"
        components_to_build = [(component, dockerfile)]

    # Build all components
    return_codes = []
    for comp_name, comp_dockerfile in components_to_build:
        rc = build_multiarch_image(
            comp_name,
            comp_dockerfile,
            platforms,
            tag,
            registry,
            push
        )
        return_codes.append((comp_name, rc))

    # Report results
    print("")
    print("=" * 60)
    if all(rc == 0 for _, rc in return_codes):
        print("Build complete! All images built successfully.")
    else:
        print("Build completed with errors:")
        for comp_name, rc in return_codes:
            if rc != 0:
                print(f"  - {comp_name}: failed with return code {rc}")
    print("=" * 60)

    return max(rc for _, rc in return_codes)
