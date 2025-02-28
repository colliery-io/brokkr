---
title: "Installation Guide"
weight: 1
---

# Installing Brokkr

This guide will help you install Brokkr and set up your environment.

## System Requirements

### Hardware Requirements
- CPU: 2+ cores recommended
- RAM: 2GB minimum, 4GB recommended
- Disk: 1GB available space

### Software Requirements
- Operating System: Linux, macOS, or Windows
- Kubernetes cluster (v1.20+)
- `kubectl` CLI tool
- Docker (for container deployments)

## Installation Methods

### Binary Installation (Recommended)
```bash
# Download the latest release
curl -LO https://github.com/colliery-io/brokkr/releases/latest/download/brokkr-$(uname -s)-$(uname -m)

# Make it executable
chmod +x brokkr-$(uname -s)-$(uname -m)

# Move to your PATH
sudo mv brokkr-$(uname -s)-$(uname -m) /usr/local/bin/brokkr
```

### Using Docker
```bash
docker pull colliery/brokkr:latest
```

### Building from Source
```bash
# Clone the repository
git clone https://github.com/colliery-io/brokkr.git
cd brokkr

# Build using Cargo
cargo build --release

# The binary will be available at target/release/brokkr
```

## Initial Configuration

### Basic Setup
1. Create a configuration directory:
```bash
mkdir -p ~/.config/brokkr
```

2. Generate a default configuration:
```bash
brokkr init
```

### Verifying the Installation
```bash
# Check the version
brokkr --version

# Verify the configuration
brokkr config validate
```

## Next Steps
- Follow our [Quick Start Guide](../quick-start) to deploy your first application
- Learn about [Basic Concepts](../first-steps) in Brokkr
- Configure [Environment Management](../../how-to/environment-management)

## Troubleshooting

### Common Issues
- **Permission Denied**: Ensure proper file permissions when installing
- **Missing Dependencies**: Check system requirements
- **Configuration Errors**: Verify your kubeconfig and Brokkr configuration

### Getting Help
- Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues)
- Join our [Community Discord](https://discord.gg/brokkr)
- Read the [FAQ](../../reference/faq)
