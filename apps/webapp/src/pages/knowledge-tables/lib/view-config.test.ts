import { describe, expect, it } from 'vitest'
import type {
	ColumnKind,
	TableColumn,
	TableRow,
} from '#/pages/knowledge-tables/lib/types'
import {
	applySort,
	draftFromFilter,
	groupableColumns,
	groupRows,
	kanbanFields,
	MAX_SORTS,
	NO_VALUE,
	normalizeConfig,
	rowFilters,
	sameQuery,
	toggleHidden,
	toRowFilter,
	visibleColumns,
} from '#/pages/knowledge-tables/lib/view-config'

function column(
	id: string,
	kind: ColumnKind,
	overrides: Partial<TableColumn> = {},
): TableColumn {
	return {
		id,
		name: id,
		kind,
		config: {},
		is_primary: false,
		position: 1,
		...overrides,
	}
}

function row(id: string, values: Record<string, unknown>): TableRow {
	return {
		id,
		values,
		labels: {},
		created_at: '2026-11-11T08:00:00Z',
		updated_at: '2026-11-11T08:00:00Z',
	}
}

const name = column('name', 'text', { is_primary: true, position: 0 })
const status = column('status', 'single_select', {
	position: 2,
	config: {
		options: [
			{ id: 'todo', label: 'À faire', color: 'gray' },
			{ id: 'doing', label: 'En cours', color: 'blue' },
		],
	},
})
const amount = column('amount', 'currency', { position: 1 })
const due = column('due', 'date', { position: 3 })
const done = column('done', 'checkbox', { position: 4 })
const owner = column('owner', 'member', { position: 5 })
const columns = [status, amount, name, due, done, owner]

describe('normalizeConfig', () => {
	it('fills the missing parts of a view config', () => {
		expect(normalizeConfig({})).toEqual({
			filters: [],
			sorts: [],
			hidden: [],
			group_by: null,
			widths: {},
		})
	})
})

describe('visibleColumns', () => {
	it('orders by position, primary first, without the hidden ones', () => {
		expect(
			visibleColumns(columns, ['amount']).map((entry) => entry.id),
		).toEqual(['name', 'status', 'due', 'done', 'owner'])
	})

	it('never hides the primary column', () => {
		expect(visibleColumns(columns, ['name']).at(0)?.id).toBe('name')
	})
})

describe('toggleHidden', () => {
	it('hides then shows a column', () => {
		expect(toggleHidden([], 'due')).toEqual(['due'])
		expect(toggleHidden(['due', 'done'], 'due')).toEqual(['done'])
	})
})

describe('applySort', () => {
	it('replaces the sort of the same column', () => {
		expect(
			applySort([{ column_id: 'due', direction: 'asc' }], 'due', 'desc'),
		).toEqual([{ column_id: 'due', direction: 'desc' }])
	})

	it('keeps three sorts at most', () => {
		const sorts = [
			{ column_id: 'a', direction: 'asc' as const },
			{ column_id: 'b', direction: 'asc' as const },
			{ column_id: 'c', direction: 'asc' as const },
		]
		const next = applySort(sorts, 'd', 'desc')
		expect(next).toHaveLength(MAX_SORTS)
		expect(next.at(-1)).toEqual({ column_id: 'd', direction: 'desc' })
	})
})

describe('toRowFilter', () => {
	it('drops a filter that still needs its value', () => {
		expect(
			toRowFilter({ column_id: 'amount', operator: 'gt', value: '' }, amount),
		).toBeNull()
		expect(
			toRowFilter(
				{ column_id: 'status', operator: 'any_of', value: [] },
				status,
			),
		).toBeNull()
	})

	it('keeps an emptiness filter without value', () => {
		expect(
			toRowFilter(
				{ column_id: 'due', operator: 'is_empty', value: undefined },
				due,
			),
		).toEqual({ column_id: 'due', operator: 'is_empty' })
		expect(
			toRowFilter(
				{ column_id: 'done', operator: 'checked', value: null },
				done,
			),
		).toEqual({ column_id: 'done', operator: 'checked' })
	})

	it('turns typed amounts into cents and numbers into numbers', () => {
		expect(
			toRowFilter(
				{ column_id: 'amount', operator: 'gt', value: '1 000' },
				amount,
			),
		).toEqual({ column_id: 'amount', operator: 'gt', value: 100000 })
		expect(
			toRowFilter(
				{ column_id: 'n', operator: 'le', value: '2,5' },
				column('n', 'number'),
			),
		).toEqual({ column_id: 'n', operator: 'le', value: 2.5 })
	})

	it('refuses an unreadable value', () => {
		expect(
			toRowFilter(
				{ column_id: 'amount', operator: 'gt', value: 'beaucoup' },
				amount,
			),
		).toBeNull()
		expect(
			toRowFilter(
				{ column_id: 'due', operator: 'on', value: '31/02/2026' },
				due,
			),
		).toBeNull()
	})

	it('reads a date as AAAA-MM-JJ', () => {
		expect(
			toRowFilter(
				{ column_id: 'due', operator: 'before', value: '05/11/2026' },
				due,
			),
		).toEqual({ column_id: 'due', operator: 'before', value: '2026-11-05' })
	})

	it('keeps option ids', () => {
		expect(
			toRowFilter(
				{ column_id: 'status', operator: 'is', value: 'doing' },
				status,
			),
		).toEqual({ column_id: 'status', operator: 'is', value: 'doing' })
		expect(
			toRowFilter(
				{ column_id: 'status', operator: 'any_of', value: ['todo', 'doing'] },
				status,
			),
		).toEqual({
			column_id: 'status',
			operator: 'any_of',
			value: ['todo', 'doing'],
		})
	})

	it('refuses an operator the kind does not allow', () => {
		expect(
			toRowFilter(
				{ column_id: 'done', operator: 'contains', value: 'x' },
				done,
			),
		).toBeNull()
	})
})

describe('draftFromFilter', () => {
	it('writes saved numbers back the way they are typed', () => {
		expect(
			draftFromFilter(
				{ column_id: 'amount', operator: 'gt', value: 100050 } as never,
				amount,
			),
		).toMatchObject({ column_id: 'amount', operator: 'gt', value: '1000,50' })
		expect(
			draftFromFilter(
				{ column_id: 'due', operator: 'on', value: '2026-11-05' } as never,
				due,
			),
		).toMatchObject({ column_id: 'due', operator: 'on', value: '05/11/2026' })
	})
})

describe('rowFilters', () => {
	it('keeps the complete filters of known columns only', () => {
		expect(
			rowFilters(
				[
					{ column_id: 'amount', operator: 'gt', value: '10' },
					{ column_id: 'amount', operator: 'lt', value: '' },
					{ column_id: 'gone', operator: 'is_empty', value: null },
				],
				columns,
			),
		).toEqual([{ column_id: 'amount', operator: 'gt', value: 1000 }])
	})
})

describe('sameQuery', () => {
	it('compares filters and sorts', () => {
		const filters = [{ column_id: 'a', operator: 'is_empty' as const }]
		expect(
			sameQuery({ filters, sorts: [] }, { filters: [...filters], sorts: [] }),
		).toBe(true)
		expect(
			sameQuery(
				{ filters, sorts: [] },
				{ filters, sorts: [{ column_id: 'a', direction: 'asc' }] },
			),
		).toBe(false)
	})
})

describe('groupableColumns', () => {
	it('lists the single choice columns', () => {
		expect(groupableColumns(columns).map((entry) => entry.id)).toEqual([
			'status',
		])
	})
})

describe('groupRows', () => {
	it('puts each row under its option, the others under Sans valeur', () => {
		const rows = [
			row('r1', { status: 'doing' }),
			row('r2', {}),
			row('r3', { status: 'todo' }),
			row('r4', { status: 'removed' }),
			row('r5', { status: 'doing' }),
		]
		const groups = groupRows(rows, status)
		expect(
			groups.map((group) => [
				group.id,
				group.label,
				group.rows.map((entry) => entry.id),
			]),
		).toEqual([
			[null, NO_VALUE, ['r2', 'r4']],
			['todo', 'À faire', ['r3']],
			['doing', 'En cours', ['r1', 'r5']],
		])
		expect(groups[2].color).toBe('blue')
	})
})

describe('kanbanFields', () => {
	it('shows up to three visible fields besides the title and the group', () => {
		expect(
			kanbanFields(columns, ['due'], 'status').map((entry) => entry.id),
		).toEqual(['amount', 'done', 'owner'])
		expect(
			kanbanFields(columns, ['due', 'done', 'owner'], 'status').map(
				(entry) => entry.id,
			),
		).toEqual(['amount'])
	})
})
