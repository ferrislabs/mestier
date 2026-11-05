import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/doe-models/',
)({
	validateSearch: (search: Record<string, unknown>) => search,
	beforeLoad: ({ params, search }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, '/projet/doe-models'),
			search,
		})
	},
})
