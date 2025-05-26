---
title: "API Reference"
description: "Complete API documentation for Brokkr"
date: 2024-03-20
weight: 1
---

# API Reference

Welcome to the API documentation. This section contains the complete API reference for Brokkr, automatically generated from the source code.

## Quick Links

- [Core Library](/api/brokkr/)
- [Macros](/api/brokkr_macros/)

## Using the API Documentation

The API documentation is generated directly from the source code using `rustdoc`. Each function, type, and module is documented with examples and detailed explanations.

You can use the search functionality to quickly find specific items, or browse through the modules using the navigation menu.

## Cross-References

Throughout the rest of the documentation, you'll find links to specific API items using the `{{</* api-link */>}}` shortcode. For example:

```markdown
{{</* api-link path="brokkr::Config" */>}}
```

This will create a link to the `Config` type in the API documentation.

## Key Components

- [Configuration](/api/brokkr/config/) - Core configuration types
- [Error Handling](/api/brokkr/error/) - Error types and utilities
- [Core Traits](/api/brokkr/traits/) - Essential traits and implementations

## Common Patterns

### Configuration
```rust
use brokkr::Config;

let config = Config::default();
```

## Architecture

The API is organized into several key modules:

- `core/` - Core functionality and types
- `utils/` - Utility functions and helpers
- `macros/` - Procedural macros

## Related Documentation

- [Getting Started Guide](/getting-started/)
- [Architecture Overview](/architecture/)
- [Best Practices](/best-practices/)

## Working with Documentation

The documentation system is managed through angreal commands. Here are the main commands:

### Development

To serve the documentation locally with live reload:
```bash
angreal docs serve
```

This command will:
1. Generate the Rust API documentation
2. Copy it to the Hugo static directory
3. Start the Hugo development server
4. Open the documentation at http://localhost:1313

### Building for Production

To build the complete documentation site:
```bash
angreal docs build
```

This will:
1. Generate the Rust API documentation
2. Copy it to the Hugo static directory
3. Build the complete Hugo site
4. Output the site to `docs/public/`

## Troubleshooting

If you encounter issues:

1. Make sure all prerequisites are installed
2. Check that the Hugo server is running
3. Verify that rustdoc generation is working
4. Check the Hugo server logs for any errors
5. Ensure all paths in `api-link` shortcodes are correct
6. Make sure your crate name in the shortcodes matches your actual crate name
