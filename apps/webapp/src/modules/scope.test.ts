import { describe, expect, it } from 'vitest'
import { MODULES } from '#/modules/registry'
import {
	crossOrganizationPath,
	resolveScope,
	resolveSection,
} from '#/modules/scope'
import type { AppModule } from '#/modules/types'

function moduleById(id: AppModule['id']): AppModule {
	const module = MODULES.find((module) => module.id === id)
	if (!module) throw new Error(`registre: module ${id} manquant`)
	return module
}

describe('resolveSection', () => {
	it('retient la section la plus profonde qui couvre le chemin', () => {
		const crm = moduleById('crm')

		expect(resolveSection(crm, '/crm/customers')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/customers/pipeline')?.id).toBe('pipeline')
		expect(resolveSection(crm, '/crm/customers/abc-123')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/quotes/abc-123')?.id).toBe('quotes')
	})

	it('ne retient aucune section sur la racine du module', () => {
		expect(resolveSection(moduleById('crm'), '/crm')).toBeUndefined()
	})

	it('ignore une section annoncée mais non navigable', () => {
		expect(resolveSection(moduleById('crm'), '/crm/invoices')).toBeUndefined()
	})
})

describe('resolveScope', () => {
	it('résout la section malgré le préfixe de tenant', () => {
		expect(resolveScope('/o/dupont/crm/customers').label).toBe('Clients')
		expect(resolveScope('/o/dupont/crm/quotes').label).toBe('Devis')
	})

	it('retombe sur le libellé du module hors de toute section', () => {
		expect(resolveScope('/o/dupont/crm').label).toBe('CRM')
	})

	it("n'expose aucun onglet tant que la section n'en déclare pas", () => {
		expect(resolveScope('/o/dupont/crm/customers').tabs).toEqual([])
	})
})

describe('crossOrganizationPath', () => {
	it('conserve un écran de liste', () => {
		expect(crossOrganizationPath('/crm/customers')).toBe('/crm/customers')
		expect(crossOrganizationPath('/crm/customers/pipeline')).toBe(
			'/crm/customers/pipeline',
		)
	})

	it("remonte à la liste depuis la fiche d'une entité", () => {
		expect(crossOrganizationPath('/crm/customers/abc-123')).toBe(
			'/crm/customers',
		)
		expect(crossOrganizationPath('/hr/employees/abc-123/work-time')).toBe(
			'/hr/employees',
		)
	})

	it("garde la racine d'un module qui a une vue d'ensemble", () => {
		expect(crossOrganizationPath('/')).toBe('/')
		expect(crossOrganizationPath('/settings')).toBe('/settings')
	})

	it("retombe sur l'atterrissage d'un module sans vue d'ensemble", () => {
		expect(crossOrganizationPath('/crm')).toBe('/crm/customers')
	})
})
