import type { LucideIcon } from 'lucide-react'
import type { ComponentType } from 'react'
import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'
import type { ModuleId } from '#/modules/types'

export interface SettingsSection {
	id: string
	label: string
	icon: LucideIcon
	moduleId?: ModuleId
	Component: ComponentType
}

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
