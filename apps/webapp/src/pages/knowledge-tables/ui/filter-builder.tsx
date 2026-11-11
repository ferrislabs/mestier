import { Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { optionIds } from '#/pages/knowledge-tables/lib/cells'
import {
	columnOptions,
	filterOperators,
	operatorLabel,
	operatorNeedsValue,
	orderedColumns,
	SELECT_COLOR_CLASSES,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	Choice,
	FilterDraft,
	FilterOperator,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'
import { newFilterKey } from '#/pages/knowledge-tables/lib/view-config'
import { NativeSelect } from '#/pages/knowledge-tables/ui/native-select'

interface FilterBuilderProps {
	columns: readonly TableColumn[]
	filters: readonly FilterDraft[]
	choicesFor: (column: TableColumn) => Choice[]
	onChange: (filters: FilterDraft[]) => void
}

const PLACEHOLDERS: Partial<Record<TableColumn['kind'], string>> = {
	number: '12,5',
	currency: 'En euros',
	date: 'JJ/MM/AAAA',
}

function emptyValue(operator: FilterOperator): unknown {
	if (!operatorNeedsValue(operator)) return null
	return operator === 'any_of' || operator === 'all_of' ? [] : ''
}

function freshFilter(column: TableColumn): FilterDraft {
	const operator = filterOperators(column.kind)[0]
	return {
		key: newFilterKey(),
		column_id: column.id,
		operator,
		value: emptyValue(operator),
	}
}

function ValueEditor({
	column,
	filter,
	choices,
	onValue,
}: {
	column: TableColumn
	filter: FilterDraft
	choices: Choice[]
	onValue: (value: unknown) => void
}) {
	if (!operatorNeedsValue(filter.operator)) return null
	const isList = filter.operator === 'any_of' || filter.operator === 'all_of'
	if (column.kind === 'single_select' || column.kind === 'multi_select') {
		const options = columnOptions(column)
		if (isList) {
			const chosen = optionIds(filter.value)
			return (
				<fieldset className="flex flex-wrap gap-1" aria-label="Valeurs">
					{options.map((option) => {
						const isOn = chosen.includes(option.id)
						return (
							<button
								key={option.id}
								type="button"
								aria-pressed={isOn}
								className={cn(
									'rounded-full px-2 py-0.5 font-medium text-xs outline-none ring-offset-1 focus-visible:ring-2 focus-visible:ring-ring',
									SELECT_COLOR_CLASSES[option.color],
									isOn ? 'ring-2 ring-primary' : 'opacity-60',
								)}
								onClick={() =>
									onValue(
										isOn
											? chosen.filter((id) => id !== option.id)
											: [...chosen, option.id],
									)
								}
							>
								{option.label}
							</button>
						)
					})}
				</fieldset>
			)
		}
		return (
			<NativeSelect
				aria-label="Valeur"
				value={typeof filter.value === 'string' ? filter.value : ''}
				onChange={(event) => onValue(event.target.value)}
			>
				<option value="">Choisir…</option>
				{options.map((option) => (
					<option key={option.id} value={option.id}>
						{option.label}
					</option>
				))}
			</NativeSelect>
		)
	}
	if (
		column.kind === 'link' ||
		column.kind === 'file' ||
		column.kind === 'member'
	) {
		return (
			<NativeSelect
				aria-label="Valeur"
				value={typeof filter.value === 'string' ? filter.value : ''}
				onChange={(event) => onValue(event.target.value)}
			>
				<option value="">Choisir…</option>
				{choices.map((choice) => (
					<option key={choice.id} value={choice.id}>
						{choice.label}
					</option>
				))}
			</NativeSelect>
		)
	}
	return (
		<Input
			aria-label="Valeur"
			className="h-8 text-[13px]"
			value={typeof filter.value === 'string' ? filter.value : ''}
			placeholder={PLACEHOLDERS[column.kind] ?? 'Valeur'}
			inputMode={
				column.kind === 'number' || column.kind === 'currency'
					? 'decimal'
					: undefined
			}
			onChange={(event) => onValue(event.target.value)}
		/>
	)
}

export function FilterBuilder({
	columns,
	filters,
	choicesFor,
	onChange,
}: FilterBuilderProps) {
	const ordered = orderedColumns(columns)
	const replace = (index: number, next: FilterDraft) =>
		onChange(filters.map((entry, at) => (at === index ? next : entry)))

	return (
		<div className="grid gap-3">
			{filters.length === 0 ? (
				<p className="text-muted-foreground text-sm">
					Aucun filtre : toutes les lignes sont affichées.
				</p>
			) : (
				<ul className="grid gap-2">
					{filters.map((filter, index) => {
						const column = ordered.find(
							(entry) => entry.id === filter.column_id,
						)
						if (!column) return null
						return (
							<li
								key={filter.key}
								className="grid grid-cols-[minmax(0,9rem)_minmax(0,8rem)_minmax(0,1fr)_auto] items-start gap-2"
							>
								<NativeSelect
									aria-label="Colonne"
									value={column.id}
									onChange={(event) => {
										const next = ordered.find(
											(entry) => entry.id === event.target.value,
										)
										if (next)
											replace(index, { ...freshFilter(next), key: filter.key })
									}}
								>
									{ordered.map((entry) => (
										<option key={entry.id} value={entry.id}>
											{entry.name}
										</option>
									))}
								</NativeSelect>
								<NativeSelect
									aria-label="Opérateur"
									value={filter.operator}
									onChange={(event) => {
										const operator = event.target.value as FilterOperator
										const keepsValue =
											operatorNeedsValue(operator) &&
											operatorNeedsValue(filter.operator) &&
											Array.isArray(filter.value) ===
												Array.isArray(emptyValue(operator))
										replace(index, {
											...filter,
											operator,
											value: keepsValue ? filter.value : emptyValue(operator),
										})
									}}
								>
									{filterOperators(column.kind).map((operator) => (
										<option key={operator} value={operator}>
											{operatorLabel(column.kind, operator)}
										</option>
									))}
								</NativeSelect>
								<div className="min-w-0">
									<ValueEditor
										column={column}
										filter={filter}
										choices={choicesFor(column)}
										onValue={(value) => replace(index, { ...filter, value })}
									/>
								</div>
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									aria-label="Retirer le filtre"
									onClick={() =>
										onChange(filters.filter((_, at) => at !== index))
									}
								>
									<X aria-hidden />
								</Button>
							</li>
						)
					})}
				</ul>
			)}
			<div>
				<Button
					type="button"
					variant="outline"
					size="sm"
					disabled={ordered.length === 0}
					onClick={() => {
						const first = ordered[0]
						if (first) onChange([...filters, freshFilter(first)])
					}}
				>
					<Plus aria-hidden />
					Ajouter un filtre
				</Button>
			</div>
		</div>
	)
}
