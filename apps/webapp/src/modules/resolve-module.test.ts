import { describe, expect, it } from 'vitest'
import { hasSideNav, resolveModule } from '#/modules/resolve-module'

describe('resolveModule', () => {
	it('returns the home module for the root', () => {
		expect(resolveModule('/').id).toBe('home')
	})

	it('returns the module whose basePath matches', () => {
		expect(resolveModule('/crm').id).toBe('crm')
		expect(resolveModule('/crm/clients').id).toBe('crm')
		expect(resolveModule('/crm/clients/abc-123').id).toBe('crm')
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
		expect(resolveModule('/crm/clients').id).toBe('crm')
		expect(resolveModule('/crm/pipeline').id).toBe('crm')
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

	it('resolves support pages to the support module', () => {
		expect(resolveModule('/support').id).toBe('support')
		expect(resolveModule('/support/table').id).toBe('support')
	})

	it('resolves planification pages to the planification module', () => {
		expect(resolveModule('/planification').id).toBe('planification')
		expect(resolveModule('/planification/board').id).toBe('planification')
		expect(resolveModule('/planification/maintenance').id).toBe('planification')
	})

	it('resolves projet pages to the projet module', () => {
		expect(resolveModule('/projet').id).toBe('projet')
		expect(resolveModule('/projet/abc-123').id).toBe('projet')
		expect(resolveModule('/projet/abc-123/doe/preview').id).toBe('projet')
		expect(resolveModule('/projet/doe-models/m1').id).toBe('projet')
	})

	/**
	 * #468 put two modules under near-identical prefixes. They diverge at the
	 * fifth character, so `startsWith` never confuses them — asserted rather
	 * than assumed, because renaming either one could break it silently.
	 */
	it('keeps the planning and planification modules apart', () => {
		expect(resolveModule('/planning/calendar').id).toBe('planning')
		expect(resolveModule('/planification/board').id).toBe('planification')
	})
})

describe('hasSideNav', () => {
	it('gives the organization root no side nav', () => {
		expect(hasSideNav(resolveModule('/'), '/')).toBe(false)
	})

	it('keeps the side nav on the other screens of the home module', () => {
		expect(hasSideNav(resolveModule('/reporting'), '/reporting')).toBe(true)
	})

	it('gives the settings no side nav', () => {
		expect(hasSideNav(resolveModule('/settings'), '/settings')).toBe(false)
	})

	it('gives every business module a side nav', () => {
		expect(hasSideNav(resolveModule('/crm'), '/crm')).toBe(true)
		expect(hasSideNav(resolveModule('/projet'), '/projet')).toBe(true)
	})
})
