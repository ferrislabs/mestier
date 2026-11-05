import { describe, expect, it } from 'vitest'
import {
	eventContractSource,
	hasContract,
	payloadContractFields,
	payloadTypeName,
} from '#/pages/automation/lib/event-contracts'

const EVENT_NAMES = [
	'quote.created',
	'quote.updated',
	'quote.sent',
	'quote.accepted',
	'quote.declined',
	'quote.cancelled',
	'quote.deleted',
	'invoice.created',
	'invoice.updated',
	'invoice.issued',
	'invoice.partially_paid',
	'invoice.paid',
	'invoice.cancelled',
	'invoice.deleted',
	'invoice_payment.recorded',
	'invoice_payment.deleted',
	'ticket.created',
	'ticket.transitioned',
	'maintenance_intervention.proposed',
	'maintenance_intervention.validated',
	'maintenance_intervention.rejected',
	'time_entry.started',
	'time_entry.stopped',
	'time_entry.declared',
	'day.ended',
	'assignment_report.reported',
	'assignment_report.applied',
	'assignment_report.dismissed',
	'supplier_invoice.received',
	'supplier_invoice.confirmed',
	'supplier_invoice.rejected',
	'supplier_invoice_line_allocation.recorded',
	'member.invited',
	'member.joined',
	'member.removed',
]

function declaredNames(source: string): Set<string> {
	return new Set(
		[...source.matchAll(/(?:interface|type) (\w+)/g)].map((match) => match[1]),
	)
}

describe('hasContract', () => {
	it('knows every event of the catalogue', () => {
		expect(EVENT_NAMES.filter((name) => !hasContract(name))).toEqual([])
	})

	it('does not know an event outside the catalogue', () => {
		expect(hasContract('quote.archived')).toBe(false)
	})
})

describe('payloadTypeName', () => {
	it('pascal-cases the event name, underscores included', () => {
		expect(payloadTypeName('quote.accepted')).toBe('QuoteAcceptedPayload')
		expect(payloadTypeName('invoice_payment.recorded')).toBe(
			'InvoicePaymentRecordedPayload',
		)
	})
})

describe('eventContractSource', () => {
	it('names the types and keeps status unions as literals', () => {
		const source = eventContractSource('quote.sent')

		expect(source).toContain('export interface QuoteSentPayload {')
		expect(source).toContain('  quote: Quote')
		expect(source).toContain('  from: QuoteStatus')
		expect(source).toContain("  to: 'SENT'")
		expect(source).toContain(
			"type QuoteStatus = 'DRAFT' | 'SENT' | 'ACCEPTED' | 'DECLINED' | 'CANCELLED'",
		)
		expect(source).toContain('  reference: string | null')
		expect(source).toContain('  gross_cents: Cents')
		expect(source).toContain('type Cents = number')
	})

	it('declares Record and array types used by a ticket', () => {
		const source = eventContractSource('ticket.created')

		expect(source).toContain('  custom_fields: Record<string, string>')
		expect(source).toContain('  badges: Badge[]')
		expect(source).toContain('interface Badge {')
	})

	it('declares every type its payload uses, for every event', () => {
		const builtins = new Set(['Partial', 'Record'])
		for (const name of EVENT_NAMES) {
			const source = eventContractSource(name)
			const declared = declaredNames(source)
			const referenced = [...source.matchAll(/:\s*([^\n]+)/g)].flatMap(
				(match) =>
					[...match[1].matchAll(/\b([A-Z]\w*)\b/g)].map((word) => word[1]),
			)
			const undeclared = referenced.filter(
				(word) =>
					!declared.has(word) && !builtins.has(word) && !/^[A-Z_]+$/.test(word),
			)
			expect(undeclared, name).toEqual([])
		}
	})

	it('is empty for an event without a contract', () => {
		expect(eventContractSource('quote.archived')).toBe('')
	})
})

describe('payloadContractFields', () => {
	it('lists the top-level fields of the payload', () => {
		expect(payloadContractFields('quote.updated')).toEqual([
			'quote: Quote',
			'changed_fields: string[]',
			'previous: Partial<Quote>',
		])
		expect(payloadContractFields('quote.archived')).toEqual([])
	})
})
