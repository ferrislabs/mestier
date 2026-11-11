import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { RowSource } from '#/hooks/use-knowledge-tables'
import type {
	FilterDraft,
	RowFilter,
	RowSort,
	TableColumn,
	TableView,
	ViewConfig,
} from '#/pages/knowledge-tables/lib/types'
import {
	draftFromFilter,
	normalizeConfig,
	pruneViewState,
	rowFilters,
	sameQuery,
} from '#/pages/knowledge-tables/lib/view-config'

const SAVE_DELAY_MS = 600
const QUERY_DELAY_MS = 300

export interface ViewState {
	filters: FilterDraft[]
	sorts: RowSort[]
	hidden: string[]
}

interface LocalState extends ViewState {
	viewId: string
}

interface PendingSave {
	viewId: string
	config: ViewConfig
}

function useDebounced<T>(value: T, delay: number): T {
	const [debounced, setDebounced] = useState(value)
	useEffect(() => {
		const timer = setTimeout(() => setDebounced(value), delay)
		return () => clearTimeout(timer)
	}, [value, delay])
	return debounced
}

function savedState(
	view: TableView,
	columns: readonly TableColumn[],
): ViewState {
	const config = normalizeConfig(view.config)
	return {
		filters: config.filters.map((filter) =>
			draftFromFilter(
				filter,
				columns.find((column) => column.id === filter.column_id),
			),
		),
		sorts: config.sorts,
		hidden: config.hidden,
	}
}

export function useViewDraft({
	view,
	columns,
	canSave,
	onSave,
}: {
	view: TableView | null
	columns: readonly TableColumn[]
	canSave: boolean
	onSave: (viewId: string, config: ViewConfig) => void
}) {
	const [local, setLocal] = useState<LocalState | null>(null)
	const pending = useRef<PendingSave | null>(null)
	const timer = useRef<ReturnType<typeof setTimeout> | null>(null)
	const saveRef = useRef(onSave)
	saveRef.current = onSave

	const flush = useCallback(() => {
		if (timer.current) clearTimeout(timer.current)
		timer.current = null
		const next = pending.current
		pending.current = null
		if (next) saveRef.current(next.viewId, next.config)
	}, [])

	const viewId = view?.id ?? null
	useEffect(() => {
		if (viewId === null) return
		return flush
	}, [viewId, flush])

	const saved = useMemo(
		() => (view ? savedState(view, columns) : null),
		[view, columns],
	)
	const current: ViewState | null =
		view && local?.viewId === view.id ? local : saved
	const state = useMemo(
		() => (current ? pruneViewState(current, columns) : null),
		[current, columns],
	)

	const update = useCallback(
		(change: Partial<ViewState>) => {
			if (!view || !state) return
			const next = { ...state, ...change }
			setLocal({ viewId: view.id, ...next })
			if (!canSave) return
			pending.current = {
				viewId: view.id,
				config: {
					...normalizeConfig(view.config),
					filters: rowFilters(next.filters, columns),
					sorts: next.sorts,
					hidden: next.hidden,
				},
			}
			if (timer.current) clearTimeout(timer.current)
			timer.current = setTimeout(flush, SAVE_DELAY_MS)
		},
		[view, state, canSave, columns, flush],
	)

	const filters: RowFilter[] = useMemo(
		() => (state ? rowFilters(state.filters, columns) : []),
		[state, columns],
	)
	const query = useDebounced(
		useMemo(() => ({ filters, sorts: state?.sorts ?? [] }), [filters, state]),
		QUERY_DELAY_MS,
	)
	const savedQuery = useMemo(
		() =>
			saved
				? { filters: rowFilters(saved.filters, columns), sorts: saved.sorts }
				: null,
		[saved, columns],
	)

	const source: RowSource | null = useMemo(() => {
		if (!view || !savedQuery) return null
		const isSaved = local?.viewId !== view.id || sameQuery(query, savedQuery)
		return isSaved
			? { kind: 'view', viewId: view.id }
			: { kind: 'query', filters: [...query.filters], sorts: [...query.sorts] }
	}, [view, savedQuery, local, query])

	return { state, activeFilterCount: filters.length, source, update, flush }
}
