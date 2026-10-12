// The API pages under reference/api/ come from `plissken render` and are not
// in the repository, so they get no "Edit page" link.
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';

export const onRequest = defineRouteMiddleware((context) => {
	const route = context.locals.starlightRoute;
	if (route.id.startsWith('reference/api/')) route.editUrl = undefined;
});
