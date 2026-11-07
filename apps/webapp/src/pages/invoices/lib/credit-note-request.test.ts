import { describe, expect, it } from 'vitest'
import type { InvoiceLine } from '#/hooks/use-invoices'
import { lineFixture } from '#/pages/invoices/test-support/invoice-fixture'
import {
	creditConsequence,
	creditConsequenceText,
	creditLineIssue,
	creditNoteErrors,
	creditNoteRequest,
	creditPreviewNetCents,
	creditPreviewTotals,
	fillAll,
} from './credit-note-request'

function line(overrides: Partial<InvoiceLine> = {}): InvoiceLine {
	return lineFixture({
		label: 'Pose de parquet',
		quantity: '4',
		unit_price_cents: 4500,
		vat_rate_basis_points: 2000,
		...overrides,
	})
}

describe('creditNoteRequest', () => {
	it('asks the server to credit what remains for a total cancellation', () => {
		expect(creditNoteRequest([line()], 'total', {})).toEqual({
			full: true,
			lines: [],
		})
	})

	it('only credits the quantities typed for a partial correction, by source line', () => {
		const lines = [line(), line({ id: 'line-2', label: 'Plinthes' })]

		expect(creditNoteRequest(lines, 'partial', { 'line-2': '1,5' })).toEqual({
			full: false,
			lines: [{ source_line_id: 'line-2', quantity: '1.5' }],
		})
	})
})

describe('creditPreviewNetCents', () => {
	it('prices the credited quantities at the invoiced unit price', () => {
		expect(creditPreviewNetCents([line()], 'partial', { 'line-1': '2' })).toBe(
			9000,
		)
		expect(creditPreviewNetCents([line()], 'total', {})).toBe(18000)
		expect(creditPreviewNetCents([line()], 'partial', {})).toBe(0)
	})
})

describe('creditLineIssue', () => {
	it('flags an unreadable quantity and one above what was invoiced', () => {
		expect(creditLineIssue(line(), 'partial', { 'line-1': 'abc' })).toBe(
			'Quantité illisible.',
		)
		expect(creditLineIssue(line(), 'partial', { 'line-1': '5' })).toBe(
			'La quantité dépasse celle de la facture.',
		)
		expect(creditLineIssue(line(), 'partial', { 'line-1': '4' })).toBeNull()
		expect(creditLineIssue(line(), 'partial', {})).toBeNull()
	})
})

describe('creditNoteErrors', () => {
	it('needs a reason and at least one credited line', () => {
		expect(creditNoteErrors([line()], 'partial', {}, '')).toEqual([
			'Le motif de l’avoir est obligatoire.',
			'Indique une quantité à créditer sur au moins une ligne.',
		])
	})

	it('accepts a reason with a credited line', () => {
		expect(
			creditNoteErrors([line()], 'partial', { 'line-1': '1' }, 'Erreur de TVA'),
		).toEqual([])
	})

	it('accepts a total cancellation with a reason alone', () => {
		expect(creditNoteErrors([line()], 'total', {}, 'Annulation')).toEqual([])
	})
})

describe('fillAll', () => {
	it('selects the whole quantity of each line', () => {
		expect(fillAll([line()])).toEqual({ 'line-1': '4' })
	})
})

describe('creditPreviewTotals', () => {
	it('adds the VAT of the credited lines to their net total', () => {
		expect(creditPreviewTotals([line()], 'total', {})).toEqual({
			netCents: 18000,
			vatCents: 3600,
			grossCents: 21600,
		})
		expect(creditPreviewTotals([line()], 'partial', { 'line-1': '1' })).toEqual(
			{
				netCents: 4500,
				vatCents: 900,
				grossCents: 5400,
			},
		)
	})

	it('has no VAT on a line without a rate', () => {
		expect(
			creditPreviewTotals([line({ vat_rate_basis_points: null })], 'total', {}),
		).toEqual({ netCents: 18000, vatCents: 0, grossCents: 18000 })
	})
})

describe('creditConsequence', () => {
	it('says nothing when no credit is chosen', () => {
		expect(creditConsequence(12000, 0)).toEqual({ kind: 'none', after: 12000 })
		expect(creditConsequenceText(12000, 0)).toBeNull()
	})

	it('reduces the balance, settles it, or turns it into a refund', () => {
		expect(creditConsequence(12000, 5000)).toEqual({
			kind: 'reduces',
			after: 7000,
		})
		expect(creditConsequence(12000, 12000)).toEqual({
			kind: 'settles',
			after: 0,
		})
		expect(creditConsequence(12000, 15000)).toEqual({
			kind: 'refund',
			after: -3000,
		})
	})

	it('words each outcome', () => {
		expect(creditConsequenceText(12000, 12000)).toContain(
			'plus rien à encaisser',
		)
		expect(creditConsequenceText(12000, 5000)).toContain('Le reste dû passera')
		expect(creditConsequenceText(12000, 15000)).toContain('à lui rembourser')
	})
})
