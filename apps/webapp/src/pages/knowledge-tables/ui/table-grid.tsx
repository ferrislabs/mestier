import {
	type ColumnDef,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import { useVirtualizer } from '@tanstack/react-virtual'
import { Loader2, Maximize2, Plus } from 'lucide-react'
import {
	type KeyboardEvent,
	type ReactNode,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { cn } from '#/lib/utils'
import {
	cellEditText,
	cellLabel,
	optionIds,
	referenceId,
	referenceValue,
	rowTitle,
} from '#/pages/knowledge-tables/lib/cells'
import { columnOptions } from '#/pages/knowledge-tables/lib/columns'
import {
	type CellPosition,
	moveActiveCell,
} from '#/pages/knowledge-tables/lib/grid-navigation'
import type {
	Choice,
	ColumnKind,
	RowSort,
	TableColumn,
	TableRow,
} from '#/pages/knowledge-tables/lib/types'
import { CellDisplay } from '#/pages/knowledge-tables/ui/cell-display'
import {
	ChoiceCellEditor,
	type CommitMove,
	LongTextCellEditor,
	ReferenceCellEditor,
	TextCellEditor,
} from '#/pages/knowledge-tables/ui/cell-editors'
import {
	type ColumnAction,
	ColumnHeaderMenu,
} from '#/pages/knowledge-tables/ui/column-header-menu'

const ROW_HEIGHT = 36
const GUTTER_WIDTH = 72
const DEFAULT_WIDTH = 180
const PRIMARY_WIDTH = 240
const ADD_COLUMN_WIDTH = 44
const LOAD_AHEAD = 25

const INLINE_KINDS: readonly ColumnKind[] = [
	'text',
	'number',
	'currency',
	'date',
]
const TYPEABLE_KINDS: readonly ColumnKind[] = [
	'text',
	'long_text',
	'number',
	'currency',
]

interface TableGridProps {
	columns: readonly TableColumn[]
	rows: readonly TableRow[]
	total: number
	widths: Record<string, number>
	sorts: readonly RowSort[]
	canManage: boolean
	isLoading: boolean
	hasMore: boolean
	isLoadingMore: boolean
	selected: ReadonlySet<string>
	focusRowId: string | null
	choicesFor: (column: TableColumn) => Choice[]
	onLoadMore: () => void
	onCellChange: (rowId: string, columnId: string, value: unknown) => void
	onToggleRow: (rowId: string) => void
	onToggleAll: () => void
	onExpandRow: (rowId: string) => void
	onAddRow: () => void
	onAddColumn: () => void
	onColumnAction: (column: TableColumn, action: ColumnAction) => void
	onRowFocused: () => void
}

interface EditState extends CellPosition {
	initial: string | null
}

function widthOf(column: TableColumn, widths: Record<string, number>): number {
	return (
		widths[column.id] ?? (column.is_primary ? PRIMARY_WIDTH : DEFAULT_WIDTH)
	)
}

function sameValue(before: unknown, after: unknown): boolean {
	return JSON.stringify(before ?? null) === JSON.stringify(after ?? null)
}

function moveKey(move: CommitMove): string | null {
	switch (move) {
		case 'right':
			return 'Tab'
		case 'left':
			return 'ShiftTab'
		case 'down':
			return 'ArrowDown'
		default:
			return null
	}
}

function rowCountLabel(total: number): string {
	return total === 1 ? '1 ligne' : `${total.toLocaleString('fr-FR')} lignes`
}

export function TableGrid({
	columns,
	rows,
	total,
	widths,
	sorts,
	canManage,
	isLoading,
	hasMore,
	isLoadingMore,
	selected,
	focusRowId,
	choicesFor,
	onLoadMore,
	onCellChange,
	onToggleRow,
	onToggleAll,
	onExpandRow,
	onAddRow,
	onAddColumn,
	onColumnAction,
	onRowFocused,
}: TableGridProps) {
	const scrollRef = useRef<HTMLDivElement>(null)
	const activeCellRef = useRef<HTMLButtonElement>(null)
	const wantsFocus = useRef(false)
	const [active, setActive] = useState<CellPosition | null>(null)
	const [editing, setEditing] = useState<EditState | null>(null)

	const columnDefs = useMemo<ColumnDef<TableRow>[]>(
		() =>
			columns.map((column) => ({
				id: column.id,
				header: column.name,
				size: widthOf(column, widths),
				accessorFn: (row) => row.values[column.id],
			})),
		[columns, widths],
	)
	const data = useMemo(() => [...rows], [rows])
	const table = useReactTable({
		data,
		columns: columnDefs,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.id,
	})
	const modelRows = table.getRowModel().rows
	const leafColumns = table.getAllLeafColumns()
	const primary = columns.find((column) => column.is_primary) ?? null
	const bounds = { rows: modelRows.length, columns: columns.length }
	const totalWidth =
		GUTTER_WIDTH +
		leafColumns.reduce((sum, column) => sum + column.getSize(), 0) +
		ADD_COLUMN_WIDTH

	const virtualizer = useVirtualizer({
		count: modelRows.length,
		getScrollElement: () => scrollRef.current,
		estimateSize: () => ROW_HEIGHT,
		overscan: 12,
	})
	const items = virtualizer.getVirtualItems()
	const lastIndex = items.at(-1)?.index ?? -1

	useEffect(() => {
		if (
			hasMore &&
			!isLoadingMore &&
			lastIndex >= modelRows.length - LOAD_AHEAD
		) {
			onLoadMore()
		}
	}, [hasMore, isLoadingMore, lastIndex, modelRows.length, onLoadMore])

	useEffect(() => {
		if (!focusRowId) return
		const index = modelRows.findIndex((row) => row.id === focusRowId)
		if (index === -1) return
		setActive({ row: index, column: 0 })
		virtualizer.scrollToIndex(index)
		wantsFocus.current = true
		onRowFocused()
	}, [focusRowId, modelRows, virtualizer, onRowFocused])

	useEffect(() => {
		if (wantsFocus.current && activeCellRef.current) {
			activeCellRef.current.focus({ preventScroll: true })
			wantsFocus.current = false
		}
	})

	const activate = (position: CellPosition) => {
		setActive(position)
		wantsFocus.current = true
		virtualizer.scrollToIndex(position.row)
	}

	const change = (position: CellPosition, value: unknown) => {
		const row = modelRows[position.row]?.original
		const column = columns[position.column]
		if (!row || !column) return
		if (!sameValue(row.values[column.id], value)) {
			onCellChange(row.id, column.id, value)
		}
	}

	const startEdit = (position: CellPosition, initial: string | null) => {
		if (!canManage) return
		const row = modelRows[position.row]?.original
		const column = columns[position.column]
		if (!row || !column) return
		if (column.kind === 'checkbox') {
			change(position, !(row.values[column.id] === true))
			return
		}
		setActive(position)
		setEditing({ ...position, initial })
	}

	const finishEdit = (move: CommitMove) => {
		if (!editing) return
		const key = moveKey(move)
		const next = key ? moveActiveCell(editing, key, bounds) : null
		setEditing(null)
		activate(next ?? editing)
	}

	const commit = (value: unknown, move: CommitMove) => {
		if (!editing) return
		change(editing, value)
		finishEdit(move)
	}

	const onCellKeyDown = (
		event: KeyboardEvent<HTMLButtonElement>,
		active: CellPosition,
	) => {
		if (editing) return
		const key = event.key === 'Tab' && event.shiftKey ? 'ShiftTab' : event.key
		const next = moveActiveCell(active, key, bounds)
		if (next) {
			const isTab = event.key === 'Tab'
			const isSame = next.row === active.row && next.column === active.column
			if (isTab && isSame) return
			event.preventDefault()
			activate(next)
			return
		}
		const column = columns[active.column]
		if (!column || !canManage) return
		if (event.key === 'Enter' || event.key === 'F2') {
			event.preventDefault()
			startEdit(active, null)
		} else if (event.key === ' ') {
			event.preventDefault()
			if (column.kind === 'checkbox') startEdit(active, null)
		} else if (event.key === 'Delete' || event.key === 'Backspace') {
			event.preventDefault()
			change(active, null)
		} else if (
			event.key.length === 1 &&
			!event.ctrlKey &&
			!event.metaKey &&
			!event.altKey &&
			TYPEABLE_KINDS.includes(column.kind)
		) {
			event.preventDefault()
			startEdit(active, event.key)
		}
	}

	const sortOf = (column: TableColumn) =>
		sorts.find((sort) => sort.column_id === column.id)?.direction ?? null

	const renderEditor = (
		row: TableRow,
		column: TableColumn,
		display: ReactNode,
	): ReactNode => {
		if (!editing) return display
		const value = row.values[column.id]
		if (INLINE_KINDS.includes(column.kind)) {
			const initial =
				editing.initial ??
				(column.kind === 'date'
					? typeof value === 'string'
						? value
						: ''
					: cellEditText(column, value))
			return (
				<>
					{display}
					<TextCellEditor
						kind={column.kind}
						name={column.name}
						initial={initial}
						onCommit={commit}
						onCancel={() => finishEdit('stay')}
					/>
				</>
			)
		}
		if (column.kind === 'long_text') {
			return (
				<LongTextCellEditor
					anchor={display}
					name={column.name}
					initial={editing.initial ?? cellEditText(column, value)}
					onCommit={commit}
					onCancel={() => finishEdit('stay')}
				/>
			)
		}
		if (column.kind === 'single_select' || column.kind === 'multi_select') {
			const multiple = column.kind === 'multi_select'
			return (
				<ChoiceCellEditor
					anchor={display}
					name={column.name}
					options={columnOptions(column)}
					selected={optionIds(value)}
					multiple={multiple}
					onChange={(ids) => change(editing, multiple ? ids : (ids[0] ?? null))}
					onClose={() => finishEdit('stay')}
				/>
			)
		}
		return (
			<ReferenceCellEditor
				anchor={display}
				name={column.name}
				choices={choicesFor(column)}
				selectedId={referenceId(column, value)}
				onSelect={(id) => change(editing, referenceValue(column, id))}
				onClose={() => finishEdit('stay')}
			/>
		)
	}

	const allSelected =
		rows.length > 0 && rows.every((row) => selected.has(row.id))
	const someSelected = rows.some((row) => selected.has(row.id))
	const tabStop = active ?? { row: 0, column: 0 }

	return (
		<div className="flex min-h-0 flex-col rounded-lg border bg-card">
			<div
				ref={scrollRef}
				className="relative h-[calc(100vh-17rem)] min-h-80 overflow-auto"
			>
				<table
					aria-label="Lignes de la table"
					aria-rowcount={total + 1}
					aria-colcount={columns.length}
					aria-busy={isLoading || isLoadingMore}
					className="grid text-[13px]"
					style={{ width: totalWidth }}
				>
					<thead className="sticky top-0 z-20 grid">
						<tr
							aria-rowindex={1}
							className="flex border-b bg-muted"
							style={{ height: ROW_HEIGHT }}
						>
							<th
								className="sticky left-0 z-10 flex items-center gap-1 border-r bg-muted px-2"
								style={{ width: GUTTER_WIDTH }}
							>
								{canManage ? (
									<Checkbox
										aria-label="Tout sélectionner"
										checked={
											allSelected
												? true
												: someSelected
													? 'indeterminate'
													: false
										}
										onCheckedChange={onToggleAll}
									/>
								) : (
									<span className="sr-only">Ligne</span>
								)}
							</th>
							{leafColumns.map((leaf, index) => {
								const column = columns[index]
								const sort = sortOf(column)
								return (
									<th
										key={leaf.id}
										aria-sort={
											sort === 'asc'
												? 'ascending'
												: sort === 'desc'
													? 'descending'
													: undefined
										}
										className={cn(
											'flex border-r bg-muted p-0 font-normal',
											column.is_primary && 'sticky z-10',
										)}
										style={{
											width: leaf.getSize(),
											left: column.is_primary ? GUTTER_WIDTH : undefined,
										}}
									>
										<ColumnHeaderMenu
											column={column}
											sort={sort}
											canManage={canManage}
											onAction={(action) => onColumnAction(column, action)}
										/>
									</th>
								)
							})}
							<th
								className="flex items-center justify-center"
								style={{ width: ADD_COLUMN_WIDTH }}
							>
								{canManage ? (
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										aria-label="Ajouter une colonne"
										onClick={onAddColumn}
									>
										<Plus aria-hidden />
									</Button>
								) : (
									<span className="sr-only">Colonnes</span>
								)}
							</th>
						</tr>
					</thead>
					<tbody
						className="relative grid"
						style={{ height: virtualizer.getTotalSize() }}
					>
						{items.map((item) => {
							const modelRow = modelRows[item.index]
							const row = modelRow.original
							const isSelected = selected.has(row.id)
							const title = rowTitle(row, primary)
							return (
								<tr
									key={modelRow.id}
									aria-rowindex={item.index + 2}
									aria-selected={isSelected}
									className={cn(
										'group absolute top-0 left-0 flex border-b',
										isSelected ? 'bg-primary/5' : 'bg-card',
									)}
									style={{
										height: ROW_HEIGHT,
										width: totalWidth,
										transform: `translateY(${item.start}px)`,
									}}
								>
									<td
										className={cn(
											'sticky left-0 z-10 flex items-center gap-1 border-r px-2',
											isSelected ? 'bg-accent' : 'bg-card',
										)}
										style={{ width: GUTTER_WIDTH }}
									>
										{canManage ? (
											<Checkbox
												aria-label={`Sélectionner ${title}`}
												checked={isSelected}
												tabIndex={-1}
												onCheckedChange={() => onToggleRow(row.id)}
											/>
										) : null}
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											tabIndex={-1}
											aria-label={`Ouvrir la ligne ${title}`}
											className="opacity-60 group-hover:opacity-100"
											onClick={() => onExpandRow(row.id)}
										>
											<Maximize2 aria-hidden />
										</Button>
									</td>
									{modelRow.getVisibleCells().map((cell, columnIndex) => {
										const column = columns[columnIndex]
										const position = { row: item.index, column: columnIndex }
										const isActive =
											active?.row === item.index &&
											active.column === columnIndex
										const isEditing =
											editing?.row === item.index &&
											editing.column === columnIndex
										const value = cell.getValue()
										const isTabStop =
											tabStop.row === item.index &&
											tabStop.column === columnIndex
										const display = (
											<button
												ref={isTabStop ? activeCellRef : undefined}
												type="button"
												tabIndex={isTabStop ? 0 : -1}
												className="flex h-full w-full min-w-0 items-center px-2 text-left outline-none"
												onClick={() => {
													activate(position)
													if (column.kind === 'checkbox' && canManage) {
														change(position, value !== true)
													}
												}}
												onDoubleClick={() => {
													if (!isEditing) startEdit(position, null)
												}}
												onKeyDown={(event) => onCellKeyDown(event, position)}
											>
												<span className="sr-only">{column.name} : </span>
												<CellDisplay
													column={column}
													value={value}
													label={cellLabel(row, column.id)}
												/>
											</button>
										)
										return (
											<td
												key={cell.id}
												aria-colindex={columnIndex + 1}
												aria-current={isActive ? 'true' : undefined}
												className={cn(
													'relative flex items-center border-r p-0',
													column.is_primary && 'sticky z-[5] font-medium',
													isSelected ? 'bg-accent' : 'bg-card',
													isActive &&
														'outline-2 outline-primary -outline-offset-2',
												)}
												style={{
													width: cell.column.getSize(),
													left: column.is_primary ? GUTTER_WIDTH : undefined,
												}}
											>
												{isEditing
													? renderEditor(row, column, display)
													: display}
											</td>
										)
									})}
								</tr>
							)
						})}
					</tbody>
				</table>
				{isLoading ? (
					<div className="sticky left-0 flex items-center gap-2 px-4 py-10 text-muted-foreground text-sm">
						<Loader2 aria-hidden className="size-4 animate-spin" />
						Chargement…
					</div>
				) : modelRows.length === 0 ? (
					<p className="sticky left-0 px-4 py-10 text-muted-foreground text-sm">
						Aucune ligne
						{canManage ? ' : ajoute la première ci-dessous.' : '.'}
					</p>
				) : null}
				<div
					className="sticky left-0 flex items-center gap-3 px-2"
					style={{ height: ROW_HEIGHT, width: 'min(100%, 40rem)' }}
				>
					{canManage ? (
						<Button type="button" variant="ghost" size="sm" onClick={onAddRow}>
							<Plus aria-hidden />
							Nouvelle ligne
						</Button>
					) : null}
					<span className="text-muted-foreground text-xs tabular-nums">
						{rowCountLabel(total)}
					</span>
					{isLoadingMore ? (
						<Loader2
							aria-label="Chargement de la suite"
							className="size-3.5 animate-spin text-muted-foreground"
						/>
					) : null}
				</div>
			</div>
		</div>
	)
}
