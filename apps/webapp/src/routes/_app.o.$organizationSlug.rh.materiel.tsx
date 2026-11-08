import { createFileRoute } from '@tanstack/react-router'
import { EquipmentFeature } from '#/pages/hr/feature/equipment-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/rh/materiel')({
	component: EquipmentFeature,
})
