# Brokkr UI Slim

A streamlined, lightweight dashboard for Brokkr that demonstrates all core features with a fraction of the code.

## Comparison

| Metric | Original UI | Slim UI |
|--------|-------------|---------|
| Lines of Code | ~4,000 | ~1,100 |
| Dependencies | 15+ (MUI, Monaco, etc.) | 3 (React only) |
| Bundle Size | ~2MB | ~200KB |
| Components | 6 files | 1 file |

## Features

All core features are available:

- **Agents**: View, labels, annotations, targets, events, status
- **Stacks**: Create, labels, annotations, deployment objects, YAML editing, **health status**
- **Templates**: Create, edit, instantiate with parameters
- **Jobs**: Work orders (create, cancel, view history, targeting by agent/labels)
- **Health Monitoring**: Live deployment health status (healthy/degraded/failing counts per agent)
- **Diagnostics**: On-demand health checks (pod statuses, events, logs)
- **Admin**: Agent/Generator PAK management, rotation

## Design

Industrial terminal aesthetic with:
- Dark theme with high-contrast accents
- Monospace typography (JetBrains Mono)
- Collapsible sections instead of multiple pages
- Inline editing instead of dialog-heavy workflows
- Native browser controls (no heavy UI library)

## Setup

```bash
npm install
npm start
```

## Environment Variables

```
REACT_APP_BROKER_URL=http://localhost:3000
REACT_APP_ADMIN_PAK=your_admin_pak_here
```
