import { createFileRoute } from '@tanstack/react-router'
import { ActivityFeature } from '#/pages/chat/feature/activity-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/discussions/activite',
)({
	component: ActivityFeature,
})
