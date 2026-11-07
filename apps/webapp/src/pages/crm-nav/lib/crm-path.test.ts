import { describe, expect, it } from 'vitest'
import { activeSidebarEntries, ENTRY } from '#/pages/crm-nav/lib/crm-path'

describe('activeSidebarEntries', () => {
	it('lights Clients on the list and on a customer page', () => {
		expect(activeSidebarEntries('/crm/clients', {})).toEqual(
			new Set([ENTRY.clients]),
		)
		expect(activeSidebarEntries('/crm/clients/abc', {})).toEqual(
			new Set([ENTRY.clients]),
		)
	})

	it('lights Pipeline', () => {
		expect(activeSidebarEntries('/crm/pipeline', {})).toEqual(
			new Set([ENTRY.pipeline]),
		)
	})

	it('lights Devis on its sub pages', () => {
		expect(activeSidebarEntries('/crm/devis/nouveau', {})).toEqual(
			new Set([ENTRY.quotes]),
		)
	})

	it('lights Factures and Avoirs', () => {
		expect(activeSidebarEntries('/crm/factures/f1', {})).toEqual(
			new Set([ENTRY.invoices]),
		)
		expect(activeSidebarEntries('/crm/avoirs', {})).toEqual(
			new Set([ENTRY.creditNotes]),
		)
	})

	it('splits Trésorerie by type', () => {
		expect(activeSidebarEntries('/crm/tresorerie', {})).toEqual(
			new Set([ENTRY.ledger]),
		)
		expect(
			activeSidebarEntries('/crm/tresorerie', { type: 'recettes' }),
		).toEqual(new Set([ENTRY.receipts]))
		expect(
			activeSidebarEntries('/crm/tresorerie', { type: 'depenses' }),
		).toEqual(new Set([ENTRY.expenses]))
		expect(activeSidebarEntries('/crm/tresorerie', { type: 'x' })).toEqual(
			new Set([ENTRY.ledger]),
		)
	})

	it('lights Stripe, Catalogue, Fournisseurs and Réglages', () => {
		expect(activeSidebarEntries('/crm/stripe', {})).toEqual(
			new Set([ENTRY.stripe]),
		)
		expect(activeSidebarEntries('/crm/catalogue', {})).toEqual(
			new Set([ENTRY.catalog]),
		)
		expect(activeSidebarEntries('/crm/fournisseurs', {})).toEqual(
			new Set([ENTRY.suppliers]),
		)
		expect(activeSidebarEntries('/crm/reglages', {})).toEqual(
			new Set([ENTRY.settings]),
		)
	})

	it('lights nothing outside the CRM', () => {
		expect(activeSidebarEntries('/planning', {}).size).toBe(0)
		expect(activeSidebarEntries('/crm', {}).size).toBe(0)
	})
})
