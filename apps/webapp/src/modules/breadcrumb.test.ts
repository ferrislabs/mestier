import { describe, expect, it } from 'vitest'
import { buildBreadcrumbItems, matchingTargets } from '#/modules/breadcrumb'
import type { NavTarget } from '#/modules/types'

function labelsOf(pathname: string, detailLabel?: string): string[] {
	return buildBreadcrumbItems({
		pathname,
		organizationName: 'Baptiste',
		organizationSlug: 'baptiste',
		detailLabel,
	}).map((item) => item.label)
}

describe('buildBreadcrumbItems', () => {
	it("n'affiche que l'organisation à la racine du tenant", () => {
		expect(labelsOf('/o/baptiste')).toEqual(['Baptiste'])
	})

	it("ajoute l'entrée de nav correspondante", () => {
		expect(labelsOf('/o/baptiste/crm/customers')).toEqual([
			'Baptiste',
			'CRM',
			'Clients',
		])
		expect(labelsOf('/o/baptiste/settings')).toEqual(['Baptiste', 'Paramètres'])
	})

	it('empile les entrées de la plus générale à la plus précise', () => {
		expect(labelsOf('/o/baptiste/crm/customers/pipeline')).toEqual([
			'Baptiste',
			'CRM',
			'Clients',
			'Pipeline',
		])
	})

	it('ajoute le libellé de détail en dernier', () => {
		expect(
			labelsOf('/o/baptiste/crm/customers/abc-123', 'Marie Leroy'),
		).toEqual(['Baptiste', 'CRM', 'Clients', 'Marie Leroy'])
	})

	it('préfixe chaque cible de lien par le tenant, sauf le détail', () => {
		const items = buildBreadcrumbItems({
			pathname: '/o/baptiste/crm/customers/abc-123',
			organizationName: 'Baptiste',
			organizationSlug: 'baptiste',
			detailLabel: 'Marie Leroy',
		})

		expect(items[0]?.to).toBe('/o/baptiste')
		expect(items[1]?.to).toBe('/o/baptiste/crm')
		expect(items[2]?.to).toBe('/o/baptiste/crm/customers')
		expect(items[3]?.to).toBeUndefined()
	})

	it('résout le fil des devis', () => {
		expect(labelsOf('/o/baptiste/crm/quotes')).toEqual([
			'Baptiste',
			'CRM',
			'Devis',
		])
		expect(labelsOf('/o/baptiste/crm/quotes/abc-123', 'Fiche devis')).toEqual([
			'Baptiste',
			'CRM',
			'Devis',
			'Fiche devis',
		])
	})

	it('résout le fil des employés', () => {
		expect(labelsOf('/o/baptiste/hr/employees')).toEqual([
			'Baptiste',
			'RH',
			'Employés',
		])
	})

	it('résout le fil du planning', () => {
		expect(labelsOf('/o/baptiste/planning/team')).toEqual([
			'Baptiste',
			'Planning',
			'Vue équipe',
		])
	})

	it('résout le fil de la liste des tâches', () => {
		expect(labelsOf('/o/baptiste/planning/tasks')).toEqual([
			'Baptiste',
			'Planning',
			'Liste des tâches',
		])
	})
})

describe('matchingTargets', () => {
	it("trie du plus court au plus long, quel que soit l'ordre d'entrée", () => {
		const targets: NavTarget[] = [
			{ id: 'c', label: 'C', to: '/a/b/c' },
			{ id: 'b', label: 'B', to: '/a/b' },
			{ id: 'a', label: 'A', to: '/a' },
		]

		const result = matchingTargets(targets, '/none', '/a/b/c')

		expect(result.map((target) => target.label)).toEqual(['A', 'B', 'C'])
	})

	it('exclut un onglet annoncé même si son chemin correspondrait', () => {
		const targets: NavTarget[] = [
			{ id: 'a', label: 'A', to: '/a', status: 'coming-soon' },
			{ id: 'b', label: 'B', to: '/a/b' },
		]

		const result = matchingTargets(targets, '/none', '/a/b')

		expect(result.map((target) => target.label)).toEqual(['B'])
	})
})
