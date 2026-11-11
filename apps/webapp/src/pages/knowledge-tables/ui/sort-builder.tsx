import { Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	isSortable,
	orderedColumns,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	RowSort,
	SortDirection,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'
import { MAX_SORTS } from '#/pages/knowledge-tables/lib/view-config'
import { NativeSelect } from '#/pages/knowledge-tables/ui/native-select'

interface SortBuilderProps {
	columns: readonly TableColumn[]
	sorts: readonly RowSort[]
	onChange: (sorts: RowSort[]) => void
}

export function SortBuilder({ columns, sorts, onChange }: SortBuilderProps) {
	const sortable = orderedColumns(columns).filter((column) =>
		isSortable(column.kind),
	)
	const used = new Set(sorts.map((sort) => sort.column_id))
	const free = sortable.filter((column) => !used.has(column.id))
	const replace = (index: number, next: RowSort) =>
		onChange(sorts.map((entry, at) => (at === index ? next : entry)))

	return (
		<div className="grid gap-3">
			{sorts.length === 0 ? (
				<p className="text-muted-foreground text-sm">
					Aucun tri : les lignes suivent leur ordre de création.
				</p>
			) : (
				<ol className="grid gap-2">
					{sorts.map((sort, index) => (
						<li
							key={sort.column_id}
							className="grid grid-cols-[auto_minmax(0,1fr)_minmax(0,9rem)_auto] items-center gap-2"
						>
							<span className="w-12 text-muted-foreground text-xs">
								{index === 0 ? 'Trier par' : 'puis par'}
							</span>
							<NativeSelect
								aria-label={`Colonne du tri ${index + 1}`}
								value={sort.column_id}
								onChange={(event) =>
									replace(index, { ...sort, column_id: event.target.value })
								}
							>
								{sortable
									.filter(
										(column) =>
											column.id === sort.column_id || !used.has(column.id),
									)
									.map((column) => (
										<option key={column.id} value={column.id}>
											{column.name}
										</option>
									))}
							</NativeSelect>
							<NativeSelect
								aria-label={`Sens du tri ${index + 1}`}
								value={sort.direction ?? 'asc'}
								onChange={(event) =>
									replace(index, {
										...sort,
										direction: event.target.value as SortDirection,
									})
								}
							>
								<option value="asc">Croissant</option>
								<option value="desc">Décroissant</option>
							</NativeSelect>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label="Retirer le tri"
								onClick={() => onChange(sorts.filter((_, at) => at !== index))}
							>
								<X aria-hidden />
							</Button>
						</li>
					))}
				</ol>
			)}
			<div className="flex items-center gap-3">
				<Button
					type="button"
					variant="outline"
					size="sm"
					disabled={sorts.length >= MAX_SORTS || free.length === 0}
					onClick={() => {
						const first = free[0]
						if (first) {
							onChange([...sorts, { column_id: first.id, direction: 'asc' }])
						}
					}}
				>
					<Plus aria-hidden />
					Ajouter un tri
				</Button>
				<span className="text-muted-foreground text-xs">
					{MAX_SORTS} tris au plus. Liens, fichiers et membres ne se trient pas.
				</span>
			</div>
		</div>
	)
}
