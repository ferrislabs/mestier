import { ExternalLink, Loader2, Table2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	BLOCK_TOOL_BUTTON,
	BLOCK_TOOLBAR,
} from '#/pages/knowledge-documents/ui/block-toolbar'
import { DeletedElement } from '#/pages/knowledge-documents/ui/deleted-element'
import { cellLabel } from '#/pages/knowledge-tables/lib/cells'
import type { TableColumn, TableRow } from '#/pages/knowledge-tables/lib/types'
import { CellDisplay } from '#/pages/knowledge-tables/ui/cell-display'

export const TABLE_VIEW_ROW_LIMIT = 50

export type TableViewBlockState = 'empty' | 'loading' | 'ready' | 'missing'

interface TableViewBlockProps {
	state: TableViewBlockState
	tableName: string
	viewName: string
	columns: readonly TableColumn[]
	rows: readonly TableRow[]
	total: number
	isLoadingRows: boolean
	editable: boolean
	onOpen: () => void
	onPick: () => void
}

export function TableViewBlock({
	state,
	tableName,
	viewName,
	columns,
	rows,
	total,
	isLoadingRows,
	editable,
	onOpen,
	onPick,
}: TableViewBlockProps) {
	if (state === 'empty' && editable) {
		return (
			<button
				type="button"
				onClick={onPick}
				className="flex w-full items-center gap-3 rounded-lg bg-muted/50 px-4 py-3 text-left text-muted-foreground text-sm outline-none transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50"
			>
				<Table2 aria-hidden className="size-4" />
				Choisir une vue de table
			</button>
		)
	}

	if (state !== 'ready') {
		return (
			<div className="flex w-full items-center gap-3 rounded-lg bg-muted/50 px-4 py-3 text-muted-foreground text-sm">
				<Table2 aria-hidden className="size-4" />
				<span className="flex-1">
					{state === 'missing' ? (
						<DeletedElement />
					) : state === 'loading' ? (
						<Loader2 aria-hidden className="size-4 animate-spin" />
					) : (
						'Aucune vue de table choisie.'
					)}
				</span>
				{editable && state === 'missing' ? (
					<Button type="button" size="sm" variant="ghost" onClick={onPick}>
						Remplacer
					</Button>
				) : null}
			</div>
		)
	}

	const shown = rows.slice(0, TABLE_VIEW_ROW_LIMIT)
	return (
		<section
			aria-label={`Vue ${viewName} de la table ${tableName}`}
			className="group/block w-full overflow-hidden rounded-lg border bg-card"
		>
			<header className="flex items-center gap-2 border-b px-3 py-1.5 text-sm">
				<Table2 aria-hidden className="size-4 text-muted-foreground" />
				<span className="min-w-0 flex-1 truncate font-medium">
					{tableName}
					<span className="font-normal text-muted-foreground">
						{' '}
						· {viewName}
					</span>
				</span>
				{editable ? (
					<div
						contentEditable={false}
						className={cn(BLOCK_TOOLBAR, 'shadow-none')}
					>
						<button
							type="button"
							onClick={onPick}
							className={BLOCK_TOOL_BUTTON}
						>
							Changer de vue
						</button>
					</div>
				) : null}
				<Button type="button" size="sm" variant="ghost" onClick={onOpen}>
					<ExternalLink aria-hidden />
					Ouvrir la table
				</Button>
			</header>
			<div className="max-h-96 overflow-auto">
				<table className="w-full border-collapse text-sm">
					<thead className="sticky top-0 bg-muted/60 text-left">
						<tr>
							{columns.map((column) => (
								<th
									key={column.id}
									scope="col"
									className="whitespace-nowrap px-3 py-1.5 font-medium"
								>
									{column.name}
								</th>
							))}
						</tr>
					</thead>
					<tbody>
						{isLoadingRows ? (
							<tr>
								<td
									colSpan={Math.max(columns.length, 1)}
									className="px-3 py-4 text-muted-foreground"
								>
									Chargement…
								</td>
							</tr>
						) : shown.length === 0 ? (
							<tr>
								<td
									colSpan={Math.max(columns.length, 1)}
									className="px-3 py-4 text-muted-foreground"
								>
									Aucune ligne dans cette vue.
								</td>
							</tr>
						) : (
							shown.map((row) => (
								<tr key={row.id} className="border-t">
									{columns.map((column) => (
										<td
											key={column.id}
											className="max-w-64 px-3 py-1.5 align-top"
										>
											<CellDisplay
												column={column}
												value={row.values[column.id]}
												label={cellLabel(row, column.id)}
											/>
										</td>
									))}
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
			{total > shown.length ? (
				<footer className="border-t px-3 py-1.5 text-muted-foreground text-xs">
					{shown.length} premières lignes sur {total}.
				</footer>
			) : null}
		</section>
	)
}
