---
title: "Configuration Guide"
weight: 3
---

# Configuration Guide

This guide explains how to configure Brokkr components using environment variables and configuration files.

## Configuration Sources

Configuration values are loaded and overridden in the following order (later sources take precedence):

1. Default values from the embedded `default.toml` file
2. Values from an optional external configuration file (if provided)
3. Environment variables

## Environment Variables

All environment variables are prefixed with `BROKKR__` and use double underscores (`__`) as separators.

### Database Configuration

- `BROKKR__DATABASE__URL`: Sets the database connection URL
  - Default: "postgres://brokkr:brokkr@localhost:5432/brokkr"

### Logging Configuration

- `BROKKR__LOG__LEVEL`: Sets the log level for the application
  - Default: "debug"
  - Possible values: "trace", "debug", "info", "warn", "error"

### PAK Configuration

- `BROKKR__PAK__PREFIX`: Sets the prefix for PAKs (Pre-Authentication Keys)
  - Default: "brokkr"

- `BROKKR__PAK__RNG`: Sets the random number generator type for PAK generation
  - Default: "osrng"

- `BROKKR__PAK__DIGEST`: Sets the digest algorithm for PAK generation
  - Default: 8

- `BROKKR__PAK__SHORT_TOKEN_LENGTH`: Sets the length of short PAK tokens
  - Default: 8

- `BROKKR__PAK__LONG_TOKEN_LENGTH`: Sets the length of long PAK tokens
  - Default: 24

- `BROKKR__PAK__SHORT_TOKEN_PREFIX`: Sets the prefix for short PAK tokens
  - Default: "BR"

### Agent Configuration

- `BROKKR__AGENT__BROKER_URL`: Sets the URL of the broker service
  - Required for agent operation

- `BROKKR__AGENT__POLLING_INTERVAL`: Sets the polling interval in seconds
  - Default: 30

- `BROKKR__AGENT__KUBECONFIG_PATH`: Sets the path to the kubeconfig file
  - Optional, defaults to standard kubeconfig location

- `BROKKR__AGENT__MAX_RETRIES`: Sets the maximum number of retries for operations
  - Default: 3

- `BROKKR__AGENT__PAK`: Sets the agent's PAK
  - Required for agent operation

- `BROKKR__AGENT__NAME`: Sets the agent's name
  - Required for agent operation

- `BROKKR__AGENT__CLUSTER_NAME`: Sets the name of the cluster the agent manages
  - Required for agent operation

- `BROKKR__AGENT__MAX_EVENT_MESSAGE_RETRIES`: Sets the maximum number of retries for event messages
  - Default: 3

- `BROKKR__AGENT__EVENT_MESSAGE_RETRY_DELAY`: Sets the delay between event message retries in seconds
  - Default: 5

## Configuration File Format

You can also provide configuration through a TOML file. The file should follow this structure:

```toml
[database]
url = "postgres://brokkr:brokkr@localhost:5432/brokkr"

[log]
level = "debug"

[pak]
prefix = "brokkr"
rng = "osrng"
digest = 8
short_token_length = 8
long_token_length = 24
short_token_prefix = "BR"

[agent]
broker_url = "http://localhost:3000"
polling_interval = 30
kubeconfig_path = "/path/to/kubeconfig"
max_retries = 3
pak = "your-pak-here"
name = "my-agent"
cluster_name = "prod-cluster"
max_event_message_retries = 3
event_message_retry_delay = 5
```

## Best Practices

1. **Environment Variables**: Use environment variables for sensitive information like database URLs and PAKs.

2. **Configuration Files**: Use configuration files for static configuration that doesn't change between environments.

3. **Default Values**: Only override default values when necessary. The defaults are chosen to work in most scenarios.

4. **Security**: Never commit configuration files containing sensitive information to version control.

## Troubleshooting

### Common Configuration Issues

1. **Database Connection**: If you see database connection errors, verify:
   - The database URL is correct
   - The database is running and accessible
   - The credentials are correct

2. **Agent Connection**: If the agent can't connect to the broker, check:
   - The broker URL is correct
   - The PAK is valid
   - Network connectivity between agent and broker

3. **Kubernetes Access**: If the agent can't access Kubernetes, verify:
   - The kubeconfig path is correct
   - The kubeconfig has valid credentials
   - The agent has necessary permissions

### Getting Help

If you encounter configuration issues:
1. Check the logs for detailed error messages
2. Verify all required configuration values are set
3. Check our [GitHub Issues](https://github.com/colliery-io/brokkr/issues) for known issues
