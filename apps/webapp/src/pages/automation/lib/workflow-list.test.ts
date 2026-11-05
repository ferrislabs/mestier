import { describe, expect, it } from 'vitest'
import {
	ALL_TAGS,
	collectTags,
	filterWorkflows,
	sortWorkflows,
} from '#/pages/automation/lib/workflow-list'

const WORKFLOWS = [
	{
		name: 'Relance des devis',
		tags: ['Ventes'],
		updatedAt: '2026-08-01T00:00:00Z',
	},
	{
		name: 'Fin d’intervention',
		tags: ['Interventions', 'Clients'],
		updatedAt: '2026-08-03T00:00:00Z',
	},
	{ name: 'Ticket urgent', tags: [], updatedAt: '2026-08-02T00:00:00Z' },
]

describe('workflow list', () => {
	it('collects the distinct tags sorted', () => {
		expect(collectTags(WORKFLOWS)).toEqual([
			'Clients',
			'Interventions',
			'Ventes',
		])
	})

	it('filters by query ignoring accents and case', () => {
		expect(
			filterWorkflows(WORKFLOWS, 'INTERVENTION', ALL_TAGS).map((w) => w.name),
		).toEqual(['Fin d’intervention'])
	})

	it('filters by tag', () => {
		expect(filterWorkflows(WORKFLOWS, '', 'Ventes')).toHaveLength(1)
		expect(filterWorkflows(WORKFLOWS, '', ALL_TAGS)).toHaveLength(3)
	})

	it('sorts by last update, most recent first', () => {
		expect(sortWorkflows(WORKFLOWS, 'updated').map((w) => w.name)).toEqual([
			'Fin d’intervention',
			'Ticket urgent',
			'Relance des devis',
		])
	})

	it('sorts by name in both directions', () => {
		expect(sortWorkflows(WORKFLOWS, 'name-asc')[0]?.name).toBe(
			'Fin d’intervention',
		)
		expect(sortWorkflows(WORKFLOWS, 'name-desc')[0]?.name).toBe('Ticket urgent')
	})
})
