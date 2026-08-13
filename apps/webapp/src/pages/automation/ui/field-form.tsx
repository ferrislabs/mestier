import { Braces } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import type { AuthField } from '#/hooks/use-automation'

/**
 * The one place a `Field` becomes an input, for every screen that renders
 * one: a connector's config panel, a credential's data, an auth scheme. Adding
 * a connector to the backend catalogue must render here with zero change —
 * that only holds if nothing downstream special-cases a `kind`.
 *
 * The one sanctioned exception is `signing_credential_id` (issue #204's
 * documented "known gap"): its `Field` is a plain `Text` on the wire — the
 * backend's `ConnectorInput` was frozen with a single `credential_id` slot,
 * so this one lives in free-form `config` instead — but a raw UUID text box
 * would let a user point it at a credential the backend only rejects once
 * the run actually executes. `signingCredentials` (already filtered to
 * `origin: 'generated'` by the caller) turns it into a picker instead. This
 * is a field *name* exception, never a connector *kind* one: it fires
 * identically no matter which connector happens to declare that field.
 */
const SIGNING_CREDENTIAL_FIELD_NAME = 'signing_credential_id'

export interface CredentialOption {
	id: string
	name: string
}

export interface FieldFormProps {
	fields: AuthField[]
	values: Record<string, unknown>
	onChange: (name: string, value: unknown) => void
	/** Keyed by field name — the backend's structured validation error for
	 * that field, if the last save was rejected. */
	errors?: Record<string, string>
	/** Omit entirely on a form with no `expression: true` fields (e.g.
	 * credential creation) rather than passing a no-op. */
	onInsertExpression?: (fieldName: string) => void
	/** See `SIGNING_CREDENTIAL_FIELD_NAME`'s doc comment. Omit outside the
	 * connector config panel — nothing else has this field. */
	signingCredentials?: CredentialOption[]
}

export function FieldForm({
	fields,
	values,
	onChange,
	errors,
	onInsertExpression,
	signingCredentials,
}: FieldFormProps) {
	return (
		<div className="flex flex-col gap-4">
			{fields.map((field) => {
				if (field.visible_when && !isVisible(field.visible_when, values)) {
					return null
				}

				return (
					<FieldInput
						key={field.name}
						field={field}
						value={values[field.name]}
						onChange={(value) => onChange(field.name, value)}
						error={errors?.[field.name] ?? null}
						onInsertExpression={
							field.expression && onInsertExpression
								? () => onInsertExpression(field.name)
								: undefined
						}
						signingCredentials={
							field.name === SIGNING_CREDENTIAL_FIELD_NAME
								? (signingCredentials ?? [])
								: undefined
						}
					/>
				)
			})}
		</div>
	)
}

function isVisible(
	visibleWhen: NonNullable<AuthField['visible_when']>,
	values: Record<string, unknown>,
): boolean {
	const current = values[visibleWhen.field]
	return typeof current === 'string' && visibleWhen.any_of.includes(current)
}

interface FieldInputProps {
	field: AuthField
	value: unknown
	onChange: (value: unknown) => void
	error: string | null
	onInsertExpression?: () => void
	/** Present (possibly empty) only for `signing_credential_id`. */
	signingCredentials?: CredentialOption[]
}

function FieldInput({
	field,
	value,
	onChange,
	error,
	onInsertExpression,
	signingCredentials,
}: FieldInputProps) {
	const id = `field-${field.name}`

	return (
		<div className="flex flex-col gap-2">
			<div className="flex items-center justify-between gap-2">
				<span className="flex items-center gap-1">
					<Label htmlFor={id}>{field.label}</Label>
					{field.required ? null : (
						<span className="text-xs text-muted-foreground">(optionnel)</span>
					)}
				</span>
				{onInsertExpression ? (
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						title="Insérer une expression"
						onClick={onInsertExpression}
					>
						<Braces />
						<span className="sr-only">Insérer une expression</span>
					</Button>
				) : null}
			</div>

			<FieldWidget
				id={id}
				field={field}
				value={value}
				onChange={onChange}
				signingCredentials={signingCredentials}
			/>

			{error ? <p className="text-xs text-destructive">{error}</p> : null}
		</div>
	)
}

function FieldWidget({
	id,
	field,
	value,
	onChange,
	signingCredentials,
}: {
	id: string
	field: AuthField
	value: unknown
	onChange: (value: unknown) => void
	signingCredentials?: CredentialOption[]
}) {
	if (field.name === 'signing_credential_id' && signingCredentials) {
		return (
			<Select
				value={typeof value === 'string' ? value : ''}
				onValueChange={(next) => onChange(next === '' ? undefined : next)}
			>
				<SelectTrigger id={id} className="w-full">
					<SelectValue placeholder="Aucune signature" />
				</SelectTrigger>
				<SelectContent>
					{signingCredentials.map((credential) => (
						<SelectItem key={credential.id} value={credential.id}>
							{credential.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		)
	}

	const kind = field.kind

	if (typeof kind === 'object' && 'Select' in kind) {
		return (
			<Select
				value={typeof value === 'string' ? value : ''}
				onValueChange={onChange}
			>
				<SelectTrigger id={id} className="w-full">
					<SelectValue placeholder="Choisir…" />
				</SelectTrigger>
				<SelectContent>
					{kind.Select.options.map((option) => (
						<SelectItem key={option.value} value={option.value}>
							{option.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		)
	}

	if (kind === 'Bool') {
		return (
			<Switch
				id={id}
				checked={value === true}
				onCheckedChange={(checked) => onChange(checked)}
			/>
		)
	}

	if (kind === 'Number') {
		return (
			<Input
				id={id}
				type="number"
				value={typeof value === 'number' ? String(value) : ''}
				onChange={(event) => {
					const raw = event.target.value
					onChange(raw === '' ? undefined : Number(raw))
				}}
			/>
		)
	}

	if (kind === 'Json') {
		return <JsonFieldWidget id={id} value={value} onChange={onChange} />
	}

	return (
		<Input
			id={id}
			type={field.secret ? 'password' : 'text'}
			autoComplete="off"
			value={typeof value === 'string' ? value : ''}
			onChange={(event) => onChange(event.target.value)}
		/>
	)
}

/**
 * A `Json` field's wire value is already-parsed JSON (`unknown`), not text —
 * this edits it as formatted text and only ever calls `onChange` with a
 * value that parsed, so an in-progress edit can be syntactically invalid
 * without corrupting `config`. `String(value)` is deliberately never shown
 * — a Json field's value is JSON.
 */
function JsonFieldWidget({
	id,
	value,
	onChange,
}: {
	id: string
	value: unknown
	onChange: (value: unknown) => void
}) {
	const [text, setText] = useState(() => stringifyJson(value))
	const [invalid, setInvalid] = useState(false)

	return (
		<div className="flex flex-col gap-1">
			<Textarea
				id={id}
				rows={4}
				className="font-mono text-sm"
				value={text}
				onChange={(event) => {
					const next = event.target.value
					setText(next)
					if (next.trim() === '') {
						setInvalid(false)
						onChange(undefined)
						return
					}
					try {
						onChange(JSON.parse(next))
						setInvalid(false)
					} catch {
						setInvalid(true)
					}
				}}
			/>
			{invalid ? (
				<p className="text-xs text-destructive">JSON invalide</p>
			) : null}
		</div>
	)
}

function stringifyJson(value: unknown): string {
	if (value === undefined) return ''
	try {
		return JSON.stringify(value, null, 2)
	} catch {
		return ''
	}
}
