import { Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	cellLabel,
	isEmptyValue,
	rowTitle,
} from '#/pages/knowledge-tables/lib/cells'
import { SELECT_COLOR_CLASSES } from '#/pages/knowledge-tables/lib/columns'
import type { TableColumn } from '#/pages/knowledge-tables/lib/types'
import type { KanbanGroup } from '#/pages/knowledge-tables/lib/view-config'
import { CellDisplay } from '#/pages/knowledge-tables/ui/cell-display'

interface KanbanBoardProps {
	groups: readonly KanbanGroup[]
	primary: TableColumn | null
	fields: readonly TableColumn[]
	groupColumn: TableColumn
	canManage: boolean
	isLoading: boolean
	onMoveRow: (rowId: string, optionId: string | null) => void
	onOpenRow: (rowId: string) => void
	onAddRow: (optionId: string | null) => void
}

export function KanbanBoard({
	groups,
	primary,
	fields,
	groupColumn,
	canManage,
	isLoading,
	onMoveRow,
	onOpenRow,
	onAddRow,
}: KanbanBoardProps) {
	const [dragged, setDragged] = useState<{
		rowId: string
		from: string | null
	} | null>(null)

	const drop = (target: string | null) => {
		if (dragged && dragged.from !== target) onMoveRow(dragged.rowId, target)
		setDragged(null)
	}

	return (
		<div className="min-w-0 max-w-full overflow-x-auto overscroll-x-contain pb-2">
			<div className="grid w-max auto-cols-[17rem] grid-flow-col gap-3">
				{groups.map((group) => (
					<section
						key={group.id ?? 'none'}
						aria-label={`Colonne ${group.label}`}
						className={cn(
							'flex min-h-[24rem] min-w-0 flex-col rounded-lg border border-transparent bg-muted/50 p-2 transition-colors',
							dragged !== null && 'border-border border-dashed',
						)}
						onDragOver={(event) => {
							if (canManage) event.preventDefault()
						}}
						onDrop={(event) => {
							event.preventDefault()
							drop(group.id)
						}}
					>
						<header className="flex items-center justify-between gap-2 px-1.5 pt-1 pb-2.5">
							<span
								className={cn(
									'min-w-0 truncate rounded-full px-2 py-0.5 font-medium text-xs',
									group.id === null
										? 'text-muted-foreground'
										: SELECT_COLOR_CLASSES[group.color],
								)}
							>
								{group.label}
							</span>
							<span className="flex items-center gap-1">
								<span className="text-muted-foreground text-xs tabular-nums">
									{group.rows.length}
								</span>
								{canManage ? (
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										aria-label={`Ajouter une ligne dans ${group.label}`}
										onClick={() => onAddRow(group.id)}
									>
										<Plus aria-hidden />
									</Button>
								) : null}
							</span>
						</header>
						<div className="flex flex-1 flex-col gap-2">
							{isLoading ? (
								<p className="px-2 py-6 text-center text-muted-foreground text-xs">
									Chargement…
								</p>
							) : group.rows.length === 0 ? (
								<p className="px-2 py-6 text-center text-muted-foreground text-xs">
									Aucune carte
								</p>
							) : (
								group.rows.map((row) => {
									const title = rowTitle(row, primary)
									return (
										<button
											key={row.id}
											type="button"
											draggable={canManage}
											aria-label={`${title}, ${groupColumn.name} : ${group.label}`}
											className={cn(
												'grid gap-1.5 rounded-md border bg-card p-2.5 text-left shadow-xs outline-none transition-opacity hover:border-ring/60 focus-visible:ring-[3px] focus-visible:ring-ring/50',
												dragged?.rowId === row.id && 'opacity-50',
												canManage && 'cursor-grab active:cursor-grabbing',
											)}
											onDragStart={(event) => {
												event.dataTransfer?.setData('text/plain', row.id)
												setDragged({ rowId: row.id, from: group.id })
											}}
											onDragEnd={() => setDragged(null)}
											onClick={() => onOpenRow(row.id)}
										>
											<span
												className={cn(
													'truncate font-medium text-sm',
													title === 'Sans titre' &&
														'text-muted-foreground italic',
												)}
											>
												{title}
											</span>
											{fields.map((field) =>
												isEmptyValue(row.values[field.id]) ? null : (
													<span
														key={field.id}
														className="grid grid-cols-[minmax(0,6rem)_minmax(0,1fr)] items-center gap-2 text-xs"
													>
														<span className="truncate text-muted-foreground">
															{field.name}
														</span>
														<CellDisplay
															column={field}
															value={row.values[field.id]}
															label={cellLabel(row, field.id)}
														/>
													</span>
												),
											)}
										</button>
									)
								})
							)}
						</div>
					</section>
				))}
			</div>
		</div>
	)
}
