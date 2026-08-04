import { describe, expect, it } from 'vitest'
import { buildBreadcrumbItems } from '#/modules/breadcrumb'

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
