import type { LucideIcon } from 'lucide-react'
import type { ComponentType } from 'react'
import type { Equipment, ServiceRateUnit } from '#/hooks/use-reference-catalog'
import type { ModuleId } from '#/modules/types'

export interface SettingsSection {
	id: string
	label: string
	icon: LucideIcon
	moduleId?: ModuleId
	Component: ComponentType
}

export type ReferenceTab = 'equipment'

export interface OrganizationFormValues {
	name: string
	slug: string
}

export interface EquipmentFormValues {
	name: string
	hourlyRate: string
}

export interface ServiceRateFormValues {
	label: string
	unit: ServiceRateUnit
	rate: string
}

export interface ReferenceCatalogData {
	equipment: Equipment[]
}
