import { Plus, Trash2, Variable } from 'lucide-react'
import { type DragEvent, useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Checkbox } from '#/components/ui/checkbox'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'
import {
	isJsonMapRepresentable,
	isSelectKind,
	type JsonEntry,
	jsonEntriesFromValue,
	nextJsonEntryKey,
	valueFromJsonEntries,
} from '#/pages/automation/lib/connector-config'
import { insertExpressionAtCursor } from '#/pages/automation/lib/expression'

type FieldControlElement = HTMLInputElement | HTMLTextAreaElement

export interface ConnectorConfigFieldProps {
	field: Schemas.FieldResponse
	value: unknown
	error: string | null
	onChange: (value: unknown) => void
	onOpenExpression: () => void
	controlRef?: (element: FieldControlElement | null) => void
	onDropExpression?: (path: string) => void
}

export function ConnectorConfigField({
	field,
	value,
	error,
	onChange,
	onOpenExpression,
	controlRef,
	onDropExpression,
}: ConnectorConfigFieldProps) {
	const id = `connector-field-${field.name}`
	const showFieldLevelExpressionButton = field.expression && field.kind !== 'Json'

	return (
		<Field label={field.label} htmlFor={id}>
			<div className="flex items-center gap-2">
				<div className="min-w-0 flex-1">
					{renderControl({
						id,
						field,
						value,
						invalid: error !== null,
						onChange,
						controlRef,
						onDropExpression: field.expression ? onDropExpression : undefined,
						onOpenExpression,
					})}
				</div>
				{showFieldLevelExpressionButton ? (
					<ExpressionButton label={field.label} onClick={onOpenExpression} />
				) : null}
			</div>
			{error ? (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			) : null}
		</Field>
	)
}

function ExpressionButton({
	label,
	onClick,
}: {
	label: string
	onClick: () => void
}) {
	return (
		<button
			type="button"
			aria-label={`Insérer une donnée dans ${label}`}
			title="Insérer une donnée disponible (clic pour parcourir, ou glisser-déposer)"
			onClick={onClick}
			className="flex h-8 shrink-0 items-center gap-1 rounded-md border px-2 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground"
		>
			<Variable className="size-3.5" />
			Donnée
		</button>
	)
}

function dropHandlers(onDropExpression?: (path: string) => void) {
	if (!onDropExpression) return {}
	return {
		onDragOver: (event: DragEvent<FieldControlElement>) =>
			event.preventDefault(),
		onDrop: (event: DragEvent<FieldControlElement>) => {
			event.preventDefault()
			onDropExpression(event.dataTransfer.getData('text/plain'))
		},
	}
}

function renderControl({
	id,
	field,
	value,
	invalid,
	onChange,
	controlRef,
	onDropExpression,
	onOpenExpression,
}: {
	id: string
	field: Schemas.FieldResponse
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
	controlRef?: (element: FieldControlElement | null) => void
	onDropExpression?: (path: string) => void
	onOpenExpression: () => void
}) {
	if (isSelectKind(field.kind)) {
		return (
			<Select
				value={typeof value === 'string' ? value : ''}
				onValueChange={(next) => onChange(next)}
			>
				<SelectTrigger id={id} className="w-full" aria-invalid={invalid}>
					<SelectValue placeholder="Choisir…" />
				</SelectTrigger>
				<SelectContent>
					{field.kind.Select.options.map((option) => (
						<SelectItem key={option.value} value={option.value}>
							{option.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		)
	}

	if (field.kind === 'Bool') {
		return (
			<Checkbox
				id={id}
				checked={value === true}
				aria-invalid={invalid}
				onCheckedChange={(checked) => onChange(checked === true)}
			/>
		)
	}

	if (field.kind === 'Number') {
		return (
			<Input
				id={id}
				type="number"
				value={typeof value === 'number' ? String(value) : ''}
				aria-invalid={invalid}
				onChange={(event) => {
					const raw = event.target.value
					onChange(raw === '' ? undefined : Number(raw))
				}}
			/>
		)
	}

	if (field.kind === 'Json') {
		return (
			<JsonField
				id={id}
				field={field}
				value={value}
				invalid={invalid}
				onChange={onChange}
				controlRef={controlRef}
				onDropExpression={onDropExpression}
				onOpenExpression={onOpenExpression}
			/>
		)
	}

	return (
		<Input
			id={id}
			ref={controlRef}
			type={field.secret ? 'password' : 'text'}
			value={typeof value === 'string' ? value : ''}
			aria-invalid={invalid}
			autoComplete="off"
			onChange={(event) => onChange(event.target.value)}
			{...dropHandlers(onDropExpression)}
		/>
	)
}

function stringifyJsonValue(value: unknown): string {
	if (value === undefined) return ''
	if (typeof value === 'string') return value
	return JSON.stringify(value, null, 2)
}

type JsonMode = 'rows' | 'raw'

interface JsonRow extends JsonEntry {
	id: number
}

function JsonField({
	id,
	field,
	value,
	invalid,
	onChange,
	controlRef,
	onDropExpression,
	onOpenExpression,
}: {
	id: string
	field: Schemas.FieldResponse
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
	controlRef?: (element: FieldControlElement | null) => void
	onDropExpression?: (path: string) => void
	onOpenExpression: () => void
}) {
	const nextRowId = useRef(0)
	function toRows(entries: JsonEntry[]): JsonRow[] {
		return entries.map((entry) => {
			nextRowId.current += 1
			return { ...entry, id: nextRowId.current }
		})
	}

	const [mode, setMode] = useState<JsonMode>(
		isJsonMapRepresentable(value) ? 'rows' : 'raw',
	)
	const [rows, setRows] = useState<JsonRow[]>(() =>
		toRows(jsonEntriesFromValue(value)),
	)
	const [rawText, setRawText] = useState(() => stringifyJsonValue(value))
	const [rawInvalid, setRawInvalid] = useState(false)
	const lastEmitted = useRef(value)

	useEffect(() => {
		if (value === lastEmitted.current) return
		lastEmitted.current = value
		setRows(toRows(jsonEntriesFromValue(value)))
		setRawText(stringifyJsonValue(value))
		setRawInvalid(false)
		setMode(isJsonMapRepresentable(value) ? 'rows' : 'raw')
		// biome-ignore lint/correctness/useExhaustiveDependencies: toRows is stable across renders
	}, [value])

	function emit(next: unknown) {
		lastEmitted.current = next
		onChange(next)
	}

	function commitRows(next: JsonRow[]) {
		setRows(next)
		emit(valueFromJsonEntries(next))
	}

	function addRow() {
		const key = nextJsonEntryKey(rows)
		nextRowId.current += 1
		commitRows([...rows, { id: nextRowId.current, key, value: 'value' }])
	}

	function removeRow(rowId: number) {
		commitRows(rows.filter((row) => row.id !== rowId))
	}

	function changeRowKey(rowId: number, key: string) {
		commitRows(
			rows.map((row) => (row.id === rowId ? { ...row, key } : row)),
		)
	}

	function changeRowValue(rowId: number, entryValue: string) {
		commitRows(
			rows.map((row) => (row.id === rowId ? { ...row, value: entryValue } : row)),
		)
	}

	function dropOnRow(
		rowId: number,
		element: HTMLInputElement,
		path: string,
	) {
		const row = rows.find((candidate) => candidate.id === rowId)
		if (!row) return
		const selection = {
			start: element.selectionStart ?? row.value.length,
			end: element.selectionEnd ?? row.value.length,
		}
		const result = insertExpressionAtCursor(row.value, path, selection)
		changeRowValue(rowId, result.text)
	}

	const representable = isJsonMapRepresentable(lastEmitted.current)

	function switchToRaw() {
		setRawText(stringifyJsonValue(value))
		setRawInvalid(false)
		setMode('raw')
	}

	function switchToRows() {
		setRows(toRows(jsonEntriesFromValue(value)))
		setMode('rows')
	}

	if (mode === 'rows') {
		return (
			<div className="flex flex-col gap-2">
				{rows.length === 0 ? (
					<p className="rounded-md border border-dashed px-3 py-2 text-xs text-muted-foreground">
						Aucune entrée. Ajoutez-en une pour commencer.
					</p>
				) : (
					<ul className="flex flex-col gap-1.5">
						{rows.map((row) => (
							<li key={row.id} className="flex items-center gap-1.5">
								<Input
									aria-label={`Clé — ${field.label}`}
									value={row.key}
									onChange={(event) =>
										changeRowKey(row.id, event.target.value)
									}
									className="w-2/5 font-mono text-xs"
								/>
								<Input
									aria-label={`Valeur — ${field.label}`}
									value={row.value}
									onChange={(event) =>
										changeRowValue(row.id, event.target.value)
									}
									className="flex-1 font-mono text-xs"
									{...(field.expression
										? {
												onDragOver: (event: DragEvent<HTMLInputElement>) =>
													event.preventDefault(),
												onDrop: (event: DragEvent<HTMLInputElement>) => {
													event.preventDefault()
													dropOnRow(
														row.id,
														event.currentTarget,
														event.dataTransfer.getData('text/plain'),
													)
												},
											}
										: {})}
								/>
								<button
									type="button"
									aria-label={`Supprimer l'entrée ${row.key || row.id}`}
									onClick={() => removeRow(row.id)}
									className="flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground"
								>
									<Trash2 className="size-3.5" />
								</button>
							</li>
						))}
					</ul>
				)}
				<div className="flex items-center gap-3">
					<button
						type="button"
						onClick={addRow}
						className="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground"
					>
						<Plus className="size-3.5" />
						Ajouter une entrée
					</button>
					<button
						type="button"
						onClick={switchToRaw}
						className="text-xs text-muted-foreground underline-offset-2 hover:underline"
					>
						Passer en JSON
					</button>
				</div>
			</div>
		)
	}

	return (
		<div className="flex flex-col gap-1">
			<div className="flex items-center gap-2">
				<Textarea
					id={id}
					ref={controlRef}
					value={rawText}
					aria-invalid={invalid || rawInvalid}
					className="font-mono text-xs"
					onChange={(event) => {
						const raw = event.target.value
						setRawText(raw)
						if (raw.trim() === '') {
							setRawInvalid(false)
							emit(undefined)
							return
						}
						try {
							const parsed = JSON.parse(raw)
							setRawInvalid(false)
							emit(parsed)
						} catch {
							setRawInvalid(true)
						}
					}}
					{...dropHandlers(onDropExpression)}
				/>
				{field.expression ? (
					<ExpressionButton label={field.label} onClick={onOpenExpression} />
				) : null}
			</div>
			{rawInvalid ? (
				<p className="text-xs text-destructive">JSON invalide</p>
			) : null}
			{representable ? (
				<button
					type="button"
					onClick={switchToRows}
					className="self-start text-xs text-muted-foreground underline-offset-2 hover:underline"
				>
					Revenir aux champs
				</button>
			) : null}
		</div>
	)
}
