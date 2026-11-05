import { describe, expect, it } from 'vitest'
import type { Project } from '#/hooks/use-projects'
import { EMPTY_EXTRAS } from '#/pages/projects/lib/model'
import {
	CLIENT_INTERNAL,
	DEFAULT_FILTERS,
	filterRows,
	listEmpty,
	type ProjectRow,
	projectsSummary,
	projectTiles,
	resolveClient,
	statusOf,
} from '#/pages/projects/lib/project-list'

function row(
	name: string,
	overrides: Partial<ProjectRow> & { customerId?: string | null } = {},
): ProjectRow {
	const { customerId = 'c1', ...rest } = overrides
	const project = {
		id: name,
		organization_id: 'o',
		name,
		customer_id: customerId,
		is_internal: customerId === null,
		archived_at: null,
		created_at: '',
		updated_at: '',
	} as Project
	return {
		project,
		status: 'active',
		customerName: customerId === null ? null : 'Duval',
		extras: EMPTY_EXTRAS,
		missions: { total: 0, todo: 0, inProgress: 2, done: 0, progress: 0 },
		doeRevision: null,
		owner: undefined,
		...rest,
	}
}

describe('project list', () => {
	const rows = [
		row('Zèbre'),
		row('Alpha', { customerId: null }),
		row('Vieux', { status: 'archived', doeRevision: 'A' }),
	]

	it('shows active projects by name, archived ones apart', () => {
		expect(
			filterRows(rows, DEFAULT_FILTERS).map((r) => r.project.name),
		).toEqual(['Alpha', 'Zèbre'])
		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, status: 'tous' }).map(
				(r) => r.project.name,
			),
		).toEqual(['Alpha', 'Zèbre', 'Vieux'])
	})

	it('filters internal projects and searches without accents', () => {
		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, client: CLIENT_INTERNAL }).map(
				(r) => r.project.name,
			),
		).toEqual(['Alpha'])
		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, query: 'zebre' }).map(
				(r) => r.project.name,
			),
		).toEqual(['Zèbre'])
	})

	it('tells why the list is empty', () => {
		expect(listEmpty(0, DEFAULT_FILTERS)).toBe('none')
		expect(listEmpty(3, DEFAULT_FILTERS)).toBe('no-active')
		expect(listEmpty(3, { ...DEFAULT_FILTERS, status: 'archives' })).toBe(
			'no-archived',
		)
		expect(listEmpty(3, { ...DEFAULT_FILTERS, query: 'x' })).toBe('no-match')
	})

	it('summarises and counts the tiles on active projects only', () => {
		expect(projectsSummary(rows)).toBe('2 projets actifs, 1 archivé')
		expect(projectTiles(rows)).toEqual({
			active: 2,
			internal: 1,
			missionsInProgress: 4,
			doeToCreate: 2,
		})
	})

	it('falls back to all clients for an unknown customer', () => {
		expect(resolveClient('zzz', [{ id: 'c1' }])).toBe('tous')
		expect(resolveClient('c1', [{ id: 'c1' }])).toBe('c1')
		expect(statusOf({ archived_at: '2026-01-01T00:00:00Z' })).toBe('archived')
	})
})
