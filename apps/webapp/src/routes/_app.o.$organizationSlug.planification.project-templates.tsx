import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/** Moved to Settings › Modèles; the old address still resolves. */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/project-templates',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, '/settings'),
			hash: 'modeles',
		})
	},
})
