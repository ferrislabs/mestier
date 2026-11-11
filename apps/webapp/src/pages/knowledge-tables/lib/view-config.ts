import { optionIds } from '#/pages/knowledge-tables/lib/cells'
import {
	columnOptions,
	filterOperators,
	isSortable,
	operatorNeedsValue,
	orderedColumns,
} from '#/pages/knowledge-tables/lib/columns'
import {
	centsInputText,
	formatIsoDate,
	numberInputText,
	parseDateInput,
	parseEurosInput,
	parseNumberInput,
} from '#/pages/knowledge-tables/lib/format'
import type {
	FilterCondition,
	FilterDraft,
	RowFilter,
	RowSort,
	SelectColor,
	SortDirection,
	TableColumn,
	TableRow,
	ViewConfig,
} from '#/pages/knowledge-tables/lib/types'

export const MAX_SORTS = 3
export const NO_VALUE = 'Sans valeur'
export const KANBAN_FIELDS = 3

export interface NormalizedConfig {
	filters: RowFilter[]
	sorts: RowSort[]
	hidden: string[]
	group_by: string | null
	widths: Record<string, number>
}

export function normalizeConfig(config: ViewConfig): NormalizedConfig {
	return {
		filters: config.filters ?? [],
		sorts: config.sorts ?? [],
		hidden: config.hidden ?? [],
		group_by: config.group_by ?? null,
		widths: config.widths ?? {},
	}
}

export function visibleColumns(
	columns: readonly TableColumn[],
	hidden: readonly string[],
): TableColumn[] {
	return orderedColumns(columns).filter(
		(column) => column.is_primary || !hidden.includes(column.id),
	)
}

export function toggleHidden(
	hidden: readonly string[],
	columnId: string,
): string[] {
	return hidden.includes(columnId)
		? hidden.filter((id) => id !== columnId)
		: [...hidden, columnId]
}

export function applySort(
	sorts: readonly RowSort[],
	columnId: string,
	direction: SortDirection,
): RowSort[] {
	const sort = { column_id: columnId, direction }
	if (sorts.some((entry) => entry.column_id === columnId)) {
		return sorts.map((entry) => (entry.column_id === columnId ? sort : entry))
	}
	if (sorts.length < MAX_SORTS) return [...sorts, sort]
	return [...sorts.slice(0, MAX_SORTS - 1), sort]
}

function apiFilter(
	column_id: string,
	operator: RowFilter['operator'],
	value?: unknown,
): RowFilter {
	return value === undefined
		? { column_id, operator }
		: { column_id, operator, value: value as RowFilter['value'] }
}

function filterValue(draft: FilterCondition, column: TableColumn): unknown {
	const text = typeof draft.value === 'string' ? draft.value : ''
	switch (column.kind) {
		case 'number': {
			const parsed = parseNumberInput(text)
			return parsed.kind === 'value' ? parsed.value : undefined
		}
		case 'currency': {
			const parsed = parseEurosInput(text)
			return parsed.kind === 'value' ? parsed.value : undefined
		}
		case 'date': {
			const parsed = parseDateInput(text)
			return parsed.kind === 'value' ? parsed.value : undefined
		}
		case 'single_select':
		case 'multi_select': {
			if (draft.operator === 'any_of' || draft.operator === 'all_of') {
				const ids = optionIds(draft.value)
				return ids.length > 0 ? ids : undefined
			}
			return text === '' ? undefined : text
		}
		default:
			return text === '' ? undefined : text
	}
}

export function toRowFilter(
	draft: FilterCondition,
	column: TableColumn,
): RowFilter | null {
	if (!filterOperators(column.kind).includes(draft.operator)) return null
	if (!operatorNeedsValue(draft.operator)) {
		return apiFilter(draft.column_id, draft.operator)
	}
	const value = filterValue(draft, column)
	return value === undefined
		? null
		: apiFilter(draft.column_id, draft.operator, value)
}

let filterKeys = 0

export function newFilterKey(): string {
	filterKeys += 1
	return `filter-${filterKeys}`
}

export function draftFromFilter(
	filter: RowFilter,
	column: TableColumn | undefined,
): FilterDraft {
	const value: unknown = filter.value
	const draft = {
		key: newFilterKey(),
		column_id: filter.column_id,
		operator: filter.operator,
	}
	if (column?.kind === 'number' && typeof value === 'number') {
		return { ...draft, value: numberInputText(value) }
	}
	if (column?.kind === 'currency' && typeof value === 'number') {
		return { ...draft, value: centsInputText(value) }
	}
	if (column?.kind === 'date' && typeof value === 'string') {
		return { ...draft, value: formatIsoDate(value) }
	}
	return { ...draft, value }
}

export function rowFilters(
	drafts: readonly FilterCondition[],
	columns: readonly TableColumn[],
): RowFilter[] {
	return drafts.flatMap((draft) => {
		const column = columns.find((entry) => entry.id === draft.column_id)
		const filter = column ? toRowFilter(draft, column) : null
		return filter ? [filter] : []
	})
}

function filterFits(draft: FilterCondition, column: TableColumn): boolean {
	if (!filterOperators(column.kind).includes(draft.operator)) return false
	if (
		(column.kind === 'single_select' || column.kind === 'multi_select') &&
		operatorNeedsValue(draft.operator)
	) {
		const known = columnOptions(column).map((option) => option.id)
		return optionIds(draft.value).every((id) => known.includes(id))
	}
	return true
}

export function pruneViewState<
	T extends {
		filters: readonly FilterDraft[]
		sorts: readonly RowSort[]
		hidden: readonly string[]
	},
>(state: T, columns: readonly TableColumn[]): T {
	const byId = new Map(columns.map((column) => [column.id, column]))
	const filters = state.filters.filter((draft) => {
		const column = byId.get(draft.column_id)
		return column !== undefined && filterFits(draft, column)
	})
	const sorts = state.sorts.filter((sort) => {
		const column = byId.get(sort.column_id)
		return column !== undefined && isSortable(column.kind)
	})
	const hidden = state.hidden.filter((id) => byId.has(id))
	if (
		filters.length === state.filters.length &&
		sorts.length === state.sorts.length &&
		hidden.length === state.hidden.length
	) {
		return state
	}
	return { ...state, filters, sorts, hidden }
}

export interface RowQueryShape {
	filters: readonly RowFilter[]
	sorts: readonly RowSort[]
}

export function sameQuery(a: RowQueryShape, b: RowQueryShape): boolean {
	return (
		JSON.stringify([a.filters, a.sorts]) ===
		JSON.stringify([b.filters, b.sorts])
	)
}

export function groupableColumns(
	columns: readonly TableColumn[],
): TableColumn[] {
	return orderedColumns(columns).filter(
		(column) => column.kind === 'single_select',
	)
}

export interface KanbanGroup {
	id: string | null
	label: string
	color: SelectColor
	rows: TableRow[]
}

export function groupRows(
	rows: readonly TableRow[],
	column: TableColumn,
): KanbanGroup[] {
	const options = columnOptions(column)
	const groups: KanbanGroup[] = [
		{ id: null, label: NO_VALUE, color: 'gray', rows: [] },
		...options.map((option) => ({
			id: option.id,
			label: option.label,
			color: option.color,
			rows: [],
		})),
	]
	for (const row of rows) {
		const value = row.values[column.id]
		const group =
			groups.find((entry) => entry.id !== null && entry.id === value) ??
			groups[0]
		group.rows.push(row)
	}
	return groups
}

export function kanbanFields(
	columns: readonly TableColumn[],
	hidden: readonly string[],
	groupBy: string | null,
): TableColumn[] {
	return visibleColumns(columns, hidden)
		.filter((column) => !column.is_primary && column.id !== groupBy)
		.slice(0, KANBAN_FIELDS)
}
