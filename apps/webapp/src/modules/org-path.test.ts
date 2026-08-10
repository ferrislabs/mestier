import { describe, expect, it } from 'vitest'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'

describe('buildOrgPath', () => {
	it('prefixes a module path with the tenant', () => {
		expect(buildOrgPath('dupont', '/crm/customers')).toBe(
			'/o/dupont/crm/customers',
		)
	})

	it('leaves no trailing slash on the organization root', () => {
		expect(buildOrgPath('dupont', '/')).toBe('/o/dupont')
	})

	it('keeps the router parameter segments', () => {
		expect(buildOrgPath('dupont', '/crm/quotes/$quoteId')).toBe(
			'/o/dupont/crm/quotes/$quoteId',
		)
	})
})

describe('splitOrgPath', () => {
	it('splits the tenant off the module path', () => {
		expect(splitOrgPath('/o/dupont/crm/customers')).toEqual({
			organizationSlug: 'dupont',
			path: '/crm/customers',
		})
	})

	it("brings the tenant root back to '/'", () => {
		expect(splitOrgPath('/o/dupont')).toEqual({
			organizationSlug: 'dupont',
			path: '/',
		})
	})

	it('decodes an encoded slug', () => {
		expect(splitOrgPath('/o/mon%20org/hr').organizationSlug).toBe('mon org')
	})

	it('leaves a path outside any organization untouched', () => {
		expect(splitOrgPath('/invite/abc')).toEqual({
			organizationSlug: null,
			path: '/invite/abc',
		})
	})

	it('does not mistake a prefix that starts with the same letters', () => {
		expect(splitOrgPath('/onboarding')).toEqual({
			organizationSlug: null,
			path: '/onboarding',
		})
	})
})
