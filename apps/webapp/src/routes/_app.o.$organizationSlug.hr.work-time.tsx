import { createFileRoute } from '@tanstack/react-router'
import { WorkTimeOverviewFeature } from '#/pages/hr/feature/work-time-overview-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/work-time')({
	component: WorkTimeOverviewFeature,
})
