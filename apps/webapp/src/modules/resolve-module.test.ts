import { describe, expect, it } from 'vitest'
import { resolveModule } from '#/modules/resolve-module'

describe('resolveModule', () => {
	it('returns the home module for the root', () => {
		expect(resolveModule('/').id).toBe('home')
	})

	it('returns the module whose basePath matches', () => {
		expect(resolveModule('/crm').id).toBe('crm')
		expect(resolveModule('/crm/customers').id).toBe('crm')
		expect(resolveModule('/crm/customers/abc-123').id).toBe('crm')
	})

	it('matches on a segment boundary only', () => {
		expect(resolveModule('/crmsomething').id).toBe('home')
	})

	it('attaches parameters to their own module', () => {
		expect(resolveModule('/settings').id).toBe('settings')
	})

	it('falls back to home for an unknown URL', () => {
		expect(resolveModule('/inconnu').id).toBe('home')
	})

	it('resolves customer pages to the crm module', () => {
		expect(resolveModule('/crm/customers').id).toBe('crm')
		expect(resolveModule('/crm/customers/pipeline').id).toBe('crm')
	})

	it('resolves an announced module like any other', () => {
		expect(resolveModule('/chat').id).toBe('chat')
	})

	it('resolves HR pages to the hr module', () => {
		expect(resolveModule('/hr/employees').id).toBe('hr')
	})

	it('resolves planning pages to the planning module', () => {
		expect(resolveModule('/planning').id).toBe('planning')
		expect(resolveModule('/planning/team').id).toBe('planning')
		expect(resolveModule('/planning/tasks').id).toBe('planning')
	})
})
