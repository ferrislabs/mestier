import { describe, expect, it } from 'vitest'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'

describe('buildOrgPath', () => {
	it('préfixe un chemin de module par le tenant', () => {
		expect(buildOrgPath('dupont', '/crm/customers')).toBe(
			'/o/dupont/crm/customers',
		)
	})

	it("ne laisse pas de barre finale sur la racine de l'organisation", () => {
		expect(buildOrgPath('dupont', '/')).toBe('/o/dupont')
	})

	it('conserve les segments paramétrés du routeur', () => {
		expect(buildOrgPath('dupont', '/crm/quotes/$quoteId')).toBe(
			'/o/dupont/crm/quotes/$quoteId',
		)
	})
})

describe('splitOrgPath', () => {
	it('sépare le tenant du chemin de module', () => {
		expect(splitOrgPath('/o/dupont/crm/customers')).toEqual({
			organizationSlug: 'dupont',
			path: '/crm/customers',
		})
	})

	it("ramène la racine du tenant à '/'", () => {
		expect(splitOrgPath('/o/dupont')).toEqual({
			organizationSlug: 'dupont',
			path: '/',
		})
	})

	it('décode un slug encodé', () => {
		expect(splitOrgPath('/o/mon%20org/hr').organizationSlug).toBe('mon org')
	})

	it('laisse intact un chemin hors organisation', () => {
		expect(splitOrgPath('/invite/abc')).toEqual({
			organizationSlug: null,
			path: '/invite/abc',
		})
	})

	it('ne confond pas un préfixe qui commence par les mêmes lettres', () => {
		expect(splitOrgPath('/onboarding')).toEqual({
			organizationSlug: null,
			path: '/onboarding',
		})
	})
})
