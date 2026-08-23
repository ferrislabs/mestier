import { createFileRoute } from '@tanstack/react-router'
import { AssignmentReportListFeature } from '#/pages/planning/feature/assignment-report-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/reports',
)({
	component: AssignmentReportListFeature,
})
