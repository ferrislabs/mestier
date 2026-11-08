import type { Schemas } from '#/api/api.client'
import { normalizeText } from '#/pages/automation/lib/text'

type EventDescriptor = Schemas.EventDescriptorResponse

interface EventDomain {
	id: string
	label: string
	events: readonly string[]
}

const EVENT_DOMAINS: readonly EventDomain[] = [
	{
		id: 'quote',
		label: 'Devis',
		events: [
			'quote.created',
			'quote.updated',
			'quote.sent',
			'quote.accepted',
			'quote.declined',
			'quote.cancelled',
			'quote.deleted',
		],
	},
	{
		id: 'invoice',
		label: 'Factures',
		events: [
			'invoice.created',
			'invoice.updated',
			'invoice.issued',
			'invoice.partially_paid',
			'invoice.paid',
			'invoice.cancelled',
			'invoice.deleted',
		],
	},
	{
		id: 'payment',
		label: 'Paiements',
		events: ['invoice_payment.recorded', 'invoice_payment.deleted'],
	},
	{
		id: 'support',
		label: 'Support',
		events: ['ticket.created', 'ticket.transitioned'],
	},
	{
		id: 'maintenance',
		label: 'Entretien',
		events: [
			'maintenance_intervention.proposed',
			'maintenance_intervention.validated',
			'maintenance_intervention.rejected',
		],
	},
	{
		id: 'time',
		label: 'Temps de travail',
		events: [
			'time_entry.started',
			'time_entry.stopped',
			'time_entry.declared',
			'day.ended',
		],
	},
	{
		id: 'planning',
		label: 'Écarts de planning',
		events: [
			'assignment_report.reported',
			'assignment_report.applied',
			'assignment_report.dismissed',
		],
	},
	{
		id: 'purchase',
		label: 'Achats',
		events: [
			'supplier_invoice.received',
			'supplier_invoice.confirmed',
			'supplier_invoice.rejected',
			'supplier_invoice_line_allocation.recorded',
		],
	},
	{
		id: 'team',
		label: 'Équipe',
		events: ['member.invited', 'member.joined', 'member.removed'],
	},
	{
		id: 'customer',
		label: 'Clients',
		events: ['customer.created'],
	},
	{
		id: 'deal',
		label: 'Opportunités',
		events: ['deal.created', 'deal.stage_changed'],
	},
]

const OTHER_DOMAIN = { id: 'other', label: 'Autres' }

function subjectOf(name: string): string {
	return name.split('.')[0] ?? name
}

const DOMAIN_OF_NAME = new Map<string, number>()
const RANK_OF_NAME = new Map<string, number>()
const DOMAIN_OF_SUBJECT = new Map<string, number>()

EVENT_DOMAINS.forEach((domain, domainIndex) => {
	domain.events.forEach((name, rank) => {
		DOMAIN_OF_NAME.set(name, domainIndex)
		RANK_OF_NAME.set(name, rank)
		DOMAIN_OF_SUBJECT.set(subjectOf(name), domainIndex)
	})
})

const UNRANKED = Number.MAX_SAFE_INTEGER

function domainIndexOf(event: EventDescriptor): number {
	return (
		DOMAIN_OF_NAME.get(event.name) ??
		DOMAIN_OF_SUBJECT.get(subjectOf(event.name)) ??
		EVENT_DOMAINS.length
	)
}

function byPrototypeOrder(a: EventDescriptor, b: EventDescriptor): number {
	return (
		(RANK_OF_NAME.get(a.name) ?? UNRANKED) -
			(RANK_OF_NAME.get(b.name) ?? UNRANKED) ||
		a.label.localeCompare(b.label, 'fr') ||
		a.name.localeCompare(b.name)
	)
}

export interface EventSearchGroup {
	id: string
	label: string
	events: readonly EventDescriptor[]
}

export function groupEvents(
	events: readonly EventDescriptor[],
): EventSearchGroup[] {
	const buckets = new Map<number, EventDescriptor[]>()
	for (const event of events) {
		const index = domainIndexOf(event)
		buckets.set(index, [...(buckets.get(index) ?? []), event])
	}
	return [...buckets.entries()]
		.sort(([a], [b]) => a - b)
		.map(([index, members]) => {
			const domain = EVENT_DOMAINS[index] ?? OTHER_DOMAIN
			return {
				id: domain.id,
				label: domain.label,
				events: members.sort(byPrototypeOrder),
			}
		})
}

export function firstEvent(
	events: readonly EventDescriptor[],
): EventDescriptor | undefined {
	return groupEvents(events)[0]?.events[0]
}

export function searchEvents(
	events: readonly EventDescriptor[],
	query: string,
): EventSearchGroup[] {
	const needle = normalizeText(query)
	return groupEvents(events)
		.map((group) => ({
			...group,
			events: group.events.filter(
				(event) =>
					needle === '' ||
					normalizeText(`${event.label} ${event.name} ${group.label}`).includes(
						needle,
					),
			),
		}))
		.filter((group) => group.events.length > 0)
}

export function typographicEventLabels(
	events: readonly EventDescriptor[],
): EventDescriptor[] {
	return events.map((event) =>
		event.label.includes("'")
			? { ...event, label: event.label.replaceAll("'", '’') }
			: event,
	)
}
