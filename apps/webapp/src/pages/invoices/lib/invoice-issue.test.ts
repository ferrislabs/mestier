import { describe, expect, it } from 'vitest'
import type { Invoice } from '#/hooks/use-invoices'
import {
	issueDateBounds,
	issueDateErrors,
	lastIssuedDate,
} from './invoice-issue'

const TODAY = '2026-06-10'

function invoice(
	overrides: Partial<Invoice>,
): Pick<Invoice, 'kind' | 'status' | 'issued_on'> {
	return {
		kind: 'STANDARD',
		status: 'ISSUED',
		issued_on: '2026-05-02',
		...overrides,
	}
}

describe('lastIssuedDate', () => {
	it('is the latest issue day of the invoices, credit notes and drafts aside', () => {
		expect(
			lastIssuedDate([
				invoice({ issued_on: '2026-05-02' }),
				invoice({ issued_on: '2026-05-20', status: 'PAID' }),
				invoice({ issued_on: '2026-06-01', kind: 'CREDIT_NOTE' }),
				invoice({ status: 'DRAFT', issued_on: null }),
			]),
		).toBe('2026-05-20')
	})

	it('is null when nothing was issued yet', () => {
		expect(lastIssuedDate([])).toBeNull()
		expect(
			lastIssuedDate([invoice({ status: 'DRAFT', issued_on: null })]),
		).toBeNull()
	})
})

describe('issueDateBounds', () => {
	it('runs from the last issued invoice to today', () => {
		expect(issueDateBounds([invoice({})], TODAY)).toEqual({
			min: '2026-05-02',
			max: TODAY,
		})
		expect(issueDateBounds([], TODAY)).toEqual({ min: null, max: TODAY })
	})
})

describe('issueDateErrors', () => {
	it('accepts today and the last issue day', () => {
		expect(issueDateErrors(TODAY, TODAY, '2026-05-02')).toEqual([])
		expect(issueDateErrors('2026-05-02', TODAY, '2026-05-02')).toEqual([])
	})

	it('rejects an unreadable date', () => {
		expect(issueDateErrors('', TODAY, null)).toEqual([
			'La date d’émission n’est pas valide.',
		])
		expect(issueDateErrors('2026-13-40', TODAY, null)).toEqual([
			'La date d’émission n’est pas valide.',
		])
	})

	it('rejects a future date', () => {
		expect(issueDateErrors('2026-12-01', TODAY, null)[0]).toContain('futur')
	})

	it('rejects a date before the last issued invoice', () => {
		const [message] = issueDateErrors('2026-01-01', TODAY, '2026-05-02')
		expect(message).toContain('précéder')
		expect(message).toContain('chronologique')
	})
})
