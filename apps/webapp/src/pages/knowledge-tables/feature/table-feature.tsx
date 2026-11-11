import { useCallback, useEffect, useMemo, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateTableColumn,
	useCreateTableRow,
	useCreateTableView,
	useDeleteTableColumn,
	useDeleteTableRows,
	useDeleteTableView,
	useKnowledgeTable,
	useTableRows,
	useUpdateKnowledgeTable,
	useUpdateTableColumn,
	useUpdateTableRow,
	useUpdateTableView,
} from '#/hooks/use-knowledge-tables'
import { useHasPermission } from '#/hooks/use-permissions'
import { useTableChoices } from '#/pages/knowledge-tables/feature/use-table-choices'
import {
	useExportTableView,
	useImportTableRows,
} from '#/pages/knowledge-tables/feature/use-table-csv'
import { useViewDraft } from '#/pages/knowledge-tables/feature/use-view-draft'
import {
	addOption,
	type ColumnDraft,
	columnDraftError,
	columnUpdate,
	configFromDraft,
	draftFromColumn,
	EMPTY_COLUMN_DRAFT,
	kindChoices,
} from '#/pages/knowledge-tables/lib/column-draft'
import {
	defaultConfig,
	orderedColumns,
	primaryColumn,
} from '#/pages/knowledge-tables/lib/columns'
import {
	csvFileRefusal,
	hiddenDiffers,
} from '#/pages/knowledge-tables/lib/csv-transfer'
import { tableErrorMessage } from '#/pages/knowledge-tables/lib/errors'
import type {
	RowValues,
	TableColumn,
	TableImportReport,
	TableView,
} from '#/pages/knowledge-tables/lib/types'
import {
	applySort,
	groupableColumns,
	groupRows,
	kanbanFields,
	normalizeConfig,
	rowFilters,
	toggleHidden,
	visibleColumns,
} from '#/pages/knowledge-tables/lib/view-config'
import { ColumnDialog } from '#/pages/knowledge-tables/ui/column-dialog'
import type { ColumnAction } from '#/pages/knowledge-tables/ui/column-header-menu'
import { ImportDialog } from '#/pages/knowledge-tables/ui/import-dialog'
import { KanbanBoard } from '#/pages/knowledge-tables/ui/kanban-board'
import { RowSheet } from '#/pages/knowledge-tables/ui/row-sheet'
import {
	TableDialog,
	type TableDraft,
} from '#/pages/knowledge-tables/ui/table-dialog'
import { TableGrid } from '#/pages/knowledge-tables/ui/table-grid'
import { TablePage } from '#/pages/knowledge-tables/ui/table-page'
import { TableToast } from '#/pages/knowledge-tables/ui/table-toast'
import {
	ViewDialog,
	type ViewDraft,
} from '#/pages/knowledge-tables/ui/view-dialog'
import { ViewTabs } from '#/pages/knowledge-tables/ui/view-tabs'
import { ViewToolbar } from '#/pages/knowledge-tables/ui/view-toolbar'

const TOAST_MS = 6000
const NO_IDS: string[] = []

interface TableFeatureProps {
	tableId: string
	viewId: string | null
	onViewChange: (viewId: string) => void
	onBack: () => void
}

export function TableFeature(props: TableFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<TableWorkspace
			key={`${activeOrganization.id}-${props.tableId}`}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

type ColumnEditor =
	| { mode: 'create'; draft: ColumnDraft }
	| { mode: 'edit'; column: TableColumn; draft: ColumnDraft }

type ViewEditor =
	| { mode: 'create'; draft: ViewDraft }
	| { mode: 'rename'; view: TableView; draft: ViewDraft }

interface RowsImport {
	file: File | null
	refusal: string | null
	report: TableImportReport | null
}

const EMPTY_ROWS_IMPORT: RowsImport = {
	file: null,
	refusal: null,
	report: null,
}

type Confirmation =
	| { kind: 'column'; column: TableColumn }
	| { kind: 'view'; view: TableView }
	| { kind: 'rows'; rowIds: string[] }

function confirmationText(confirmation: Confirmation | null) {
	switch (confirmation?.kind) {
		case 'column':
			return {
				title: `Supprimer la colonne « ${confirmation.column.name} » ?`,
				description:
					'Ses valeurs disparaissent de toutes les lignes, ainsi que des filtres et tris des vues.',
				confirm: 'Supprimer la colonne',
			}
		case 'view':
			return {
				title: `Supprimer la vue « ${confirmation.view.name} » ?`,
				description: 'Les lignes restent : seule la vue disparaît.',
				confirm: 'Supprimer la vue',
			}
		case 'rows':
			return {
				title:
					confirmation.rowIds.length === 1
						? 'Supprimer cette ligne ?'
						: `Supprimer ${confirmation.rowIds.length} lignes ?`,
				description: 'Rien ne se récupère.',
				confirm: 'Supprimer',
			}
		default:
			return { title: '', description: '', confirm: 'Supprimer' }
	}
}

function TableWorkspace({
	organizationId,
	tableId,
	viewId,
	onViewChange,
	onBack,
}: TableFeatureProps & { organizationId: string }) {
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const detail = useKnowledgeTable(organizationId, tableId)
	const table = detail.data ?? null
	const columns = useMemo(
		() => orderedColumns(table?.columns ?? []),
		[table?.columns],
	)
	const views = useMemo(
		() => [...(table?.views ?? [])].sort((a, b) => a.position - b.position),
		[table?.views],
	)
	const view = views.find((entry) => entry.id === viewId) ?? views[0] ?? null
	const config = view ? normalizeConfig(view.config) : null
	const primary = primaryColumn(columns)

	const [toast, setToast] = useState<string | null>(null)
	const [selected, setSelected] = useState<Set<string>>(new Set())
	const [openRowId, setOpenRowId] = useState<string | null>(null)
	const [focusRowId, setFocusRowId] = useState<string | null>(null)
	const [tableEditor, setTableEditor] = useState<TableDraft | null>(null)
	const [columnEditor, setColumnEditor] = useState<ColumnEditor | null>(null)
	const [viewEditor, setViewEditor] = useState<ViewEditor | null>(null)
	const [confirmation, setConfirmation] = useState<Confirmation | null>(null)
	const [rowsImport, setRowsImport] = useState<RowsImport | null>(null)

	useEffect(() => {
		if (!toast) return
		const timer = setTimeout(() => setToast(null), TOAST_MS)
		return () => clearTimeout(timer)
	}, [toast])

	const fail = useCallback(
		(error: unknown) => setToast(tableErrorMessage(error)),
		[],
	)

	const updateTable = useUpdateKnowledgeTable(organizationId)
	const createColumn = useCreateTableColumn(organizationId, tableId)
	const updateColumn = useUpdateTableColumn(organizationId, tableId)
	const deleteColumn = useDeleteTableColumn(organizationId, tableId)
	const createView = useCreateTableView(organizationId, tableId)
	const updateView = useUpdateTableView(organizationId, tableId)
	const deleteView = useDeleteTableView(organizationId, tableId)
	const createRow = useCreateTableRow(organizationId, tableId)
	const updateRow = useUpdateTableRow(organizationId, tableId)
	const deleteRows = useDeleteTableRows(organizationId, tableId)
	const importRows = useImportTableRows(organizationId, tableId)
	const exportView = useExportTableView(organizationId, tableId)

	const saveView = useCallback(
		(id: string, next: TableView['config']) =>
			updateView.mutate({ viewId: id, config: next }, { onError: fail }),
		[updateView.mutate, fail],
	)
	const draft = useViewDraft({
		view,
		columns,
		canSave: canManage,
		onSave: saveView,
	})
	const rowsQuery = useTableRows(organizationId, tableId, draft.source)
	const rows = useMemo(
		() => rowsQuery.data?.pages.flatMap((page) => page.rows) ?? [],
		[rowsQuery.data],
	)
	const total = rowsQuery.data?.pages.at(-1)?.total ?? 0
	const choicesFor = useTableChoices(organizationId, columns)

	const hidden = draft.state?.hidden ?? NO_IDS
	const visible = useMemo(
		() => visibleColumns(columns, hidden),
		[columns, hidden],
	)
	const groupable = useMemo(() => groupableColumns(columns), [columns])
	const groupColumn =
		view?.kind === 'kanban'
			? (groupable.find((column) => column.id === config?.group_by) ?? null)
			: null
	const groups = useMemo(
		() => (groupColumn ? groupRows(rows, groupColumn) : []),
		[rows, groupColumn],
	)

	const { hasNextPage, isFetchingNextPage, fetchNextPage } = rowsQuery
	const loadMore = useCallback(() => {
		void fetchNextPage()
	}, [fetchNextPage])
	useEffect(() => {
		if (groupColumn && hasNextPage && !isFetchingNextPage) loadMore()
	}, [groupColumn, hasNextPage, isFetchingNextPage, loadMore])

	const currentViewId = view?.id
	useEffect(() => {
		if (currentViewId) setSelected(new Set())
	}, [currentViewId])

	const openRow = rows.find((row) => row.id === openRowId) ?? null

	const exportUsesSavedView =
		draft.source?.kind === 'query' ||
		(config !== null && hiddenDiffers(hidden, config.hidden))

	const exportCurrentView = () => {
		if (!table || !view) return
		exportView.mutate(
			{ viewId: view.id, tableName: table.name, viewName: view.name },
			{ onError: fail },
		)
	}

	const closeImport = () => {
		setRowsImport(null)
		importRows.reset()
	}

	const submitImport = () => {
		if (!rowsImport?.file || rowsImport.refusal) return
		importRows.mutate(rowsImport.file, {
			onSuccess: (report) =>
				setRowsImport((current) =>
					current ? { ...current, report } : current,
				),
		})
	}

	const changeCell = (rowId: string, columnId: string, value: unknown) => {
		updateRow.mutate(
			{ rowId, values: { [columnId]: value ?? null } },
			{ onError: fail },
		)
	}

	const addRow = (values: RowValues, openAfter: boolean) => {
		createRow.mutate(values, {
			onSuccess: (row) => {
				if (openAfter) setOpenRowId(row.id)
				else setFocusRowId(row.id)
			},
			onError: fail,
		})
	}

	const columnAction = (column: TableColumn, action: ColumnAction) => {
		switch (action) {
			case 'edit':
				setColumnEditor({
					mode: 'edit',
					column,
					draft: draftFromColumn(column),
				})
				return
			case 'hide':
				draft.update({ hidden: toggleHidden(hidden, column.id) })
				return
			case 'sort_asc':
			case 'sort_desc':
				draft.update({
					sorts: applySort(
						draft.state?.sorts ?? [],
						column.id,
						action === 'sort_asc' ? 'asc' : 'desc',
					),
				})
				return
			case 'delete':
				setConfirmation({ kind: 'column', column })
		}
	}

	const columnMutation =
		columnEditor?.mode === 'edit' ? updateColumn : createColumn
	const columnError =
		columnEditor === null
			? null
			: columnEditor.draft.name.trim() === ''
				? null
				: (columnDraftError(columnEditor.draft) ??
					tableErrorMessage(columnMutation.error))

	const submitColumn = async () => {
		if (!columnEditor || columnDraftError(columnEditor.draft)) return
		try {
			if (columnEditor.mode === 'create') {
				await createColumn.mutateAsync({
					name: columnEditor.draft.name.trim(),
					kind: columnEditor.draft.kind,
					config: configFromDraft(columnEditor.draft),
				})
			} else {
				const update = columnUpdate(columnEditor.column, columnEditor.draft)
				if (Object.keys(update).length > 0) {
					await updateColumn.mutateAsync({
						columnId: columnEditor.column.id,
						...update,
					})
				}
			}
			setColumnEditor(null)
		} catch {}
	}

	const viewMutation = viewEditor?.mode === 'rename' ? updateView : createView

	const submitView = async () => {
		if (!viewEditor) return
		const name = viewEditor.draft.name.trim()
		try {
			if (viewEditor.mode === 'create') {
				const created = await createView.mutateAsync({
					name,
					kind: viewEditor.draft.kind,
					config:
						viewEditor.draft.kind === 'kanban'
							? { group_by: viewEditor.draft.groupBy }
							: {},
				})
				onViewChange(created.id)
			} else {
				await updateView.mutateAsync({ viewId: viewEditor.view.id, name })
			}
			setViewEditor(null)
		} catch {}
	}

	const changeGroupBy = (columnId: string) => {
		if (!view || !draft.state) return
		draft.flush()
		updateView.mutate(
			{
				viewId: view.id,
				config: {
					...normalizeConfig(view.config),
					filters: rowFilters(draft.state.filters, columns),
					sorts: draft.state.sorts,
					hidden: draft.state.hidden,
					group_by: columnId,
				},
			},
			{ onError: fail },
		)
	}

	const confirm = async () => {
		if (!confirmation) return
		try {
			if (confirmation.kind === 'column') {
				await deleteColumn.mutateAsync(confirmation.column.id)
			} else if (confirmation.kind === 'view') {
				await deleteView.mutateAsync(confirmation.view.id)
				const next = views.find((entry) => entry.id !== confirmation.view.id)
				if (next) onViewChange(next.id)
			} else {
				await deleteRows.mutateAsync(confirmation.rowIds)
				setSelected(new Set())
				if (openRowId && confirmation.rowIds.includes(openRowId)) {
					setOpenRowId(null)
				}
			}
			setConfirmation(null)
		} catch (error) {
			setConfirmation(null)
			fail(error)
		}
	}

	const confirmText = confirmationText(confirmation)
	const isConfirming =
		deleteColumn.isPending || deleteView.isPending || deleteRows.isPending
	const notFound = detail.isError && !table

	return (
		<>
			<TablePage
				name={table?.name ?? ''}
				description={table?.description ?? null}
				canManage={canManage}
				isLoading={detail.isLoading}
				error={
					notFound
						? tableErrorMessage(detail.error)
						: tableErrorMessage(rowsQuery.error)
				}
				onBack={onBack}
				onRename={(name) =>
					updateTable.mutate({ tableId, name }, { onError: fail })
				}
				onEdit={() =>
					setTableEditor({
						name: table?.name ?? '',
						description: table?.description ?? '',
					})
				}
				tabs={
					table ? (
						<ViewTabs
							views={views}
							activeId={view?.id ?? null}
							canManage={canManage}
							onSelect={onViewChange}
							onCreate={() =>
								setViewEditor({
									mode: 'create',
									draft: { name: '', kind: 'grid', groupBy: '' },
								})
							}
							onRename={(target) =>
								setViewEditor({
									mode: 'rename',
									view: target,
									draft: {
										name: target.name,
										kind: target.kind,
										groupBy: target.config.group_by ?? '',
									},
								})
							}
							onDelete={(target) =>
								setConfirmation({ kind: 'view', view: target })
							}
						/>
					) : null
				}
				toolbar={
					table && view && draft.state ? (
						<ViewToolbar
							viewKind={view.kind}
							columns={columns}
							filters={draft.state.filters}
							activeFilterCount={draft.activeFilterCount}
							sorts={draft.state.sorts}
							hidden={hidden}
							groupBy={groupColumn?.id ?? null}
							groupable={groupable}
							canManage={canManage}
							selectedCount={selected.size}
							isExporting={exportView.isPending}
							exportUsesSavedView={exportUsesSavedView}
							choicesFor={choicesFor}
							onFiltersChange={(filters) => draft.update({ filters })}
							onSortsChange={(sorts) => draft.update({ sorts })}
							onHiddenChange={(next) => draft.update({ hidden: next })}
							onGroupByChange={changeGroupBy}
							onDeleteSelected={() =>
								setConfirmation({ kind: 'rows', rowIds: [...selected] })
							}
							onClearSelection={() => setSelected(new Set())}
							onImport={() => setRowsImport(EMPTY_ROWS_IMPORT)}
							onExport={exportCurrentView}
						/>
					) : null
				}
			>
				{!table || !view ? null : groupColumn ? (
					<KanbanBoard
						groups={groups}
						primary={primary}
						fields={kanbanFields(columns, hidden, groupColumn.id)}
						groupColumn={groupColumn}
						canManage={canManage}
						isLoading={rowsQuery.isLoading}
						onMoveRow={(rowId, optionId) =>
							changeCell(rowId, groupColumn.id, optionId)
						}
						onOpenRow={setOpenRowId}
						onAddRow={(optionId) =>
							addRow(optionId ? { [groupColumn.id]: optionId } : {}, true)
						}
					/>
				) : (
					<TableGrid
						columns={visible}
						rows={rows}
						total={total}
						widths={config?.widths ?? {}}
						sorts={draft.state?.sorts ?? []}
						canManage={canManage}
						isLoading={rowsQuery.isLoading}
						hasMore={hasNextPage}
						isLoadingMore={isFetchingNextPage}
						selected={selected}
						focusRowId={focusRowId}
						choicesFor={choicesFor}
						onLoadMore={loadMore}
						onCellChange={changeCell}
						onToggleRow={(rowId) =>
							setSelected((current) => {
								const next = new Set(current)
								if (next.has(rowId)) next.delete(rowId)
								else next.add(rowId)
								return next
							})
						}
						onToggleAll={() =>
							setSelected((current) =>
								rows.every((row) => current.has(row.id))
									? new Set()
									: new Set(rows.map((row) => row.id)),
							)
						}
						onExpandRow={setOpenRowId}
						onAddRow={() => addRow({}, false)}
						onAddColumn={() =>
							setColumnEditor({ mode: 'create', draft: EMPTY_COLUMN_DRAFT })
						}
						onColumnAction={columnAction}
						onRowFocused={() => setFocusRowId(null)}
					/>
				)}
			</TablePage>
			<RowSheet
				row={openRow}
				columns={columns}
				primary={primary}
				canManage={canManage}
				choicesFor={choicesFor}
				onChange={(columnId, value) =>
					openRow ? changeCell(openRow.id, columnId, value) : undefined
				}
				onDelete={() =>
					openRow
						? setConfirmation({ kind: 'rows', rowIds: [openRow.id] })
						: undefined
				}
				onOpenChange={(open) => {
					if (!open) setOpenRowId(null)
				}}
			/>
			<TableDialog
				open={tableEditor !== null}
				mode="rename"
				draft={tableEditor ?? { name: '', description: '' }}
				error={tableErrorMessage(updateTable.error)}
				isSaving={updateTable.isPending}
				onDraftChange={(change) => {
					updateTable.reset()
					setTableEditor((current) =>
						current ? { ...current, ...change } : current,
					)
				}}
				onSubmit={() => {
					if (!tableEditor) return
					updateTable.mutate(
						{
							tableId,
							name: tableEditor.name.trim(),
							description: tableEditor.description.trim() || null,
						},
						{ onSuccess: () => setTableEditor(null) },
					)
				}}
				onOpenChange={(open) => {
					if (!open) {
						setTableEditor(null)
						updateTable.reset()
					}
				}}
			/>
			<ColumnDialog
				open={columnEditor !== null}
				mode={columnEditor?.mode ?? 'create'}
				draft={columnEditor?.draft ?? EMPTY_COLUMN_DRAFT}
				kinds={kindChoices(
					columnEditor?.mode === 'edit' ? columnEditor.column : null,
				)}
				isPrimary={
					columnEditor?.mode === 'edit' && columnEditor.column.is_primary
				}
				error={columnError}
				isSaving={columnMutation.isPending}
				onDraftChange={(change) => {
					createColumn.reset()
					updateColumn.reset()
					setColumnEditor((current) => {
						if (!current) return current
						const next = { ...current.draft, ...change }
						const kindChanged =
							change.kind !== undefined &&
							change.kind !== current.draft.kind &&
							current.mode === 'create'
						const config = kindChanged ? defaultConfig(next.kind) : null
						return {
							...current,
							draft: config
								? {
										...next,
										decimals: config.decimals ?? next.decimals,
										linkKind: config.link_kind ?? next.linkKind,
									}
								: next,
						}
					})
				}}
				onAddOption={() =>
					setColumnEditor((current) =>
						current
							? { ...current, draft: addOption(current.draft, '') }
							: current,
					)
				}
				onSubmit={() => {
					void submitColumn()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setColumnEditor(null)
						createColumn.reset()
						updateColumn.reset()
					}
				}}
			/>
			<ViewDialog
				open={viewEditor !== null}
				mode={viewEditor?.mode ?? 'create'}
				draft={viewEditor?.draft ?? { name: '', kind: 'grid', groupBy: '' }}
				groupable={groupable}
				error={tableErrorMessage(viewMutation.error)}
				isSaving={viewMutation.isPending}
				onDraftChange={(change) => {
					createView.reset()
					updateView.reset()
					setViewEditor((current) =>
						current
							? { ...current, draft: { ...current.draft, ...change } }
							: current,
					)
				}}
				onSubmit={() => {
					void submitView()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setViewEditor(null)
						createView.reset()
						updateView.reset()
					}
				}}
			/>
			<ImportDialog
				open={rowsImport !== null}
				mode="rows"
				fileName={rowsImport?.file?.name ?? null}
				name=""
				report={rowsImport?.report ?? null}
				error={rowsImport?.refusal ?? tableErrorMessage(importRows.error)}
				isImporting={importRows.isPending}
				onFileChange={(file) => {
					importRows.reset()
					setRowsImport((current) =>
						current
							? {
									...current,
									file,
									refusal: file ? csvFileRefusal(file) : null,
								}
							: current,
					)
				}}
				onNameChange={() => {}}
				onSubmit={submitImport}
				onFinish={closeImport}
				onOpenChange={(open) => {
					if (!open) closeImport()
				}}
			/>
			<ConfirmDialog
				open={confirmation !== null}
				onOpenChange={(open) => {
					if (!open) setConfirmation(null)
				}}
				title={confirmText.title}
				description={confirmText.description}
				confirmLabel={confirmText.confirm}
				destructive
				disabled={isConfirming}
				onConfirm={() => {
					void confirm()
				}}
			/>
			<TableToast message={toast} onDismiss={() => setToast(null)} />
		</>
	)
}
