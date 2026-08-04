import type { Equipment, ServiceRateUnit } from '#/hooks/use-reference-catalog'

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
