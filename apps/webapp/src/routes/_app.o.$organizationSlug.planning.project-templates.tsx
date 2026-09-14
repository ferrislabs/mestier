import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/**
 * Moved to the Planification module (#468); the old address still resolves.
 * `validateSearch` passes the query string through untouched — see
 * `planning.projects.index.tsx` for why it is not the real schema.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/project-templates',
)({
	validateSearch: (search: Record<string, unknown>) => search,
	beforeLoad: ({ params, search }) => {
		throw redirect({
			to: buildOrgPath(
				params.organizationSlug,
				'/planification/project-templates',
			),
			search,
		})
	},
})
