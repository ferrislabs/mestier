import { useState } from 'react'
import { Checkbox } from '#/components/ui/checkbox'
import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import {
	cellEditText,
	DELETED_ELEMENT,
	invalidCellMessage,
	optionIds,
	parseCellText,
	referenceId,
	referenceValue,
} from '#/pages/knowledge-tables/lib/cells'
import {
	columnOptions,
	SELECT_COLOR_CLASSES,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	CellLabel,
	Choice,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'

interface RowFieldProps {
	id: string
	column: TableColumn
	value: unknown
	label: CellLabel | null
	choices: readonly Choice[]
	onChange: (value: unknown) => void
}

function sameValue(before: unknown, after: unknown): boolean {
	return JSON.stringify(before ?? null) === JSON.stringify(after ?? null)
}

function TextField({ id, column, value, onChange }: RowFieldProps) {
	const initial =
		column.kind === 'date'
			? typeof value === 'string'
				? value
				: ''
			: cellEditText(column, value)
	const [text, setText] = useState(initial)
	const [error, setError] = useState<string | null>(null)

	const commit = () => {
		const parsed = parseCellText(column.kind, text)
		if (parsed.kind === 'invalid') {
			setError(invalidCellMessage(column.kind))
			return
		}
		const next = parsed.kind === 'value' ? parsed.value : null
		if (!sameValue(value, next)) onChange(next)
	}

	const isLong = column.kind === 'long_text'
	const shared = {
		id,
		value: text,
		'aria-invalid': error !== null,
		onBlur: commit,
	}
	return (
		<div className="grid gap-1">
			{isLong ? (
				<Textarea
					{...shared}
					rows={5}
					className="text-sm"
					onChange={(event) => {
						setText(event.target.value)
						setError(null)
					}}
				/>
			) : (
				<Input
					{...shared}
					type={column.kind === 'date' ? 'date' : 'text'}
					inputMode={
						column.kind === 'number' || column.kind === 'currency'
							? 'decimal'
							: undefined
					}
					className={cn(
						(column.kind === 'number' || column.kind === 'currency') &&
							'text-right',
					)}
					onChange={(event) => {
						setText(event.target.value)
						setError(null)
					}}
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							event.preventDefault()
							commit()
						}
					}}
				/>
			)}
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
		</div>
	)
}

function ChoiceField({ id, column, value, onChange }: RowFieldProps) {
	const options = columnOptions(column)
	if (column.kind === 'single_select') {
		return (
			<select
				id={id}
				className="h-9 w-full rounded-md border border-input bg-card px-2.5 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
				value={typeof value === 'string' ? value : ''}
				onChange={(event) => onChange(event.target.value || null)}
			>
				<option value="">Aucune</option>
				{options.map((option) => (
					<option key={option.id} value={option.id}>
						{option.label}
					</option>
				))}
			</select>
		)
	}
	const chosen = optionIds(value)
	return (
		<div id={id} className="flex flex-wrap gap-1.5">
			{options.length === 0 ? (
				<span className="text-muted-foreground text-sm">Aucune option.</span>
			) : null}
			{options.map((option) => {
				const isOn = chosen.includes(option.id)
				return (
					<button
						key={option.id}
						type="button"
						aria-pressed={isOn}
						className={cn(
							'rounded-full px-2.5 py-0.5 font-medium text-xs outline-none ring-offset-1 focus-visible:ring-2 focus-visible:ring-ring',
							SELECT_COLOR_CLASSES[option.color],
							isOn ? 'ring-2 ring-primary' : 'opacity-60',
						)}
						onClick={() => {
							const next = isOn
								? chosen.filter((entry) => entry !== option.id)
								: [...chosen, option.id]
							onChange(next.length > 0 ? next : null)
						}}
					>
						{option.label}
					</button>
				)
			})}
		</div>
	)
}

function ReferenceField({
	id,
	column,
	value,
	label,
	choices,
	onChange,
}: RowFieldProps) {
	const current = referenceId(column, value)
	const isKnown =
		current === null || choices.some((choice) => choice.id === current)
	return (
		<select
			id={id}
			className="h-9 w-full rounded-md border border-input bg-card px-2.5 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
			value={current ?? ''}
			onChange={(event) =>
				onChange(referenceValue(column, event.target.value || null))
			}
		>
			<option value="">Aucun</option>
			{isKnown ? null : (
				<option value={current ?? ''}>
					{label?.missing ? DELETED_ELEMENT : (label?.label ?? current)}
				</option>
			)}
			{choices.map((choice) => (
				<option key={choice.id} value={choice.id}>
					{choice.label}
				</option>
			))}
		</select>
	)
}

export function RowField(props: RowFieldProps) {
	switch (props.column.kind) {
		case 'checkbox':
			return (
				<Checkbox
					id={props.id}
					checked={props.value === true}
					onCheckedChange={() => props.onChange(!(props.value === true))}
				/>
			)
		case 'single_select':
		case 'multi_select':
			return <ChoiceField {...props} />
		case 'link':
		case 'file':
		case 'member':
			return <ReferenceField {...props} />
		default:
			return <TextField key={JSON.stringify(props.value ?? null)} {...props} />
	}
}
