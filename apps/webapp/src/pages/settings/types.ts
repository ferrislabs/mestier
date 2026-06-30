import type { ProductCatalogFormValues } from '#/hooks/use-catalog-items'
import type {
	Employee,
	Equipment,
	Product,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'

export type ReferenceTab =
	| 'employees'
	| 'equipment'
	| 'service-rates'
	| 'products'

export interface OrganizationFormValues {
	name: string
	slug: string
}

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
	vatRate: string
}

export interface ReferenceCatalogData {
	employees: Employee[]
	equipment: Equipment[]
	serviceRates: ServiceRate[]
	products: Product[]
}

export type { ProductCatalogFormValues, Product }
