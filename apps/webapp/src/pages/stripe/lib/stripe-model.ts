import { addDays } from '#/pages/customers/lib/format'
import {
	STRIPE_REASON_LABELS,
	type StripeEvent,
	type StripeEventStatus,
	type StripeEventType,
	type StripeScenario,
} from '#/pages/stripe/types'

export type StripeSearch = {
	statut?: StripeEventStatus
	type?: StripeEventType
	q?: string
}

export type StripeSearchPatch = {
	[Key in keyof StripeSearch]?: StripeSearch[Key] | null
}

const STATUSES: readonly StripeEventStatus[] = [
	'processed',
	'to_review',
	'ignored',
]

const TYPES: readonly StripeEventType[] = [
	'invoice.paid',
	'payment_intent.succeeded',
	'charge.refunded',
]

export const STRIPE_STATS_WINDOW_DAYS = 30

function isStatus(value: unknown): value is StripeEventStatus {
	return STATUSES.some((status) => status === value)
}

function isType(value: unknown): value is StripeEventType {
	return TYPES.some((type) => type === value)
}

function textOf(value: unknown): string | null {
	if (typeof value === 'number') return String(value)
	if (typeof value !== 'string') return null
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function parseStripeSearch(raw: Record<string, unknown>): StripeSearch {
	const query = textOf(raw.q)
	return {
		...(isStatus(raw.statut) ? { statut: raw.statut } : {}),
		...(isType(raw.type) ? { type: raw.type } : {}),
		...(query !== null ? { q: query } : {}),
	}
}

export function cleanStripeSearch(search: StripeSearchPatch): StripeSearch {
	return {
		...(search.statut ? { statut: search.statut } : {}),
		...(search.type ? { type: search.type } : {}),
		...(search.q ? { q: search.q } : {}),
	}
}

export function updateStripeSearch(
	current: StripeSearch,
	patch: StripeSearchPatch,
): StripeSearch {
	return cleanStripeSearch({ ...current, ...patch })
}

export function hasStripeFilters(search: StripeSearch): boolean {
	return Boolean(search.statut || search.type || search.q)
}

export function filterStripeEvents(
	events: readonly StripeEvent[],
	search: StripeSearch,
): StripeEvent[] {
	const query = (search.q ?? '').trim().toLowerCase()
	return events
		.filter((event) => {
			if (search.statut && event.status !== search.statut) return false
			if (search.type && event.type !== search.type) return false
			if (query === '') return true
			const haystack = [
				event.id,
				event.customerName,
				event.customerEmail,
				event.customerId ?? '',
				event.description,
				event.paymentIntentId ?? '',
				event.stripeInvoiceNumber ?? '',
				event.mestierInvoiceNumber ?? '',
				event.outcome,
				event.accountName,
			]
				.join(' ')
				.toLowerCase()
			return haystack.includes(query)
		})
		.sort(
			(a, b) =>
				b.receivedAt.localeCompare(a.receivedAt) || a.id.localeCompare(b.id),
		)
}

export function stripeToReview(events: readonly StripeEvent[]): StripeEvent[] {
	return events
		.filter((event) => event.status === 'to_review')
		.sort((a, b) => b.receivedAt.localeCompare(a.receivedAt))
}

export type StripeStats = {
	received: number
	toReview: number
	automatic: number
	ignored: number
}

export function stripeStats(
	events: readonly StripeEvent[],
	today: string,
): StripeStats {
	const since = addDays(today, -STRIPE_STATS_WINDOW_DAYS)
	const recent = events.filter(
		(event) => event.receivedAt.slice(0, 10) >= since,
	)
	return {
		received: recent.length,
		toReview: events.filter((event) => event.status === 'to_review').length,
		automatic: recent.filter(
			(event) =>
				event.status === 'processed' && event.handledById === 'webhook',
		).length,
		ignored: recent.filter((event) => event.status === 'ignored').length,
	}
}

export type ReviewAction = 'link' | 'allocate' | 'none'

export function reviewActionOf(event: StripeEvent): ReviewAction {
	if (event.reviewReason === 'unknown_customer') return 'link'
	return event.paymentId !== null ? 'allocate' : 'none'
}

export function linkNotice(accountName: string): string {
	return `Client Stripe rattaché à ${accountName}, mais le paiement reste à rapprocher : ${STRIPE_REASON_LABELS.no_open_invoice}.`
}

export function allocateNotice(event: StripeEvent | undefined): string {
	if (event?.status === 'processed') return `${event.outcome}.`
	return 'Le paiement n’a pas pu être affecté à cette facture.'
}

export type SimulationScenario = {
	id: StripeScenario
	label: string
	hint: string
}

export const SIMULATION_SCENARIOS: readonly SimulationScenario[] = [
	{
		id: 'payment_succeeded',
		label: 'Paiement d’une facture ouverte',
		hint: 'Le paiement est affecté seul à la facture du client.',
	},
	{
		id: 'invoice_paid',
		label: 'Facture Stripe payée',
		hint: 'La facture est importée et rattachée au client.',
	},
	{
		id: 'unknown_customer',
		label: 'Client Stripe inconnu',
		hint: 'Le paiement attend d’être rattaché à un client.',
	},
]

const UNAVAILABLE: Record<StripeScenario, string> = {
	payment_succeeded:
		'Le scénario n’a pas pu être joué : aucun client relié à Stripe n’a de facture à régler.',
	invoice_paid:
		'Le scénario n’a pas pu être joué : aucun client n’est relié à Stripe.',
	unknown_customer: 'Le scénario n’a pas pu être joué.',
}

export function simulationNotice(
	scenario: StripeScenario,
	event: StripeEvent | null,
): string {
	if (!event) return UNAVAILABLE[scenario]
	if (event.status === 'processed')
		return `Webhook simulé, traité automatiquement : ${event.outcome}.`
	if (event.reviewReason !== null)
		return `Webhook simulé, à rapprocher : ${STRIPE_REASON_LABELS[event.reviewReason]}.`
	return 'Webhook simulé : l’événement est à rapprocher.'
}
