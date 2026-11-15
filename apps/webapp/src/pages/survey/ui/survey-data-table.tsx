import { flexRender, type Table as TableInstance } from '@tanstack/react-table'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'

export function SurveyDataTable<T>({
	table,
	label,
	onRowClick,
}: {
	table: TableInstance<T>
	label: string
	onRowClick?: (row: T) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table aria-label={label}>
				<TableHeader>
					{table.getHeaderGroups().map((group) => (
						<TableRow key={group.id} className="hover:bg-transparent">
							{group.headers.map((header) => (
								<TableHead key={header.id} className="first:pl-4 last:pr-4">
									{header.isPlaceholder
										? null
										: flexRender(
												header.column.columnDef.header,
												header.getContext(),
											)}
								</TableHead>
							))}
						</TableRow>
					))}
				</TableHeader>
				<TableBody>
					{table.getRowModel().rows.map((row) => (
						<TableRow
							key={row.id}
							className={onRowClick ? 'cursor-pointer' : undefined}
							onClick={onRowClick ? () => onRowClick(row.original) : undefined}
						>
							{row.getVisibleCells().map((cell) => (
								<TableCell
									key={cell.id}
									className="py-1.5 text-sm first:pl-4 last:pr-4"
								>
									{flexRender(cell.column.columnDef.cell, cell.getContext())}
								</TableCell>
							))}
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}
