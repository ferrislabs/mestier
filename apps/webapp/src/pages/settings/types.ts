import type {
	Employee,
	Equipment,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'

export type ReferenceTab = 'employees' | 'equipment' | 'service-rates'

export interface EmployeeFormValues {
	name: string
	hourlyRate: string
	userId: string
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
	employees: Employee[]
	equipment: Equipment[]
	serviceRates: ServiceRate[]
}
