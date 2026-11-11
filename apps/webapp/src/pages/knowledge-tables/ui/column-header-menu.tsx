import {
	ArrowDownWideNarrow,
	ArrowUpNarrowWide,
	EyeOff,
	Pencil,
	Trash2,
} from 'lucide-react'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { isSortable, kindLabel } from '#/pages/knowledge-tables/lib/columns'
import type {
	SortDirection,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'
import { ColumnKindIcon } from '#/pages/knowledge-tables/ui/column-kind-icon'

export type ColumnAction = 'edit' | 'hide' | 'sort_asc' | 'sort_desc' | 'delete'

interface ColumnHeaderMenuProps {
	column: TableColumn
	sort: SortDirection | null
	canManage: boolean
	onAction: (action: ColumnAction) => void
}

export function ColumnHeaderMenu({
	column,
	sort,
	canManage,
	onAction,
}: ColumnHeaderMenuProps) {
	const sortable = isSortable(column.kind)
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<button
					type="button"
					aria-label={`Colonne ${column.name}, ${kindLabel(column.kind)}`}
					className="flex h-full w-full min-w-0 items-center gap-1.5 px-2 text-left font-medium text-[13px] text-muted-foreground outline-none hover:bg-muted/60 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
				>
					<ColumnKindIcon kind={column.kind} />
					<span className="min-w-0 flex-1 truncate">{column.name}</span>
					{sort === 'asc' ? (
						<ArrowUpNarrowWide
							aria-label="Tri croissant"
							className="size-3.5"
						/>
					) : sort === 'desc' ? (
						<ArrowDownWideNarrow
							aria-label="Tri décroissant"
							className="size-3.5"
						/>
					) : null}
				</button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="start" className="w-56">
				{canManage ? (
					<DropdownMenuItem onSelect={() => onAction('edit')}>
						<Pencil aria-hidden /> Modifier la colonne
					</DropdownMenuItem>
				) : null}
				{sortable ? (
					<>
						<DropdownMenuItem onSelect={() => onAction('sort_asc')}>
							<ArrowUpNarrowWide aria-hidden /> Trier croissant
						</DropdownMenuItem>
						<DropdownMenuItem onSelect={() => onAction('sort_desc')}>
							<ArrowDownWideNarrow aria-hidden /> Trier décroissant
						</DropdownMenuItem>
					</>
				) : null}
				{column.is_primary ? null : (
					<DropdownMenuItem onSelect={() => onAction('hide')}>
						<EyeOff aria-hidden /> Masquer
					</DropdownMenuItem>
				)}
				{canManage && !column.is_primary ? (
					<>
						<DropdownMenuSeparator />
						<DropdownMenuItem
							variant="destructive"
							onSelect={() => onAction('delete')}
						>
							<Trash2 aria-hidden /> Supprimer
						</DropdownMenuItem>
					</>
				) : null}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}
