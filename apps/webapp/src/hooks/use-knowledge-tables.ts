import {
	type InfiniteData,
	type QueryClient,
	useInfiniteQuery,
	useMutation,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type {
	ColumnConfig,
	ColumnKind,
	KnowledgeTable,
	KnowledgeTableDetail,
	RowFilter,
	RowSort,
	RowValues,
	TableColumn,
	TableRow,
	TableRowPage,
	TableView,
	ViewConfig,
	ViewKind,
} from '#/pages/knowledge-tables/lib/types'

const TABLES_PATH = '/api/v1/organizations/{organization_id}/knowledge/tables'
const TABLE_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}'
const COLUMNS_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns'
const COLUMN_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/{column_id}'
const VIEWS_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views'
const VIEW_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views/{view_id}'
const ROWS_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows'
const ROWS_QUERY_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/query'
const ROWS_DELETE_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/delete'
const ROW_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/{row_id}'

export const ROW_PAGE_SIZE = 100
const DELETE_BATCH = 1000

const TABLES_KEY = 'knowledge-tables'

export type RowSource =
	| { kind: 'view'; viewId: string }
	| { kind: 'query'; filters: RowFilter[]; sorts: RowSort[] }

type RowPages = InfiniteData<TableRowPage, number>

function tableKeys(organizationId: string) {
	return {
		all: [TABLES_KEY, organizationId] as const,
		list: [TABLES_KEY, organizationId, 'list'] as const,
		table: (tableId: string) =>
			[TABLES_KEY, organizationId, 'table', tableId] as const,
		rows: (tableId: string) =>
			[TABLES_KEY, organizationId, 'rows', tableId] as const,
	}
}

export function useKnowledgeTables(organizationId: string) {
	return useQuery({
		queryKey: tableKeys(organizationId).list,
		queryFn: async ({ signal }): Promise<KnowledgeTable[]> => {
			const response = await window.api.get(TABLES_PATH, {
				path: { organization_id: organizationId },
				overrides: { signal },
			})
			return response.data
		},
		enabled: Boolean(organizationId),
	})
}

export function useKnowledgeTable(organizationId: string, tableId: string) {
	return useQuery({
		queryKey: tableKeys(organizationId).table(tableId),
		queryFn: async ({ signal }): Promise<KnowledgeTableDetail> => {
			const response = await window.api.get(TABLE_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
				overrides: { signal },
			})
			return response.data
		},
		enabled: Boolean(organizationId) && Boolean(tableId),
	})
}

function readRowPage(
	organizationId: string,
	tableId: string,
	source: RowSource,
	offset: number,
	signal: AbortSignal,
): Promise<{ data: TableRowPage }> {
	const path = { organization_id: organizationId, table_id: tableId }
	if (source.kind === 'view') {
		return window.api.get(ROWS_PATH, {
			path,
			query: { view_id: source.viewId, limit: ROW_PAGE_SIZE, offset },
			overrides: { signal },
		})
	}
	return window.api.post(ROWS_QUERY_PATH, {
		path,
		body: {
			filters: source.filters,
			sorts: source.sorts,
			limit: ROW_PAGE_SIZE,
			offset,
		},
		overrides: { signal },
	})
}

export function useTableRows(
	organizationId: string,
	tableId: string,
	source: RowSource | null,
) {
	return useInfiniteQuery({
		queryKey: [...tableKeys(organizationId).rows(tableId), source],
		queryFn: async ({ pageParam, signal }): Promise<TableRowPage> => {
			if (!source) throw new Error('Aucune vue')
			const response = await readRowPage(
				organizationId,
				tableId,
				source,
				pageParam,
				signal,
			)
			return response.data
		},
		initialPageParam: 0,
		getNextPageParam: (last: TableRowPage) => {
			const next = last.offset + last.rows.length
			return last.rows.length > 0 && next < last.total ? next : undefined
		},
		enabled: Boolean(organizationId) && Boolean(tableId) && source !== null,
		placeholderData: (previous) => previous,
	})
}

function useTableInvalidation(organizationId: string) {
	const queryClient = useQueryClient()
	const keys = tableKeys(organizationId)
	return {
		list: () => queryClient.invalidateQueries({ queryKey: keys.list }),
		table: (tableId: string) =>
			queryClient.invalidateQueries({ queryKey: keys.table(tableId) }),
		rows: (tableId: string) =>
			queryClient.invalidateQueries({ queryKey: keys.rows(tableId) }),
	}
}

export function useCreateKnowledgeTable(organizationId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (input: {
			name: string
			description: string | null
		}): Promise<KnowledgeTableDetail> => {
			const response = await window.api.post(TABLES_PATH, {
				path: { organization_id: organizationId },
				body: input,
			})
			return response.data
		},
		onSuccess: () => {
			void invalidate.list()
		},
	})
}

export function useUpdateKnowledgeTable(organizationId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (input: {
			tableId: string
			name?: string
			description?: string | null
		}) => {
			const { tableId, ...body } = input
			const response = await window.api.patch(TABLE_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
				body,
			})
			return response.data
		},
		onSuccess: (_, input) => {
			void invalidate.list()
			void invalidate.table(input.tableId)
		},
	})
}

export function useDeleteKnowledgeTable(organizationId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (tableId: string) => {
			await window.api.delete(TABLE_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
			})
		},
		onSuccess: () => {
			void invalidate.list()
		},
	})
}

export function useCreateTableColumn(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (input: {
			name: string
			kind: ColumnKind
			config: ColumnConfig
		}): Promise<TableColumn> => {
			const response = await window.api.post(COLUMNS_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
				body: input,
			})
			return response.data
		},
		onSuccess: () => {
			void invalidate.table(tableId)
		},
	})
}

export function useUpdateTableColumn(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (input: {
			columnId: string
			name?: string
			kind?: ColumnKind
			config?: ColumnConfig
		}): Promise<TableColumn> => {
			const { columnId, ...body } = input
			const response = await window.api.patch(COLUMN_PATH, {
				path: {
					organization_id: organizationId,
					table_id: tableId,
					column_id: columnId,
				},
				body,
			})
			return response.data
		},
		onSuccess: () => {
			void invalidate.table(tableId)
			void invalidate.rows(tableId)
		},
	})
}

export function useDeleteTableColumn(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (columnId: string) => {
			await window.api.delete(COLUMN_PATH, {
				path: {
					organization_id: organizationId,
					table_id: tableId,
					column_id: columnId,
				},
			})
		},
		onSuccess: () => {
			void invalidate.table(tableId)
			void invalidate.rows(tableId)
		},
	})
}

export function useCreateTableView(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (input: {
			name: string
			kind: ViewKind
			config: ViewConfig
		}): Promise<TableView> => {
			const response = await window.api.post(VIEWS_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
				body: input,
			})
			return response.data
		},
		onSuccess: () => {
			void invalidate.table(tableId)
		},
	})
}

export function useUpdateTableView(organizationId: string, tableId: string) {
	const queryClient = useQueryClient()
	const keys = tableKeys(organizationId)
	return useMutation({
		mutationFn: async (input: {
			viewId: string
			name?: string
			kind?: ViewKind
			config?: ViewConfig
		}): Promise<TableView> => {
			const { viewId, ...body } = input
			const response = await window.api.patch(VIEW_PATH, {
				path: {
					organization_id: organizationId,
					table_id: tableId,
					view_id: viewId,
				},
				body,
			})
			return response.data
		},
		onSuccess: (view) => {
			queryClient.setQueryData<KnowledgeTableDetail>(
				keys.table(tableId),
				(table) =>
					table
						? {
								...table,
								views: table.views.map((entry) =>
									entry.id === view.id ? view : entry,
								),
							}
						: table,
			)
			void queryClient.invalidateQueries({ queryKey: keys.table(tableId) })
		},
	})
}

export function useDeleteTableView(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (viewId: string) => {
			await window.api.delete(VIEW_PATH, {
				path: {
					organization_id: organizationId,
					table_id: tableId,
					view_id: viewId,
				},
			})
		},
		onSuccess: () => {
			void invalidate.table(tableId)
		},
	})
}

function mapRows(
	pages: RowPages | undefined,
	change: (row: TableRow) => TableRow,
): RowPages | undefined {
	if (!pages) return pages
	return {
		...pages,
		pages: pages.pages.map((page) => ({
			...page,
			rows: page.rows.map(change),
		})),
	}
}

function mergeValues(values: RowValues, patch: RowValues): RowValues {
	const merged = { ...values }
	for (const [key, value] of Object.entries(patch)) {
		if (value === null) delete merged[key]
		else merged[key] = value
	}
	return merged
}

function cachedRow(
	queryClient: QueryClient,
	key: readonly unknown[],
	rowId: string,
): TableRow | undefined {
	for (const [, pages] of queryClient.getQueriesData<RowPages>({
		queryKey: key,
	})) {
		for (const page of pages?.pages ?? []) {
			const found = page.rows.find((row) => row.id === rowId)
			if (found) return found
		}
	}
	return undefined
}

function patchCachedRows(
	queryClient: QueryClient,
	key: readonly unknown[],
	rowId: string,
	change: (row: TableRow) => TableRow,
) {
	queryClient.setQueriesData<RowPages>({ queryKey: key }, (pages) =>
		mapRows(pages, (row) => (row.id === rowId ? change(row) : row)),
	)
}

export function useUpdateTableRow(organizationId: string, tableId: string) {
	const queryClient = useQueryClient()
	const rowsKey = tableKeys(organizationId).rows(tableId)
	return useMutation({
		mutationFn: async (input: {
			rowId: string
			values: RowValues
		}): Promise<TableRow> => {
			const response = await window.api.patch(ROW_PATH, {
				path: {
					organization_id: organizationId,
					table_id: tableId,
					row_id: input.rowId,
				},
				body: { values: input.values },
			})
			return response.data
		},
		onMutate: async (input) => {
			await queryClient.cancelQueries({ queryKey: rowsKey })
			const previous: RowValues = {}
			const absent: string[] = []
			const current = cachedRow(queryClient, rowsKey, input.rowId)
			for (const key of Object.keys(input.values)) {
				if (current && key in current.values)
					previous[key] = current.values[key]
				else absent.push(key)
			}
			patchCachedRows(queryClient, rowsKey, input.rowId, (row) => ({
				...row,
				values: mergeValues(row.values, input.values),
			}))
			return { previous, absent }
		},
		onError: (_error, input, context) => {
			patchCachedRows(queryClient, rowsKey, input.rowId, (row) => {
				const values = { ...row.values }
				for (const [key, written] of Object.entries(input.values)) {
					const untouched =
						written === null
							? !(key in values)
							: JSON.stringify(values[key]) === JSON.stringify(written)
					if (!untouched) continue
					if (context && key in context.previous)
						values[key] = context.previous[key]
					else delete values[key]
				}
				return { ...row, values }
			})
			void queryClient.invalidateQueries({ queryKey: rowsKey })
		},
		onSuccess: (row) => {
			patchCachedRows(queryClient, rowsKey, row.id, () => row)
		},
	})
}

export function useCreateTableRow(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (values: RowValues): Promise<TableRow> => {
			const response = await window.api.post(ROWS_PATH, {
				path: { organization_id: organizationId, table_id: tableId },
				body: { values },
			})
			return response.data
		},
		onSuccess: () => {
			void invalidate.rows(tableId)
		},
	})
}

export function useDeleteTableRows(organizationId: string, tableId: string) {
	const invalidate = useTableInvalidation(organizationId)
	return useMutation({
		mutationFn: async (rowIds: string[]): Promise<number> => {
			const path = { organization_id: organizationId, table_id: tableId }
			if (rowIds.length === 1) {
				await window.api.delete(ROW_PATH, {
					path: { ...path, row_id: rowIds[0] },
				})
				return 1
			}
			let deleted = 0
			for (let start = 0; start < rowIds.length; start += DELETE_BATCH) {
				const response = await window.api.post(ROWS_DELETE_PATH, {
					path,
					body: { ids: rowIds.slice(start, start + DELETE_BATCH) },
				})
				deleted += response.data.deleted
			}
			return deleted
		},
		onSuccess: () => {
			void invalidate.rows(tableId)
		},
	})
}

const ARTICLES_PATH =
	'/api/v1/organizations/{organization_id}/knowledge/articles'
const DRIVE_FILE_PAGE = 100

export interface DriveFileChoice {
	id: string
	title: string
}

export function useDriveFileChoices(organizationId: string, enabled: boolean) {
	return useQuery({
		queryKey: [TABLES_KEY, organizationId, 'drive-files'],
		queryFn: async ({ signal }): Promise<DriveFileChoice[]> => {
			const response = await window.api.get(ARTICLES_PATH, {
				path: { organization_id: organizationId },
				query: { page: 1, per_page: DRIVE_FILE_PAGE },
				overrides: { signal },
			})
			return response.data
				.filter((article) => article.kind === 'file')
				.map((article) => ({ id: article.id, title: article.title }))
		},
		enabled: Boolean(organizationId) && enabled,
	})
}
