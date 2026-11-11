import {
	ArrowUpDown,
	Columns3,
	Download,
	Filter,
	Trash2,
	Upload,
	X,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { orderedColumns } from '#/pages/knowledge-tables/lib/columns'
import type {
	Choice,
	FilterDraft,
	RowSort,
	TableColumn,
	ViewKind,
} from '#/pages/knowledge-tables/lib/types'
import { ColumnKindIcon } from '#/pages/knowledge-tables/ui/column-kind-icon'
import { FilterBuilder } from '#/pages/knowledge-tables/ui/filter-builder'
import { NativeSelect } from '#/pages/knowledge-tables/ui/native-select'
import { SortBuilder } from '#/pages/knowledge-tables/ui/sort-builder'

const SOON = 'Bientôt disponible'

interface ViewToolbarProps {
	viewKind: ViewKind
	columns: readonly TableColumn[]
	filters: readonly FilterDraft[]
	activeFilterCount: number
	sorts: readonly RowSort[]
	hidden: readonly string[]
	groupBy: string | null
	groupable: readonly TableColumn[]
	canManage: boolean
	selectedCount: number
	choicesFor: (column: TableColumn) => Choice[]
	onFiltersChange: (filters: FilterDraft[]) => void
	onSortsChange: (sorts: RowSort[]) => void
	onHiddenChange: (hidden: string[]) => void
	onGroupByChange: (columnId: string) => void
	onDeleteSelected: () => void
	onClearSelection: () => void
}

function ToolbarPopover({
	icon,
	label,
	count,
	width,
	children,
}: {
	icon: ReactNode
	label: string
	count: number
	width: string
	children: ReactNode
}) {
	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant={count > 0 ? 'secondary' : 'ghost'}
					size="sm"
				>
					{icon}
					{label}
					{count > 0 ? (
						<span className="rounded-full bg-primary px-1.5 text-[11px] text-primary-foreground tabular-nums">
							{count}
						</span>
					) : null}
				</Button>
			</PopoverTrigger>
			<PopoverContent align="start" className={width}>
				{children}
			</PopoverContent>
		</Popover>
	)
}

export function ViewToolbar({
	viewKind,
	columns,
	filters,
	activeFilterCount,
	sorts,
	hidden,
	groupBy,
	groupable,
	canManage,
	selectedCount,
	choicesFor,
	onFiltersChange,
	onSortsChange,
	onHiddenChange,
	onGroupByChange,
	onDeleteSelected,
	onClearSelection,
}: ViewToolbarProps) {
	const hideable = orderedColumns(columns).filter(
		(column) => !column.is_primary,
	)
	const hiddenCount = hideable.filter((column) =>
		hidden.includes(column.id),
	).length

	return (
		<div
			role="toolbar"
			aria-label="Outils de la vue"
			className="flex flex-wrap items-center gap-1.5"
		>
			<ToolbarPopover
				icon={<Filter aria-hidden />}
				label="Filtrer"
				count={activeFilterCount}
				width="w-[36rem] max-w-[calc(100vw-2rem)]"
			>
				<FilterBuilder
					columns={columns}
					filters={filters}
					choicesFor={choicesFor}
					onChange={onFiltersChange}
				/>
			</ToolbarPopover>
			<ToolbarPopover
				icon={<ArrowUpDown aria-hidden />}
				label="Trier"
				count={sorts.length}
				width="w-[30rem] max-w-[calc(100vw-2rem)]"
			>
				<SortBuilder columns={columns} sorts={sorts} onChange={onSortsChange} />
			</ToolbarPopover>
			<ToolbarPopover
				icon={<Columns3 aria-hidden />}
				label="Colonnes"
				count={hiddenCount}
				width="w-64"
			>
				{hideable.length === 0 ? (
					<p className="text-muted-foreground text-sm">
						La colonne principale reste toujours visible.
					</p>
				) : (
					<ul className="grid gap-1.5">
						{hideable.map((column) => {
							const id = `column-visible-${column.id}`
							return (
								<li key={column.id} className="flex items-center gap-2">
									<Checkbox
										id={id}
										checked={!hidden.includes(column.id)}
										onCheckedChange={(checked) =>
											onHiddenChange(
												checked === true
													? hidden.filter((entry) => entry !== column.id)
													: [...hidden, column.id],
											)
										}
									/>
									<Label
										htmlFor={id}
										className="flex min-w-0 items-center gap-1.5 font-normal"
									>
										<ColumnKindIcon
											kind={column.kind}
											className="size-3.5 shrink-0 text-muted-foreground"
										/>
										<span className="truncate">{column.name}</span>
									</Label>
								</li>
							)
						})}
					</ul>
				)}
			</ToolbarPopover>
			{viewKind === 'kanban' ? (
				<div className="flex items-center gap-2">
					<Label
						htmlFor="kanban-group-by"
						className="font-normal text-muted-foreground text-xs"
					>
						Grouper par
					</Label>
					<NativeSelect
						id="kanban-group-by"
						className="w-40"
						value={groupBy ?? ''}
						disabled={!canManage || groupable.length === 0}
						onChange={(event) => onGroupByChange(event.target.value)}
					>
						{groupable.map((column) => (
							<option key={column.id} value={column.id}>
								{column.name}
							</option>
						))}
					</NativeSelect>
				</div>
			) : null}
			<span className="flex-1" />
			{selectedCount > 0 ? (
				<div className="flex items-center gap-1.5 rounded-md border bg-muted/40 px-2 py-0.5">
					<span className="text-sm">
						{selectedCount === 1
							? '1 ligne sélectionnée'
							: `${selectedCount} lignes sélectionnées`}
					</span>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="text-destructive"
						onClick={onDeleteSelected}
					>
						<Trash2 aria-hidden />
						Supprimer
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						aria-label="Annuler la sélection"
						onClick={onClearSelection}
					>
						<X aria-hidden />
					</Button>
				</div>
			) : null}
			{canManage ? (
				<span title={SOON}>
					<Button type="button" variant="ghost" size="sm" disabled>
						<Upload aria-hidden />
						Importer
					</Button>
				</span>
			) : null}
			<span title={SOON}>
				<Button type="button" variant="ghost" size="sm" disabled>
					<Download aria-hidden />
					Exporter
				</Button>
			</span>
		</div>
	)
}
