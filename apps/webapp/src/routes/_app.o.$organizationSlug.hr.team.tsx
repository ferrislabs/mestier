import { createFileRoute } from '@tanstack/react-router'
import { TeamListFeature } from '#/pages/hr/feature/team-list-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/team')({
	component: TeamListFeature,
})
