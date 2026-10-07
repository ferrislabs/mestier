import { createFileRoute } from '@tanstack/react-router'
import { SuppliersFeature } from '#/pages/suppliers/feature/suppliers-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/purchase/suppliers',
)({
	component: SuppliersFeature,
})
