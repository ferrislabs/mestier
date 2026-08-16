import { createFileRoute } from '@tanstack/react-router'
import { AbsencesOverviewFeature } from '#/pages/hr/feature/absences-overview-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/absences')({
	component: AbsencesOverviewFeature,
})
