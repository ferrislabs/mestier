import { describe, expect, it } from 'vitest'
import type { InvoiceLine } from '#/hooks/use-invoices'
import {
	creditLineIssue,
	creditNoteErrors,
	creditNoteLines,
	creditPreviewNetCents,
	fillAll,
} from './credit-note-request'

function line(overrides: Partial<InvoiceLine> = {}): InvoiceLine {
	return {
		id: 'line-1',
		invoice_id: 'invoice-1',
		organization_id: 'org-1',
		label: 'Pose de parquet',
		position: 0,
		quantity: '4',
		unit_price_cents: 4500,
		vat_rate_basis_points: 2000,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

describe('creditNoteLines', () => {
	it('credits every line in full for a total cancellation', () => {
		expect(creditNoteLines([line()], 'total', {})).toEqual([
			{
				label: 'Pose de parquet',
				quantity: '4',
				unit_price_cents: 4500,
				vat_rate_basis_points: 2000,
			},
		])
	})

	it('only credits the quantities typed for a partial correction', () => {
		const lines = [line(), line({ id: 'line-2', label: 'Plinthes' })]

		const request = creditNoteLines(lines, 'partial', { 'line-2': '1,5' })

		expect(request).toHaveLength(1)
		expect(request[0]).toMatchObject({ label: 'Plinthes', quantity: '1.5' })
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
