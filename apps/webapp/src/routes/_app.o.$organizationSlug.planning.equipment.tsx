import { createFileRoute } from '@tanstack/react-router'
import { EquipmentFeature } from '#/pages/planning/feature/equipment-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/equipment',
)({
	component: EquipmentFeature,
})
