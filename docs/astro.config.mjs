// The Brokkr book (BROKKR-T-0350): Astro Starlight with the Colliery theme
// starlight-aurora, in the four Diátaxis sections.
import { existsSync } from 'node:fs';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';
import starlightAurora from 'starlight-aurora';
import mdbookRedirects from 'starlight-aurora/mdbook-redirects';
import { oldBookExtra } from './old-book-redirects.mjs';

// The Pages workflow sets DOCS_SITE and DOCS_BASE. On a pull request and
// locally they are empty, and the site is at /.
const site = process.env.DOCS_SITE || undefined;
const base = process.env.DOCS_BASE || '/';

const PROJECT = 'Brokkr';
const REPO = 'https://github.com/colliery-io/brokkr';

// The API pages come from `plissken render` (plissken.toml), which writes
// them and their sidebar group to reference/api/. `angreal docs build` and the
// docs workflow run it first. Without it the site builds with no API pages.
const API_SIDEBAR = new URL('./src/content/docs/reference/api/_sidebar.mjs', import.meta.url);
const apiSidebar = existsSync(API_SIDEBAR) ? (await import(API_SIDEBAR.href)).default : null;
if (!apiSidebar) console.warn('No API pages: run `plissken render` in the repository root.');

// The sidebar is written out, not generated: it keeps the order and the
// labels of the old SUMMARY.md. Add a new page here.
const page = (slug, label) => ({ slug, label });

export default defineConfig({
	site,
	base,
	// Mermaid is one large chunk, loaded only on a page with a diagram.
	vite: { build: { chunkSizeWarningLimit: 4096 } },
	integrations: [
		// Before Starlight. Light diagrams in the light theme; dark diagrams in
		// the dark theme and in hyper.
		mermaid({ theme: 'dark', autoTheme: true }),
		// The old mdBook addresses (how-to/x.html) go to the new pages.
		mdbookRedirects({ extra: oldBookExtra() }),
		starlight({
			title: PROJECT,
			description:
				'Brokkr: an environment-aware control plane that distributes Kubernetes objects to clusters.',
			social: [{ icon: 'github', label: 'GitHub', href: REPO }],
			editLink: { baseUrl: `${REPO}/edit/main/docs/` },
			expressiveCode: {
				frames: { extractFileNameFromCode: false },
				// Shiki has no PromQL grammar: show it as plain text.
				shiki: { langAlias: { promql: 'txt' } },
			},
			plugins: [starlightAurora({ hyper: true })],
			sidebar: [
				{ label: 'Introduction', link: '/' },
				{
					label: 'Getting Started',
					items: [
						page('tutorials/getting-started', 'Overview'),
						page('tutorials/evaluate', 'Evaluate Brokkr Locally'),
						page('how-to/installation', 'Installation'),
						page('how-to/configuration', 'Configuration'),
						page('how-to/development', 'Local Development Environment'),
					],
				},
				{
					label: 'Tutorials',
					items: [
						page('tutorials', 'Overview'),
						page('tutorials/first-tenant', 'Your First Tenant, From a Fresh Broker to a Running Manifest'),
						page('tutorials/first-deployment', 'Deploy Your First Application'),
						page('tutorials/multi-cluster-targeting', 'Multi-Cluster Targeting'),
						page('tutorials/cicd-generators', 'CI/CD with Generators'),
						page('tutorials/templates', 'Standardized Deployments with Templates'),
					],
				},
				{
					label: 'How-To Guides',
					items: [
						page('how-to', 'Overview'),
						page('how-to/cli-apply', 'Submitting a Folder of Manifests (CLI)'),
						page('how-to/managing-stacks', 'Managing Stacks'),
						page('how-to/templates', 'Using Stack Templates'),
						page('how-to/generators', 'Working with Generators'),
						page('how-to/agent-registration', 'Register Agents with Generators'),
						page('how-to/pak-management', 'Managing PAKs (Key Rotation)'),
						page('how-to/operator-console', 'Using the Operator Console'),
						page('how-to/fleet-monitoring', 'Monitoring Your Agent Fleet'),
						page('how-to/deployment-health', 'Monitoring Deployment Health'),
						page('how-to/log-streaming', 'Streaming Pod Logs & Live Tail'),
						page('how-to/diagnostics', 'Running On-Demand Diagnostics'),
						page('how-to/troubleshoot-reconciliation', 'Troubleshooting Reconciliation'),
						page('how-to/audit-logs', 'Querying Audit Logs'),
						page('how-to/webhooks', 'Configuring Webhooks'),
						{
							label: 'Using the SDKs',
							items: [
								page('how-to/sdks', 'Overview'),
								page('how-to/sdks/rust', 'Rust SDK'),
								page('how-to/sdks/python', 'Python SDK'),
								page('how-to/sdks/typescript', 'TypeScript SDK'),
								page('how-to/sdks/regeneration', 'Regenerating SDKs'),
							],
						},
						page('how-to/install-operations', 'Installing, Upgrading & Uninstalling'),
						page('how-to/monitoring-setup', 'Setting Up Monitoring'),
						page('how-to/network-configuration', 'Network Configuration'),
						page('how-to/security-hardening', 'Security Hardening'),
						page('how-to/multi-tenant-setup', 'Setting Up Multi-Tenant Operation'),
						page('how-to/shipwright-builds', 'Container Builds with Shipwright'),
						page('how-to/build-and-publish-images', 'Building & Publishing Images'),
					],
				},
				{
					label: 'Explanation',
					items: [
						page('explanation', 'Overview'),
						page('explanation/core-concepts', 'Core Concepts'),
						page('explanation/architecture', 'Technical Architecture'),
						page('explanation/components', 'Component Implementation'),
						page('explanation/data-model', 'Data Model'),
						page('explanation/network-flows', 'Network Flows'),
						page('explanation/data-flows', 'Data Flows'),
						page('explanation/security-model', 'Security Model'),
						page('explanation/publishing-strategy', 'Container Image Publishing Strategy'),
						page('explanation/work-orders', 'Work Orders & Build System'),
						page('explanation/template-system', 'Template Matching & Rendering'),
						page('explanation/reconciliation', 'Reconciliation Model'),
						page('explanation/internal-ws-channel', 'Internal Broker↔Agent WS Channel'),
						page('explanation/fleet-legibility', 'Fleet Legibility'),
					],
				},
				{
					label: 'Reference',
					items: [
						page('reference', 'Technical Reference'),
						page('reference/rest-api', 'REST API Reference'),
						page('reference/cli', 'CLI Reference'),
						page('reference/environment-variables', 'Environment Variables'),
						page('reference/error-codes', 'Stable Error Codes'),
						page('reference/templates', 'Templates'),
						page('reference/work-orders', 'Work Orders'),
						page('reference/webhooks', 'Webhooks'),
						page('reference/generators', 'Generators'),
						page('reference/diagnostics', 'Diagnostics'),
						page('reference/multi-tenancy', 'Multi-Tenancy'),
						page('reference/soft-deletion', 'Soft Deletion'),
						page('reference/audit-logs', 'Audit Logs'),
						page('reference/health-endpoints', 'Health Endpoints'),
						page('reference/deployment-health', 'Deployment Health'),
						page('reference/fleet', 'Fleet Observability'),
						page('reference/ws-protocol', 'WebSocket Protocol'),
						page('reference/agent-annotations', 'Agent Annotations & Labels'),
						page('reference/network-ports', 'Network Ports'),
						page('reference/container-images', 'Container Images'),
						page('reference/monitoring', 'Monitoring & Observability'),
						{
							label: 'API Documentation',
							items: [
								page('reference/rust-api', 'Overview'),
								...(apiSidebar ? apiSidebar.items : []),
								page('reference/client-sdks', 'Client SDKs'),
							],
						},
					],
				},
			],
		}),
	],
});
