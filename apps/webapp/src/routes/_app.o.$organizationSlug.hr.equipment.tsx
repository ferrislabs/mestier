import { createFileRoute } from '@tanstack/react-router'
import { EquipmentFeature } from '#/pages/hr/feature/equipment-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/equipment')({
	component: EquipmentFeature,
})
