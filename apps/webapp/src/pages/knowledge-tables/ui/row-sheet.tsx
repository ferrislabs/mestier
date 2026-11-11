import { Trash2 } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { cellLabel, rowTitle } from '#/pages/knowledge-tables/lib/cells'
import { kindLabel } from '#/pages/knowledge-tables/lib/columns'
import type {
	Choice,
	TableColumn,
	TableRow,
} from '#/pages/knowledge-tables/lib/types'
import { CellDisplay } from '#/pages/knowledge-tables/ui/cell-display'
import { ColumnKindIcon } from '#/pages/knowledge-tables/ui/column-kind-icon'
import { RowField } from '#/pages/knowledge-tables/ui/row-fields'

interface RowSheetProps {
	row: TableRow | null
	columns: readonly TableColumn[]
	primary: TableColumn | null
	canManage: boolean
	choicesFor: (column: TableColumn) => Choice[]
	onChange: (columnId: string, value: unknown) => void
	onDelete: () => void
	onOpenChange: (open: boolean) => void
}

function formatStamp(iso: string): string {
	return new Date(iso).toLocaleString('fr-FR', {
		dateStyle: 'short',
		timeStyle: 'short',
	})
}

export function RowSheet({
	row,
	columns,
	primary,
	canManage,
	choicesFor,
	onChange,
	onDelete,
	onOpenChange,
}: RowSheetProps) {
	const baseId = useId()
	return (
		<Sheet open={row !== null} onOpenChange={onOpenChange}>
			<SheetContent className="w-full overflow-y-auto sm:max-w-lg">
				{row ? (
					<>
						<SheetHeader>
							<SheetTitle>{rowTitle(row, primary)}</SheetTitle>
							<SheetDescription>
								Créée le {formatStamp(row.created_at)}, modifiée le{' '}
								{formatStamp(row.updated_at)}.
							</SheetDescription>
						</SheetHeader>
						<div className="grid gap-4 px-4">
							{columns.map((column) => {
								const id = `${baseId}-${column.id}`
								const value = row.values[column.id]
								return (
									<div key={column.id} className="grid gap-1.5">
										<Label
											htmlFor={id}
											className="flex items-center gap-1.5 text-muted-foreground"
										>
											<ColumnKindIcon kind={column.kind} />
											{column.name}
											<span className="sr-only">
												, {kindLabel(column.kind)}
											</span>
										</Label>
										{canManage ? (
											<RowField
												id={id}
												column={column}
												value={value}
												label={cellLabel(row, column.id)}
												choices={choicesFor(column)}
												onChange={(next) => onChange(column.id, next)}
											/>
										) : (
											<div id={id} className="min-h-6 text-sm">
												<CellDisplay
													column={column}
													value={value}
													label={cellLabel(row, column.id)}
												/>
											</div>
										)}
									</div>
								)
							})}
						</div>
						{canManage ? (
							<SheetFooter>
								<Button
									type="button"
									variant="outline"
									className="text-destructive"
									onClick={onDelete}
								>
									<Trash2 aria-hidden />
									Supprimer la ligne
								</Button>
							</SheetFooter>
						) : null}
					</>
				) : null}
			</SheetContent>
		</Sheet>
	)
}
