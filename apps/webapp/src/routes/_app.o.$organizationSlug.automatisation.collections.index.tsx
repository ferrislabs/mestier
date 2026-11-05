import { createFileRoute } from '@tanstack/react-router'
import { CollectionsListFeature } from '#/pages/automation/feature/collections-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/collections/',
)({
	component: CollectionsListFeature,
})
