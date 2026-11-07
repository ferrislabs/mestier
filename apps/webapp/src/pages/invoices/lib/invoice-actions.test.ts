import { describe, expect, it } from 'vitest'
import {
	type InvoiceActionsInput,
	invoiceActions,
	unavailableNotes,
} from './invoice-actions'

function input(
	overrides: Partial<InvoiceActionsInput> = {},
): InvoiceActionsInput {
	return {
		status: 'ISSUED',
		grossCents: 12000,
		creditedCents: 0,
		remainingCents: 12000,
		sentAt: null,
		...overrides,
	}
}

describe('invoiceActions', () => {
	it('lets a draft be issued, cancelled or deleted and nothing else', () => {
		const actions = invoiceActions(input({ status: 'DRAFT' }))

		expect(actions.issue.enabled).toBe(true)
		expect(actions.cancel.enabled).toBe(true)
		expect(actions.deleteDraft.enabled).toBe(true)
		expect(actions.refund.visible).toBe(false)
		expect(actions.recordPayment.visible).toBe(false)
		expect(actions.creditNote.visible).toBe(false)
	})

	it('offers a payment, a reminder and a credit note on an issued invoice', () => {
		const actions = invoiceActions(input())

		expect(actions.issue.visible).toBe(false)
		expect(actions.recordPayment.enabled).toBe(true)
		expect(actions.remind.enabled).toBe(true)
		expect(actions.creditNote.enabled).toBe(true)
		expect(actions.cancel.enabled).toBe(true)
	})

	it('does not offer to cancel an invoice that already received a payment', () => {
		expect(
			invoiceActions(input({ status: 'PARTIALLY_PAID' })).cancel.visible,
		).toBe(false)
	})

	it('explains why nothing more can be collected on a paid invoice', () => {
		const actions = invoiceActions(input({ status: 'PAID', remainingCents: 0 }))

		expect(actions.recordPayment.enabled).toBe(false)
		expect(actions.recordPayment.reason).toContain('soldée')
		expect(actions.remind.visible).toBe(false)
	})

	it('blocks a credit note once credit notes cover the whole invoice', () => {
		const actions = invoiceActions(
			input({ creditedCents: 12000, remainingCents: 0 }),
		)

		expect(actions.creditNote.enabled).toBe(false)
		expect(actions.creditNote.reason).toContain('Plus rien à créditer')
	})

	it('keeps the payment available while the balance is still loading', () => {
		expect(
			invoiceActions(input({ remainingCents: null, creditedCents: null }))
				.recordPayment.enabled,
		).toBe(true)
	})

	it('offers to send again once it was sent', () => {
		expect(invoiceActions(input()).sendLabel).toBe('Envoyer au client')
		expect(
			invoiceActions(input({ sentAt: '2026-06-01T00:00:00Z' })).sendLabel,
		).toBe('Renvoyer')
	})

	it('blocks the actions of a cancelled invoice', () => {
		const actions = invoiceActions(input({ status: 'CANCELLED' }))

		expect(actions.recordPayment.enabled).toBe(false)
		expect(actions.send.enabled).toBe(false)
	})
})

describe('refund', () => {
	it('offers a refund, not a new payment, when the client paid too much', () => {
		const actions = invoiceActions(input({ remainingCents: -3000 }))

		expect(actions.refund.enabled).toBe(true)
		expect(actions.recordPayment.enabled).toBe(false)
		expect(actions.recordPayment.reason).toContain('Rembourser')
		expect(actions.remind.visible).toBe(false)
	})

	it('hides the refund and the draft deletion on a settled invoice', () => {
		const actions = invoiceActions(input({ status: 'PAID', remainingCents: 0 }))

		expect(actions.refund.visible).toBe(false)
		expect(actions.deleteDraft.visible).toBe(false)
	})
})

describe('unavailableNotes', () => {
	it('lists each visible but disabled action with its reason', () => {
		const notes = unavailableNotes(
			invoiceActions(input({ status: 'PAID', remainingCents: 0 })),
		)

		expect(notes.map((note) => note.key)).toEqual(['recordPayment'])
		expect(notes[0]?.label).toBe('Enregistrer un paiement')
	})
})
