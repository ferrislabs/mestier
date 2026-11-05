import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/doe-models/$modelId',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(
				params.organizationSlug,
				`/projet/doe-models/${params.modelId}`,
			),
		})
	},
})
