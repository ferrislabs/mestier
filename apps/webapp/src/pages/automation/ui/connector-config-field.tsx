import { Braces } from 'lucide-react'
import { useState } from 'react'
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
import { isSelectKind } from '#/pages/automation/lib/connector-config'

export interface ConnectorConfigFieldProps {
	field: Schemas.FieldResponse
	value: unknown
	error: string | null
	onChange: (value: unknown) => void
	onOpenExpression: () => void
}

export function ConnectorConfigField({
	field,
	value,
	error,
	onChange,
	onOpenExpression,
}: ConnectorConfigFieldProps) {
	const id = `connector-field-${field.name}`

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
					})}
				</div>
				{field.expression ? (
					<button
						type="button"
						aria-label={`Insérer une donnée dans ${field.label}`}
						onClick={onOpenExpression}
						className="flex size-8 shrink-0 items-center justify-center rounded-md border text-muted-foreground hover:bg-accent hover:text-accent-foreground"
					>
						<Braces className="size-4" />
					</button>
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

function renderControl({
	id,
	field,
	value,
	invalid,
	onChange,
}: {
	id: string
	field: Schemas.FieldResponse
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
			<JsonField id={id} value={value} invalid={invalid} onChange={onChange} />
		)
	}

	return (
		<Input
			id={id}
			type={field.secret ? 'password' : 'text'}
			value={typeof value === 'string' ? value : ''}
			aria-invalid={invalid}
			autoComplete="off"
			onChange={(event) => onChange(event.target.value)}
		/>
	)
}

function stringifyJsonValue(value: unknown): string {
	if (value === undefined) return ''
	if (typeof value === 'string') return value
	return JSON.stringify(value, null, 2)
}

function JsonField({
	id,
	value,
	invalid,
	onChange,
}: {
	id: string
	value: unknown
	invalid: boolean
	onChange: (value: unknown) => void
}) {
	const [text, setText] = useState(() => stringifyJsonValue(value))
	const [isInvalidJson, setInvalidJson] = useState(false)

	return (
		<div className="flex flex-col gap-1">
			<Textarea
				id={id}
				value={text}
				aria-invalid={invalid || isInvalidJson}
				className="font-mono text-xs"
				onChange={(event) => {
					const raw = event.target.value
					setText(raw)
					if (raw.trim() === '') {
						setInvalidJson(false)
						onChange(undefined)
						return
					}
					try {
						const parsed = JSON.parse(raw)
						setInvalidJson(false)
						onChange(parsed)
					} catch {
						setInvalidJson(true)
					}
				}}
			/>
			{isInvalidJson ? (
				<p className="text-xs text-destructive">JSON invalide</p>
			) : null}
		</div>
	)
}
