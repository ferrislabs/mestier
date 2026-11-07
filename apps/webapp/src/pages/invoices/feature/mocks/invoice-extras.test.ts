import { beforeEach, describe, expect, it } from 'vitest'
import {
	hideDraft,
	readInvoiceExtras,
	recordRefund,
	resetInvoiceExtras,
	setIssueDate,
	setQuoteLink,
} from './invoice-extras'

describe('invoice extras mock', () => {
	beforeEach(() => {
		resetInvoiceExtras()
	})

	it('starts empty', () => {
		expect(readInvoiceExtras()).toEqual({
			quoteLinks: {},
			issueDates: {},
			refunds: {},
			hiddenDrafts: [],
		})
	})

	it('keeps what is written and persists it in localStorage', () => {
		setQuoteLink('inv-1', { quoteId: 'q-1', kind: 'deposit', percent: 30 })
		setIssueDate('inv-1', '2026-06-01')
		hideDraft('inv-2')
		recordRefund('inv-1', {
			date: '2026-06-02',
			amountCents: 1000,
			method: 'Virement',
			reference: '',
			note: '',
		})

		const extras = readInvoiceExtras()
		expect(extras.quoteLinks['inv-1']).toEqual({
			quoteId: 'q-1',
			kind: 'deposit',
			percent: 30,
		})
		expect(extras.issueDates['inv-1']).toBe('2026-06-01')
		expect(extras.hiddenDrafts).toEqual(['inv-2'])
		expect(extras.refunds['inv-1']).toHaveLength(1)
		expect(extras.refunds['inv-1']?.[0]?.id).toBeTruthy()
		expect(
			window.localStorage.getItem('mestier.mock.invoice-extras'),
		).toContain('q-1')
	})

	it('does not hide the same draft twice', () => {
		hideDraft('inv-2')
		hideDraft('inv-2')

		expect(readInvoiceExtras().hiddenDrafts).toEqual(['inv-2'])
	})
})
