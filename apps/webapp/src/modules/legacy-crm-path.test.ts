import { describe, expect, it } from 'vitest'
import { legacyCrmPath } from '#/modules/legacy-crm-path'

describe('legacyCrmPath', () => {
	it.each([
		['customers', '/crm/clients'],
		['customers/abc', '/crm/clients/abc'],
		['customers/pipeline', '/crm/pipeline'],
		['quotes', '/crm/devis'],
		['quotes/new', '/crm/devis/nouveau'],
		['quotes/q1', '/crm/devis/q1'],
		['quotes/q1/handover', '/crm/devis/q1/passation'],
		['invoices', '/crm/factures'],
		['invoices/new', '/crm/factures/nouvelle'],
		['invoices/i1', '/crm/factures/i1'],
		['credit-notes', '/crm/avoirs'],
		['credit-notes/c1', '/crm/avoirs/c1'],
		['treasury', '/crm/tresorerie'],
		['catalog', '/crm/catalogue'],
		['settings', '/crm/reglages'],
	])('maps %s to %s', (legacy, current) => {
		expect(legacyCrmPath(legacy)).toBe(current)
	})

	it('matches whole segments only', () => {
		expect(legacyCrmPath('customersx')).toBeNull()
		expect(legacyCrmPath('quotes-archive')).toBeNull()
	})

	it('leaves the current URLs alone', () => {
		expect(legacyCrmPath('clients')).toBeNull()
		expect(legacyCrmPath('stripe')).toBeNull()
		expect(legacyCrmPath('')).toBeNull()
	})
})
