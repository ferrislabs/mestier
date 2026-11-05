import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'
import { workflowPath } from '#/pages/automation/lib/automation-paths'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(
				params.organizationSlug,
				workflowPath(params.workflowId),
			),
		})
	},
})
