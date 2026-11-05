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
	it('shows only the organization at the tenant root', () => {
		expect(labelsOf('/o/baptiste')).toEqual(['Baptiste'])
	})

	it('adds the matching nav entry', () => {
		expect(labelsOf('/o/baptiste/crm/customers')).toEqual([
			'Baptiste',
			'CRM',
			'Comptes',
		])
		expect(labelsOf('/o/baptiste/settings')).toEqual(['Baptiste', 'Paramètres'])
	})

	it('stacks entries from the most general to the most precise', () => {
		expect(labelsOf('/o/baptiste/crm/customers/pipeline')).toEqual([
			'Baptiste',
			'CRM',
			'Comptes',
			'Pipeline',
		])
	})

	it('adds the detail label last', () => {
		expect(
			labelsOf('/o/baptiste/crm/customers/abc-123', 'Marie Leroy'),
		).toEqual(['Baptiste', 'CRM', 'Comptes', 'Marie Leroy'])
	})

	it('prefixes every link target with the tenant, except the detail', () => {
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

	it('resolves the quotes trail', () => {
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

	it('resolves the team trail', () => {
		expect(labelsOf('/o/baptiste/hr/team')).toEqual([
			'Baptiste',
			'RH',
			'Équipe',
		])
	})

	it('resolves the planning trail', () => {
		expect(labelsOf('/o/baptiste/planning/team')).toEqual([
			'Baptiste',
			'Planning',
			'Vue équipe',
		])
	})

	/**
	 * #468 took the task list out of the navigation and WS5 (#466) deleted it
	 * outright, board in its place. The path is dead, but a stale bookmark
	 * still resolves through this trail, so it has to stop at the module
	 * rather than naming a section that no longer exists.
	 */
	it('stops at the module for the retired task list path', () => {
		expect(labelsOf('/o/baptiste/planning/tasks')).toEqual([
			'Baptiste',
			'Planning',
		])
	})

	it('resolves the projet trails', () => {
		expect(labelsOf('/o/baptiste/projet')).toEqual(['Baptiste', 'Projet'])
		expect(labelsOf('/o/baptiste/projet/new', 'Nouveau projet')).toEqual([
			'Baptiste',
			'Projet',
			'Nouveau projet',
		])
		expect(labelsOf('/o/baptiste/projet/doe-models')).toEqual([
			'Baptiste',
			'Projet',
			'Modèles de DOE',
		])
		expect(labelsOf('/o/baptiste/projet/doe-models/m1')).toEqual([
			'Baptiste',
			'Projet',
			'Modèles de DOE',
		])
	})

	it('resolves the planification trails', () => {
		expect(labelsOf('/o/baptiste/planification/board')).toEqual([
			'Baptiste',
			'Planification',
			'Tableau',
		])
	})
})

describe('buildBreadcrumbItems for the automatisation module', () => {
	it('shows Workflows once on the workflow list', () => {
		expect(labelsOf('/o/baptiste/automatisation/workflows')).toEqual([
			'Baptiste',
			'Workflows',
		])
	})

	it('shows Workflows once in the editor', () => {
		expect(labelsOf('/o/baptiste/automatisation/workflows/wf-1')).toEqual([
			'Baptiste',
			'Workflows',
		])
		expect(labelsOf('/o/baptiste/automatisation/workflows/nouveau')).toEqual([
			'Baptiste',
			'Workflows',
		])
	})

	it('shows Workflows once on a project page', () => {
		expect(labelsOf('/o/baptiste/automatisation/projets/ventes')).toEqual([
			'Baptiste',
			'Workflows',
		])
	})

	it('names the other sections after the module', () => {
		expect(labelsOf('/o/baptiste/automatisation/executions')).toEqual([
			'Baptiste',
			'Workflows',
			'Exécutions',
		])
		expect(labelsOf('/o/baptiste/automatisation/collections/col-1')).toEqual([
			'Baptiste',
			'Workflows',
			'Collections',
		])
		expect(labelsOf('/o/baptiste/automatisation/identifications')).toEqual([
			'Baptiste',
			'Workflows',
			'Identifications',
		])
	})

	it('links the module crumb to its base path', () => {
		const items = buildBreadcrumbItems({
			pathname: '/o/baptiste/automatisation/workflows',
			organizationName: 'Baptiste',
			organizationSlug: 'baptiste',
		})

		expect(items.map((item) => item.to)).toEqual([
			'/o/baptiste',
			'/o/baptiste/automatisation',
		])
	})
})

describe('matchingTargets', () => {
	it('sorts shortest to longest, whatever the input order', () => {
		const targets: NavTarget[] = [
			{ id: 'c', label: 'C', to: '/a/b/c' },
			{ id: 'b', label: 'B', to: '/a/b' },
			{ id: 'a', label: 'A', to: '/a' },
		]

		const result = matchingTargets(targets, '/none', '/a/b/c')

		expect(result.map((target) => target.label)).toEqual(['A', 'B', 'C'])
	})

	it('excludes an announced tab even when its path would match', () => {
		const targets: NavTarget[] = [
			{ id: 'a', label: 'A', to: '/a', status: 'coming-soon' },
			{ id: 'b', label: 'B', to: '/a/b' },
		]

		const result = matchingTargets(targets, '/none', '/a/b')

		expect(result.map((target) => target.label)).toEqual(['B'])
	})
})
