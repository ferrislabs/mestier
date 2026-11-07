import { Plus, Unlink, X } from 'lucide-react'
import { type ReactNode, useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { cn } from '#/lib/utils'
import {
	type FieldAddOns,
	isJsonMapRepresentable,
	isSelectKind,
	JSON_ENTRY_KINDS,
	type JsonEntry,
	type JsonEntryKind,
	jsonEntriesFromValue,
	jsonEntryValue,
	numberFieldError,
	numberFieldValue,
	parseRawJson,
	RAW_JSON_ERROR,
	valueFromJsonEntries,
} from '#/pages/automation/lib/connector-config'
import type { DisplayField } from '#/pages/automation/lib/connector-field-copy'
import { isExpression } from '#/pages/automation/lib/parameters'
import { TextArea } from '#/pages/automation/ui/text-area'

export interface ConnectorConfigFieldProps {
	field: DisplayField & FieldAddOns
	value: unknown
	issues: readonly string[]
	preview?: ReactNode
	onChange: (value: unknown) => void
}

function textOf(value: unknown): string {
	if (typeof value === 'string') return value
	return typeof value === 'number' ? String(value) : ''
}

export function ConnectorConfigField({
	field,
	value,
	issues,
	preview,
	onChange,
}: ConnectorConfigFieldProps) {
	const id = `connector-field-${field.name}`
	const expression = field.expression && isExpression(textOf(value))

	return (
		<div className="grid gap-2">
			<div className="flex items-center justify-between gap-2">
				<Label htmlFor={id}>{field.label}</Label>
				{expression && (
					<span className="rounded-sm bg-accent px-1.5 py-0.5 font-mono text-[10px] text-accent-foreground">
						Expression
					</span>
				)}
			</div>
			{renderControl({
				id,
				field,
				value,
				invalid: issues.length > 0,
				onChange,
			})}
			{expression && preview}
			{issues.map((issue) => (
				<p
					key={issue}
					role="alert"
					className="flex items-start gap-1.5 text-state-alert text-xs"
				>
					<Unlink className="mt-0.5 size-3.5 shrink-0" aria-hidden />
					{issue}
				</p>
			))}
			{field.hint && (
				<p className="text-muted-foreground text-xs">{field.hint}</p>
			)}
		</div>
	)
}

function renderControl({
	id,
	field,
	value,
	invalid,
	onChange,
}: {
	id: string
	field: DisplayField & FieldAddOns
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
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
		if (field.expression) {
			return (
				<NumberExpressionField
					id={id}
					field={field}
					value={value}
					invalid={invalid}
					onChange={onChange}
				/>
			)
		}
		return (
			<Input
				id={id}
				type="number"
				value={typeof value === 'number' ? String(value) : ''}
				placeholder={field.placeholder}
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
			/>
		)
	}

	const text = typeof value === 'string' ? value : ''
	const className = cn(
		field.expression && 'expression-drop',
		isExpression(text) && 'font-mono',
	)
	if (field.rows !== undefined) {
		return (
			<TextArea
				id={id}
				rows={field.rows}
				value={text}
				placeholder={field.placeholder}
				aria-invalid={invalid}
				className={className}
				onChange={(event) => onChange(event.target.value)}
			/>
		)
	}
	return (
		<Input
			id={id}
			type={field.secret ? 'password' : 'text'}
			value={text}
			placeholder={field.placeholder}
			aria-invalid={invalid}
			autoComplete="off"
			className={className}
			onChange={(event) => onChange(event.target.value)}
		/>
	)
}

function NumberExpressionField({
	id,
	field,
	value,
	invalid,
	onChange,
}: {
	id: string
	field: DisplayField & FieldAddOns
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
}) {
	const [draft, setDraft] = useState(() => textOf(value))
	const lastEmitted = useRef(value)

	useEffect(() => {
		if (value === lastEmitted.current) return
		lastEmitted.current = value
		setDraft(textOf(value))
	}, [value])

	const error = numberFieldError(draft)

	return (
		<>
			<Input
				id={id}
				type="text"
				inputMode="decimal"
				value={draft}
				placeholder={field.placeholder}
				aria-invalid={invalid || error !== null}
				autoComplete="off"
				className={cn(
					field.expression && 'expression-drop',
					isExpression(draft) && 'font-mono',
				)}
				onChange={(event) => {
					const raw = event.target.value
					const next = numberFieldValue(raw)
					setDraft(raw)
					lastEmitted.current = next
					onChange(next)
				}}
			/>
			{error && <p className="text-state-alert text-xs">{error}</p>}
		</>
	)
}

function stringifyJsonValue(value: unknown): string {
	if (value === undefined) return ''
	if (typeof value === 'string') return value
	return JSON.stringify(value, null, 2)
}

function sameJson(a: unknown, b: unknown): boolean {
	return JSON.stringify(a) === JSON.stringify(b)
}

type JsonMode = 'rows' | 'raw'

interface JsonRow extends JsonEntry {
	id: number
}

const ROW_COLUMNS = 'grid-cols-[minmax(0,1fr)_7.5rem_minmax(0,1.4fr)_2rem]'

function JsonField({
	id,
	field,
	value,
	invalid,
	onChange,
}: {
	id: string
	field: DisplayField & FieldAddOns
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
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
		toRows(jsonEntriesFromValue(value, field.entryKinds)),
	)
	const [rawText, setRawText] = useState(() => stringifyJsonValue(value))
	const [rawInvalid, setRawInvalid] = useState(false)
	const lastEmitted = useRef(value)

	// biome-ignore lint/correctness/useExhaustiveDependencies: toRows only closes over the id counter ref
	useEffect(() => {
		if (value === lastEmitted.current) return
		lastEmitted.current = value
		setRows(toRows(jsonEntriesFromValue(value, field.entryKinds)))
		setRawText(stringifyJsonValue(value))
		setRawInvalid(false)
		setMode(isJsonMapRepresentable(value) ? 'rows' : 'raw')
	}, [value])

	function emit(next: unknown) {
		if (sameJson(next, lastEmitted.current)) return
		lastEmitted.current = next
		onChange(next)
	}

	function commitRows(next: JsonRow[]) {
		setRows(next)
		emit(valueFromJsonEntries(next))
	}

	function addRow() {
		nextRowId.current += 1
		setRows([
			...rows,
			{ id: nextRowId.current, key: '', value: '', kind: 'string' },
		])
	}

	function updateRow(rowId: number, patch: Partial<JsonEntry>) {
		commitRows(
			rows.map((row) => (row.id === rowId ? { ...row, ...patch } : row)),
		)
	}

	function changeRaw(raw: string) {
		setRawText(raw)
		const parsed = parseRawJson(raw)
		if (parsed.state === 'invalid') {
			setRawInvalid(true)
			return
		}
		setRawInvalid(false)
		emit(parsed.state === 'empty' ? undefined : parsed.value)
	}

	const canUseRows = !rawInvalid && isJsonMapRepresentable(lastEmitted.current)

	function changeMode(next: JsonMode) {
		if (next === mode) return
		if (next === 'raw') {
			setRawText(stringifyJsonValue(lastEmitted.current))
			setRawInvalid(false)
		} else {
			if (!canUseRows) return
			setRows(
				toRows(jsonEntriesFromValue(lastEmitted.current, field.entryKinds)),
			)
		}
		setMode(next)
	}

	return (
		<fieldset aria-label={field.label} className="grid min-w-0 gap-3">
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={mode}
				onValueChange={(next) => next && changeMode(next as JsonMode)}
				aria-label={
					field.name === 'body'
						? 'Mode de saisie du corps'
						: `Mode de saisie : ${field.label}`
				}
				className="justify-self-start"
			>
				<ToggleGroupItem value="rows" disabled={mode === 'raw' && !canUseRows}>
					Champs
				</ToggleGroupItem>
				<ToggleGroupItem value="raw">JSON</ToggleGroupItem>
			</ToggleGroup>
			{mode === 'raw' && !canUseRows && (
				<p className="text-muted-foreground text-xs">
					Le mode Champs demande un objet JSON valide.
				</p>
			)}
			{mode === 'rows' ? (
				<div className="grid gap-2">
					{rows.length > 0 && (
						<div
							className={cn(
								'grid gap-2 px-0.5 text-muted-foreground text-xs',
								ROW_COLUMNS,
							)}
						>
							<span>Clé</span>
							<span>Type</span>
							<span>Valeur</span>
						</div>
					)}
					{rows.map((row, index) => {
						const { error } = jsonEntryValue(row)
						const rowLabel = row.key.trim() || `champ ${index + 1}`
						return (
							<div key={row.id} className="grid gap-1">
								<div className={cn('grid items-center gap-2', ROW_COLUMNS)}>
									<Input
										value={row.key}
										onChange={(event) =>
											updateRow(row.id, { key: event.target.value })
										}
										placeholder="clé"
										aria-label={`Clé du ${rowLabel}`}
										className="font-mono"
									/>
									<Select
										value={row.kind}
										onValueChange={(kind) =>
											updateRow(row.id, { kind: kind as JsonEntryKind })
										}
									>
										<SelectTrigger
											className="w-full"
											aria-label={`Type de ${rowLabel}`}
										>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											{JSON_ENTRY_KINDS.map((kind) => (
												<SelectItem key={kind.value} value={kind.value}>
													{kind.label}
												</SelectItem>
											))}
										</SelectContent>
									</Select>
									<Input
										value={row.value}
										onChange={(event) =>
											updateRow(row.id, { value: event.target.value })
										}
										placeholder={
											row.kind === 'boolean' ? 'true ou false' : 'valeur'
										}
										aria-label={`Valeur de ${rowLabel}`}
										aria-invalid={error ? true : undefined}
										className={cn(
											field.expression && 'expression-drop',
											(isExpression(row.value) || row.kind === 'json') &&
												'font-mono',
										)}
									/>
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										onClick={() =>
											commitRows(rows.filter((other) => other.id !== row.id))
										}
										aria-label={`Supprimer ${rowLabel}`}
									>
										<X />
									</Button>
								</div>
								{error && <p className="text-state-alert text-xs">{error}</p>}
							</div>
						)
					})}
					<Button
						type="button"
						variant="outline"
						size="sm"
						className="justify-self-start"
						onClick={addRow}
					>
						<Plus />
						Ajouter un champ
					</Button>
				</div>
			) : (
				<>
					<TextArea
						id={id}
						rows={10}
						value={rawText}
						aria-label={`${field.label} en JSON`}
						aria-invalid={invalid || rawInvalid}
						spellCheck={false}
						className={cn('font-mono', field.expression && 'expression-drop')}
						onChange={(event) => changeRaw(event.target.value)}
					/>
					{rawInvalid && (
						<p className="text-state-alert text-xs">{RAW_JSON_ERROR}</p>
					)}
				</>
			)}
		</fieldset>
	)
}
