export type VatRate = 0 | 5.5 | 10 | 20

export type PostalAddress = {
	street: string
	complement: string
	postalCode: string
	city: string
	country: string
}

export type Company = {
	name: string
	legalForm: string
	capital: string
	legalId: string
	registry: string
	vatNumber: string
	address: PostalAddress
	email: string
	phone: string
	iban: string
	bic: string
	insurance: string
}

export type SequenceKind = 'quote' | 'invoice' | 'creditNote'

export type Sequences = Record<SequenceKind, Record<string, number>>

export type Settings = {
	company: Company
	paymentTermsDays: number
	quoteValidityDays: number
	defaultVatRate: VatRate
	latePenalty: string
	recoveryFee: number
	invoiceFooter: string
	numberPadding: number
	stripeEndpoint: string
	stripeSecretSet: boolean
}

export type CrmSettingsState = {
	settings: Settings
	sequences: Sequences
}

export type StripeEventType =
	| 'invoice.paid'
	| 'payment_intent.succeeded'
	| 'charge.refunded'

export const VAT_RATES: readonly VatRate[] = [20, 10, 5.5, 0]

export function formatVatRate(rate: VatRate): string {
	return `${String(rate).replace('.', ',')} %`
}
