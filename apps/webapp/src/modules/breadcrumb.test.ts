import { describe, expect, it } from 'vitest'
import type { NavItem } from '#/components/nav-main'
import { buildBreadcrumbItems, matchingNavItems } from '#/modules/breadcrumb'

function labelsOf(pathname: string, detailLabel?: string): string[] {
	return buildBreadcrumbItems({
		pathname,
		organizationName: 'Baptiste',
		detailLabel,
	}).map((item) => item.label)
}

describe('buildBreadcrumbItems', () => {
	it("n'affiche que l'organisation à la racine", () => {
		expect(labelsOf('/')).toEqual(['Baptiste'])
	})

	it("ajoute l'entrée de nav correspondante", () => {
		expect(labelsOf('/customers')).toEqual(['Baptiste', 'Clients'])
		expect(labelsOf('/settings')).toEqual(['Baptiste', 'Paramètres'])
	})

	it('empile les entrées de la plus générale à la plus précise', () => {
		expect(labelsOf('/customers/pipeline')).toEqual([
			'Baptiste',
			'Clients',
			'Pipeline',
		])
	})

	it('ajoute le libellé de détail en dernier', () => {
		expect(labelsOf('/customers/abc-123', 'Marie Leroy')).toEqual([
			'Baptiste',
			'Clients',
			'Marie Leroy',
		])
	})

	it('donne une cible de lien à toutes les entrées sauf le détail', () => {
		const items = buildBreadcrumbItems({
			pathname: '/customers/abc-123',
			organizationName: 'Baptiste',
			detailLabel: 'Marie Leroy',
		})
		expect(items[0]?.to).toBe('/')
		expect(items[1]?.to).toBe('/customers')
		expect(items[2]?.to).toBeUndefined()
	})
})

describe('matchingNavItems', () => {
	it("trie du plus court au plus long, quel que soit l'ordre d'entrée", () => {
		const items: NavItem[] = [
			{ title: 'C', to: '/a/b/c' },
			{ title: 'B', to: '/a/b' },
			{ title: 'A', to: '/a' },
		]

		const result = matchingNavItems(items, '/none', '/a/b/c')

		expect(result.map((item) => item.title)).toEqual(['A', 'B', 'C'])
	})

	it('exclut une entrée désactivée même si elle correspondrait sinon', () => {
		const items: NavItem[] = [
			{ title: 'A', to: '/a', disabled: true },
			{ title: 'B', to: '/a/b' },
		]

		const result = matchingNavItems(items, '/none', '/a/b')

		expect(result.map((item) => item.title)).toEqual(['B'])
	})
})
