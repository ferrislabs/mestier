import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/** The DOE is reached from its project now; the old list address lands on the projects. */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/does',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, '/planification/projects'),
		})
	},
})
