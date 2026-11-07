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
	if (!module) throw new Error(`registry: module ${id} missing`)
	return module
}

describe('resolveSection', () => {
	it('keeps the deepest section covering the path', () => {
		const crm = moduleById('crm')

		expect(resolveSection(crm, '/crm/clients')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/pipeline')?.id).toBe('pipeline')
		expect(resolveSection(crm, '/crm/clients/abc-123')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/devis/abc-123')?.id).toBe('quotes')
	})

	it('keeps no section on the module root', () => {
		expect(resolveSection(moduleById('crm'), '/crm')).toBeUndefined()
	})

	it('resolves a section once it is navigable', () => {
		expect(resolveSection(moduleById('crm'), '/crm/factures')?.id).toBe(
			'invoices',
		)
	})
})

describe('resolveScope', () => {
	it('resolves the section despite the tenant prefix', () => {
		expect(resolveScope('/o/dupont/crm/clients').label).toBe('Clients')
		expect(resolveScope('/o/dupont/crm/devis').label).toBe('Devis')
	})

	it('falls back to the module label outside any section', () => {
		expect(resolveScope('/o/dupont/crm').label).toBe('CRM')
	})

	it('exposes no tab until the section declares one', () => {
		expect(resolveScope('/o/dupont/crm/clients').tabs).toEqual([])
	})
})

describe('crossOrganizationPath', () => {
	it('keeps a list screen', () => {
		expect(crossOrganizationPath('/crm/clients')).toBe('/crm/clients')
		expect(crossOrganizationPath('/crm/pipeline')).toBe('/crm/pipeline')
	})

	it('climbs back to the list from an entity screen', () => {
		expect(crossOrganizationPath('/crm/clients/abc-123')).toBe('/crm/clients')
		expect(crossOrganizationPath('/hr/team/abc-123/work-time')).toBe('/hr/team')
	})

	it('keeps the root of a module that has an overview', () => {
		expect(crossOrganizationPath('/')).toBe('/')
		expect(crossOrganizationPath('/settings')).toBe('/settings')
	})

	it('falls back to the landing of a module without an overview', () => {
		expect(crossOrganizationPath('/crm')).toBe('/crm/clients')
	})
})
