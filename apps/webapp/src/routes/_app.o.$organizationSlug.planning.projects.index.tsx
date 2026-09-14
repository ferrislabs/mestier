import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/**
 * Projects moved to the Planification module (#468). The address stays alive
 * because it is bookmarked and pasted into chat.
 *
 * `validateSearch` passes the query string through untouched rather than
 * reusing `projectsSearchSchema`: the router strips search a route does not
 * declare, and parsing here would also stamp the schema's defaults onto a
 * bare URL. The destination validates for real.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/projects/',
)({
	validateSearch: (search: Record<string, unknown>) => search,
	beforeLoad: ({ params, search }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, '/planification/projects'),
			search,
		})
	},
})
