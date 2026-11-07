import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'
import { legacyExecutionLocation } from '#/pages/automation/lib/automation-paths'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions/$runId',
)({
	beforeLoad: ({ params }) => {
		const { to, search } = legacyExecutionLocation(
			params.workflowId,
			params.runId,
		)
		throw redirect({
			to: buildOrgPath(params.organizationSlug, to),
			search,
		})
	},
})
