import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'
import { workflowExecutionsLocation } from '#/pages/automation/lib/automation-paths'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions/',
)({
	beforeLoad: ({ params }) => {
		const { to, search } = workflowExecutionsLocation(params.workflowId)
		throw redirect({
			to: buildOrgPath(params.organizationSlug, to),
			search,
		})
	},
})
