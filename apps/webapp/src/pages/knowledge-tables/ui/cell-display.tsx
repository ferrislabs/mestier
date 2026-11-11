import { Check } from 'lucide-react'
import { cn } from '#/lib/utils'
import {
	DELETED_ELEMENT,
	displayCell,
	isEmptyValue,
	optionIds,
	UNTITLED,
} from '#/pages/knowledge-tables/lib/cells'
import { columnOptions } from '#/pages/knowledge-tables/lib/columns'
import type { CellLabel, TableColumn } from '#/pages/knowledge-tables/lib/types'
import { OptionChip } from '#/pages/knowledge-tables/ui/option-chip'

interface CellDisplayProps {
	column: TableColumn
	value: unknown
	label: CellLabel | null
}

export function CellDisplay({ column, value, label }: CellDisplayProps) {
	if (column.kind === 'checkbox') {
		const checked = value === true
		return (
			<span
				className={cn(
					'flex size-4 shrink-0 items-center justify-center rounded-[4px] border',
					checked
						? 'border-primary bg-primary text-primary-foreground'
						: 'border-input',
				)}
			>
				{checked ? <Check aria-hidden className="size-3.5" /> : null}
				<span className="sr-only">{checked ? 'Oui' : 'Non'}</span>
			</span>
		)
	}

	if (column.is_primary && isEmptyValue(value)) {
		return <span className="text-muted-foreground italic">{UNTITLED}</span>
	}

	if (column.kind === 'single_select' || column.kind === 'multi_select') {
		const options = columnOptions(column)
		const chosen = optionIds(value).flatMap((id) => {
			const option = options.find((entry) => entry.id === id)
			return option ? [option] : []
		})
		return (
			<span className="flex min-w-0 gap-1 overflow-hidden">
				{chosen.map((option) => (
					<OptionChip
						key={option.id}
						label={option.label}
						color={option.color}
					/>
				))}
			</span>
		)
	}

	const text = displayCell(column, value, label)
	const isNumeric = column.kind === 'number' || column.kind === 'currency'
	return (
		<span
			className={cn(
				'block min-w-0 truncate',
				isNumeric && 'text-right tabular-nums',
				text === DELETED_ELEMENT && 'text-muted-foreground italic',
			)}
		>
			{column.kind === 'long_text' ? text.split('\n')[0] : text}
		</span>
	)
}
