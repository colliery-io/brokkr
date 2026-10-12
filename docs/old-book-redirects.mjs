// The old mdBook addresses of the pages whose folder changed in the move to
// Starlight (BROKKR-T-0350). mdbookRedirects() covers each page that kept its
// path; this adds the rest, as `extra` for it.
import { existsSync, readdirSync } from 'node:fs';
import { posix } from 'node:path';
import { fileURLToPath } from 'node:url';
import { MDBOOK_EXTRA } from 'starlight-aurora/mdbook-redirects';

// Old page (without .html) -> new route.
const MOVED = {
	'getting-started/index': 'tutorials/getting-started',
	'getting-started/evaluate': 'tutorials/evaluate',
	'getting-started/installation': 'how-to/installation',
	'getting-started/configuration': 'how-to/configuration',
	'getting-started/development': 'how-to/development',
	'reference/api/index': 'reference/rest-api',
	'api/index': 'reference/rust-api',
	'api/sdks': 'reference/client-sdks',
	// mdBook wrote each README.md as index.html, but the links of the old book
	// went to README.html (BROKKR-T-0311). Send those to the page too.
	README: '',
	'getting-started/README': 'tutorials/getting-started',
	'tutorials/README': 'tutorials',
	'how-to/README': 'how-to',
	'how-to/sdks/README': 'how-to/sdks',
	'explanation/README': 'explanation',
	'reference/README': 'reference',
	'reference/api/README': 'reference/rest-api',
	'api/README': 'reference/rust-api',
	// A Swagger UI page with an old copy of the spec. The REST API reference
	// replaces it.
	'openapi/index': 'reference/rest-api',
};

// The Rust API pages were at api/rust/<module>.html. plissken writes them
// to reference/api/rust/ before the build.
const RUST_API = fileURLToPath(new URL('./src/content/docs/reference/api/rust/', import.meta.url));

function rustApiPages(dir = RUST_API, prefix = '') {
	if (!existsSync(dir)) return [];
	return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
		if (e.isDirectory()) return rustApiPages(`${dir}${e.name}/`, `${prefix}${e.name}/`);
		if (!e.name.endsWith('.md')) return [];
		const page = `${prefix}${e.name.slice(0, -3)}`.replace(/(^|\/)index$/, '');
		return [page];
	});
}

function target(old, route) {
	const rel = posix.relative(posix.join('/', posix.dirname(old)), posix.join('/', route));
	return rel ? `${rel}/` : './';
}

export function oldBookExtra() {
	const moved = { ...MOVED };
	for (const page of rustApiPages()) moved[`api/rust/${page}`] = `reference/api/rust/${page}`;
	const extra = { ...MDBOOK_EXTRA };
	for (const [old, route] of Object.entries(moved)) extra[`${old}.html`] = target(old, route);
	return extra;
}
