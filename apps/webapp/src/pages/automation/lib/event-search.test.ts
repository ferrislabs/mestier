import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	firstEvent,
	groupEvents,
	searchEvents,
	typographicEventLabels,
} from '#/pages/automation/lib/event-search'

function event(name: string, label = name): Schemas.EventDescriptorResponse {
	return {
		name,
		label,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: {},
	}
}

const PROTOTYPE_EVENTS: readonly (readonly [string, string])[] = [
	['quote.created', 'Devis'],
	['quote.updated', 'Devis'],
	['quote.sent', 'Devis'],
	['quote.accepted', 'Devis'],
	['quote.declined', 'Devis'],
	['quote.cancelled', 'Devis'],
	['quote.deleted', 'Devis'],
	['invoice.created', 'Factures'],
	['invoice.updated', 'Factures'],
	['invoice.issued', 'Factures'],
	['invoice.partially_paid', 'Factures'],
	['invoice.paid', 'Factures'],
	['invoice.cancelled', 'Factures'],
	['invoice.deleted', 'Factures'],
	['invoice_payment.recorded', 'Paiements'],
	['invoice_payment.deleted', 'Paiements'],
	['ticket.created', 'Support'],
	['ticket.transitioned', 'Support'],
	['maintenance_intervention.proposed', 'Entretien'],
	['maintenance_intervention.validated', 'Entretien'],
	['maintenance_intervention.rejected', 'Entretien'],
	['time_entry.started', 'Temps de travail'],
	['time_entry.stopped', 'Temps de travail'],
	['time_entry.declared', 'Temps de travail'],
	['day.ended', 'Temps de travail'],
	['assignment_report.reported', 'Écarts de planning'],
	['assignment_report.applied', 'Écarts de planning'],
	['assignment_report.dismissed', 'Écarts de planning'],
	['supplier_invoice.received', 'Achats'],
	['supplier_invoice.confirmed', 'Achats'],
	['supplier_invoice.rejected', 'Achats'],
	['supplier_invoice_line_allocation.recorded', 'Achats'],
	['member.invited', 'Équipe'],
	['member.joined', 'Équipe'],
	['member.removed', 'Équipe'],
	['customer.created', 'Clients'],
	['deal.created', 'Opportunités'],
	['deal.stage_changed', 'Opportunités'],
	['absence.requested', 'Absences'],
	['absence.approved', 'Absences'],
	['absence.rejected', 'Absences'],
	['absence.cancelled', 'Absences'],
	['extra_period.requested', 'Heures supplémentaires'],
	['extra_period.approved', 'Heures supplémentaires'],
	['extra_period.rejected', 'Heures supplémentaires'],
	['extra_period.cancelled', 'Heures supplémentaires'],
]

const catalogue = PROTOTYPE_EVENTS.map(([name]) => event(name))

function reversed<T>(items: readonly T[]): T[] {
	return [...items].reverse()
}

describe('groupEvents', () => {
	it('lists the prototype domains in the prototype order', () => {
		expect(groupEvents(catalogue).map((group) => group.label)).toEqual([
			'Devis',
			'Factures',
			'Paiements',
			'Support',
			'Entretien',
			'Temps de travail',
			'Écarts de planning',
			'Achats',
			'Équipe',
			'Clients',
			'Opportunités',
			'Absences',
			'Heures supplémentaires',
		])
	})

	it('files every real event name under its prototype domain', () => {
		const domainOf = new Map(
			groupEvents(catalogue).flatMap((group) =>
				group.events.map((item) => [item.name, group.label] as const),
			),
		)

		for (const [name, label] of PROTOTYPE_EVENTS) {
			expect(domainOf.get(name), name).toBe(label)
		}
	})

	it('keeps the prototype order inside a domain, whatever order the server answers in', () => {
		const forward = groupEvents(catalogue)
		const backward = groupEvents(reversed(catalogue))

		expect(forward[0].events.map((item) => item.name)).toEqual([
			'quote.created',
			'quote.updated',
			'quote.sent',
			'quote.accepted',
			'quote.declined',
			'quote.cancelled',
			'quote.deleted',
		])
		expect(backward).toEqual(forward)
	})

	it('puts an event of a known subject in that domain, after the known ones', () => {
		const groups = groupEvents([
			event('quote.reverted_to_draft', 'Devis repassé en brouillon'),
			event('quote.sent'),
			event('quote.created'),
		])

		expect(groups.map((group) => group.label)).toEqual(['Devis'])
		expect(groups[0].events.map((item) => item.name)).toEqual([
			'quote.created',
			'quote.sent',
			'quote.reverted_to_draft',
		])
	})

	it('gathers the events with no known domain in a last group called Autres', () => {
		const groups = groupEvents([
			event('zeta.happened', 'Zêta'),
			event('member.joined'),
			event('alpha.happened', 'Alpha'),
		])

		expect(groups.map((group) => group.label)).toEqual(['Équipe', 'Autres'])
		expect(groups[1].events.map((item) => item.name)).toEqual([
			'alpha.happened',
			'zeta.happened',
		])
	})

	it('has no group for a domain nothing fills', () => {
		expect(groupEvents([event('member.joined')]).map((g) => g.id)).toEqual([
			'team',
		])
		expect(groupEvents([])).toEqual([])
	})
})

describe('firstEvent', () => {
	it('is the first event of the first group, not the first the server sent', () => {
		expect(firstEvent(reversed(catalogue))?.name).toBe('quote.created')
		expect(
			firstEvent([event('member.joined'), event('invoice.paid')])?.name,
		).toBe('invoice.paid')
	})

	it('is undefined for an empty catalogue', () => {
		expect(firstEvent([])).toBeUndefined()
	})
})

describe('searchEvents', () => {
	it('groups by domain with a French label', () => {
		expect(
			searchEvents(
				[event('quote.sent', 'Devis envoyé'), event('invoice.paid')],
				'',
			).map((group) => group.label),
		).toEqual(['Devis', 'Factures'])
	})

	it('matches the label, the name or the domain, ignoring accents', () => {
		const events = [
			event('quote.sent', 'Devis envoyé'),
			event('invoice.paid', 'Facture payée'),
		]

		expect(searchEvents(events, 'envoye')[0].events[0].name).toBe('quote.sent')
		expect(searchEvents(events, 'invoice.paid')).toHaveLength(1)
		expect(searchEvents(events, 'factures')[0].id).toBe('invoice')
	})

	it('drops the empty domains and returns nothing when nothing matches', () => {
		expect(
			searchEvents(catalogue, 'entretien').map((group) => group.label),
		).toEqual(['Entretien'])
		expect(searchEvents(catalogue, 'zzz')).toEqual([])
	})
})

describe('typographicEventLabels', () => {
	it('swaps the straight apostrophe of a label for the typographic one', () => {
		const [fixed, untouched] = typographicEventLabels([
			event('ticket.transitioned', "Statut d'un ticket de support modifié"),
			event('quote.sent', 'Devis envoyé'),
		])

		expect(fixed.label).toBe('Statut d’un ticket de support modifié')
		expect(untouched.label).toBe('Devis envoyé')
	})

	it('leaves the other fields alone', () => {
		const original = event('ticket.transitioned', "Statut d'un ticket")
		const [fixed] = typographicEventLabels([original])

		expect(fixed).toEqual({ ...original, label: 'Statut d’un ticket' })
	})
})
