export type StripeEventType =
	| 'invoice.paid'
	| 'payment_intent.succeeded'
	| 'charge.refunded'

export type StripeEventStatus = 'processed' | 'to_review' | 'ignored'

export type StripeReviewReason =
	| 'unknown_customer'
	| 'no_open_invoice'
	| 'ambiguous_invoice'

export type StripeLine = {
	description: string
	quantity: number
	unitPrice: number
	vatRate: number
}

export type StripeEvent = {
	id: string
	type: StripeEventType
	receivedAt: string
	customerId: string | null
	customerEmail: string
	customerName: string
	amount: number
	description: string
	stripeInvoiceNumber: string | null
	paymentIntentId: string | null
	mestierInvoiceNumber: string | null
	lines: readonly StripeLine[]
	status: StripeEventStatus
	reviewReason: StripeReviewReason | null
	accountId: string | null
	accountName: string
	paymentId: string | null
	outcome: string
	handledById: string | null
}

export type StripeScenario =
	| 'payment_succeeded'
	| 'invoice_paid'
	| 'unknown_customer'

export const STRIPE_TYPE_LABELS: Record<StripeEventType, string> = {
	'invoice.paid': 'Facture payée',
	'payment_intent.succeeded': 'Paiement reçu',
	'charge.refunded': 'Remboursement',
}

export const STRIPE_REASON_LABELS: Record<StripeReviewReason, string> = {
	unknown_customer: 'Client Stripe inconnu',
	no_open_invoice: 'Aucune facture à rapprocher',
	ambiguous_invoice: 'Montant ou facture à confirmer',
}

export const STRIPE_STATUS_LABELS: Record<StripeEventStatus, string> = {
	processed: 'Traité',
	to_review: 'À rapprocher',
	ignored: 'Ignoré',
}
