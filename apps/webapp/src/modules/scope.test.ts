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

		expect(resolveSection(crm, '/crm/customers')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/customers/pipeline')?.id).toBe('pipeline')
		expect(resolveSection(crm, '/crm/customers/abc-123')?.id).toBe('customers')
		expect(resolveSection(crm, '/crm/quotes/abc-123')?.id).toBe('quotes')
	})

	it('keeps no section on the module root', () => {
		expect(resolveSection(moduleById('crm'), '/crm')).toBeUndefined()
	})

	it('ignores a section announced but not navigable', () => {
		expect(resolveSection(moduleById('crm'), '/crm/invoices')).toBeUndefined()
	})
})

describe('resolveScope', () => {
	it('resolves the section despite the tenant prefix', () => {
		expect(resolveScope('/o/dupont/crm/customers').label).toBe('Clients')
		expect(resolveScope('/o/dupont/crm/quotes').label).toBe('Devis')
	})

	it('falls back to the module label outside any section', () => {
		expect(resolveScope('/o/dupont/crm').label).toBe('CRM')
	})

	it('exposes no tab until the section declares one', () => {
		expect(resolveScope('/o/dupont/crm/customers').tabs).toEqual([])
	})
})

describe('crossOrganizationPath', () => {
	it('keeps a list screen', () => {
		expect(crossOrganizationPath('/crm/customers')).toBe('/crm/customers')
		expect(crossOrganizationPath('/crm/customers/pipeline')).toBe(
			'/crm/customers/pipeline',
		)
	})

	it('climbs back to the list from an entity screen', () => {
		expect(crossOrganizationPath('/crm/customers/abc-123')).toBe(
			'/crm/customers',
		)
		expect(crossOrganizationPath('/hr/employees/abc-123/work-time')).toBe(
			'/hr/employees',
		)
	})

	it('keeps the root of a module that has an overview', () => {
		expect(crossOrganizationPath('/')).toBe('/')
		expect(crossOrganizationPath('/settings')).toBe('/settings')
	})

	it('falls back to the landing of a module without an overview', () => {
		expect(crossOrganizationPath('/crm')).toBe('/crm/customers')
	})
})
