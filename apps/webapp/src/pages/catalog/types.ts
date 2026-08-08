import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'

export interface ServiceRateFormValues {
	label: string
	unit: ServiceRateUnit
	rate: string
}
