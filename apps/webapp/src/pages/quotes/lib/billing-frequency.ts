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
