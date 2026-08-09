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
			'Clients',
		])
		expect(labelsOf('/o/baptiste/settings')).toEqual(['Baptiste', 'Paramètres'])
	})

	it('stacks entries from the most general to the most precise', () => {
		expect(labelsOf('/o/baptiste/crm/customers/pipeline')).toEqual([
			'Baptiste',
			'CRM',
			'Clients',
			'Pipeline',
		])
	})

	it('adds the detail label last', () => {
		expect(
			labelsOf('/o/baptiste/crm/customers/abc-123', 'Marie Leroy'),
		).toEqual(['Baptiste', 'CRM', 'Clients', 'Marie Leroy'])
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

	it('resolves the employees trail', () => {
		expect(labelsOf('/o/baptiste/hr/employees')).toEqual([
			'Baptiste',
			'RH',
			'Employés',
		])
	})

	it('resolves the planning trail', () => {
		expect(labelsOf('/o/baptiste/planning/team')).toEqual([
			'Baptiste',
			'Planning',
			'Vue équipe',
		])
	})

	it('resolves the task list trail', () => {
		expect(labelsOf('/o/baptiste/planning/tasks')).toEqual([
			'Baptiste',
			'Planning',
			'Liste des tâches',
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
