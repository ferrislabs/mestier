import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/** Moved to the Projet module; the old address still resolves. */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/projects/$projectId',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, `/projet/${params.projectId}`),
		})
	},
})
