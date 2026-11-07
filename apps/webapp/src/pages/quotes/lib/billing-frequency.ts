import type { Schemas } from '#/api/api.client'

export type BillingFrequency = 'one_shot' | 'monthly' | 'quarterly' | 'yearly'

export const BILLING_FREQUENCIES: readonly BillingFrequency[] = [
	'one_shot',
	'monthly',
	'quarterly',
	'yearly',
]

export const FREQUENCY_LABELS: Record<BillingFrequency, string> = {
	one_shot: 'Ponctuel',
	monthly: 'Mensuel',
	quarterly: 'Trimestriel',
	yearly: 'Annuel',
}

export const FREQUENCY_SUFFIX: Record<BillingFrequency, string> = {
	one_shot: '',
	monthly: '/ mois',
	quarterly: '/ trimestre',
	yearly: '/ an',
}

export function isBillingFrequency(value: unknown): value is BillingFrequency {
	return BILLING_FREQUENCIES.some((frequency) => frequency === value)
}

const OF_API: Record<Schemas.BillingFrequency, BillingFrequency> = {
	ONE_SHOT: 'one_shot',
	MONTHLY: 'monthly',
	QUARTERLY: 'quarterly',
	YEARLY: 'yearly',
}

const TO_API: Record<BillingFrequency, Schemas.BillingFrequency> = {
	one_shot: 'ONE_SHOT',
	monthly: 'MONTHLY',
	quarterly: 'QUARTERLY',
	yearly: 'YEARLY',
}

export function frequencyOfApi(
	frequency: Schemas.BillingFrequency,
): BillingFrequency {
	return OF_API[frequency]
}

export function apiOfFrequency(
	frequency: BillingFrequency,
): Schemas.BillingFrequency {
	return TO_API[frequency]
}
