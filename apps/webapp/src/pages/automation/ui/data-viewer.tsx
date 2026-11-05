import { useState } from 'react'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { formatItemCount } from '#/pages/automation/lib/format'
import { collectColumns, flattenItem } from '#/pages/automation/lib/items'
import { JsonView } from '#/pages/automation/ui/json-view'

type DataView = 'table' | 'json'

function ItemsTable({ items }: { items: readonly unknown[] }) {
	const entries = items.map((item, position) => ({
		position,
		row: flattenItem(item),
	}))
	const columns = collectColumns(entries.map((entry) => entry.row))
	return (
		<div className="overflow-hidden rounded-md border border-border bg-card">
			<Table className="text-xs">
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="h-8 w-8 px-2 text-muted-foreground">
							#
						</TableHead>
						{columns.map((column) => (
							<TableHead
								key={column}
								className="h-8 px-2 font-mono text-muted-foreground"
							>
								{column}
							</TableHead>
						))}
					</TableRow>
				</TableHeader>
				<TableBody>
					{entries.map((entry) => (
						<TableRow key={entry.position}>
							<TableCell className="tabular px-2 py-1.5 text-muted-foreground">
								{entry.position + 1}
							</TableCell>
							{columns.map((column) => (
								<TableCell
									key={column}
									className="max-w-[240px] truncate px-2 py-1.5"
									title={entry.row[column]}
								>
									{entry.row[column] ?? ''}
								</TableCell>
							))}
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}

export function DataViewer({ items }: { items: readonly unknown[] }) {
	const [view, setView] = useState<DataView>('table')
	return (
		<div className="grid gap-3">
			<div className="flex items-center justify-between gap-2">
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={view}
					onValueChange={(value) => {
						if (value === 'table' || value === 'json') setView(value)
					}}
					aria-label="Affichage des données"
				>
					<ToggleGroupItem value="table">Tableau</ToggleGroupItem>
					<ToggleGroupItem value="json">JSON</ToggleGroupItem>
				</ToggleGroup>
				<span className="text-muted-foreground text-sm">
					{formatItemCount(items.length)}
				</span>
			</div>
			{items.length === 0 ? (
				<p className="rounded-md border border-border border-dashed px-3 py-6 text-center text-muted-foreground text-sm">
					Aucun élément sur cette sortie.
				</p>
			) : view === 'table' ? (
				<ItemsTable items={items} />
			) : (
				<JsonView value={items} />
			)}
		</div>
	)
}
