import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'
import { workflowsPath } from '#/pages/automation/lib/automation-paths'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, workflowsPath()),
		})
	},
})
