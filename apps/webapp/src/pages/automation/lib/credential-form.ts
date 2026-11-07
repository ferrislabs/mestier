import type { CredentialField, CredentialKind } from '#/hooks/use-automation'
import type { CredentialRow } from '#/pages/automation/lib/credential-rows'
import type { CredentialFormValues } from '#/pages/automation/types'

export type CredentialFieldType =
	| 'text'
	| 'password'
	| 'textarea'
	| 'select'
	| 'number'
	| 'boolean'
	| 'json'

const FIELD_TYPES: readonly string[] = [
	'text',
	'password',
	'textarea',
	'select',
	'number',
	'boolean',
	'json',
]

export function fieldType(field: CredentialField): CredentialFieldType {
	return FIELD_TYPES.includes(field.kind)
		? (field.kind as CredentialFieldType)
		: 'text'
}

export function isMultiline(field: CredentialField): boolean {
	const type = fieldType(field)
	return type === 'textarea' || type === 'json'
}

export function emptyCredentialForm(defaultKind = ''): CredentialFormValues {
	return {
		kind: defaultKind,
		name: '',
		description: '',
		origin: 'supplied',
		data: {},
	}
}

export function visibleCredentialFields(
	fields: readonly CredentialField[],
	data: Readonly<Record<string, string>>,
): CredentialField[] {
	return fields.filter((field) => {
		if (!field.show_when) return true
		const rule = field.show_when
		const controller = fields.find((candidate) => candidate.name === rule.field)
		const current = data[rule.field] ?? controller?.default ?? ''
		return rule.one_of.includes(current)
	})
}

export function credentialFormDefaults(
	kind: Pick<CredentialKind, 'fields'> | undefined,
): Record<string, string> {
	return Object.fromEntries(
		(kind?.fields ?? [])
			.filter((field) => !field.secret && typeof field.default === 'string')
			.map((field) => [field.name, field.default ?? '']),
	)
}

export function credentialFormFromRow(
	row: CredentialRow,
	kind: Pick<CredentialKind, 'fields'> | undefined,
): CredentialFormValues {
	const stored = Object.fromEntries(
		(kind?.fields ?? [])
			.filter((field) => !field.secret && row.data[field.name] != null)
			.map((field) => [field.name, String(row.data[field.name])]),
	)
	return {
		kind: row.kind,
		name: row.name,
		description: row.description,
		origin: row.origin,
		data: { ...credentialFormDefaults(kind), ...stored },
	}
}

function submittedValue(field: CredentialField, raw: string): unknown {
	const type = fieldType(field)
	if (type === 'boolean') return raw === 'true'
	if (type === 'number' && /^\d+$/.test(raw.trim())) return Number(raw.trim())
	return field.secret ? raw : raw.trim()
}

export function credentialPayload(
	kind: Pick<CredentialKind, 'fields'>,
	data: Readonly<Record<string, string>>,
	initial?: Readonly<Record<string, string>>,
): Record<string, unknown> {
	const entries: [string, unknown][] = []
	for (const field of visibleCredentialFields(kind.fields, data)) {
		const raw = data[field.name] ?? ''
		if (field.secret) {
			if (raw !== '') entries.push([field.name, submittedValue(field, raw)])
			continue
		}
		const value = submittedValue(field, raw)
		if (initial) {
			if (raw.trim() === (initial[field.name] ?? '').trim()) continue
		} else if (value === '') continue
		entries.push([field.name, value])
	}
	return Object.fromEntries(entries)
}

export type CredentialFieldErrors = Readonly<Record<string, string>>

function isJsonObject(text: string): boolean {
	try {
		const parsed: unknown = JSON.parse(text)
		return (
			typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)
		)
	} catch {
		return false
	}
}

export function buildCredentialFieldErrors(
	values: CredentialFormValues,
	kind: Pick<CredentialKind, 'fields'> | undefined,
	storedSecrets: readonly string[] = [],
): CredentialFieldErrors {
	const errors: Record<string, string> = {}
	if (values.name.trim() === '')
		errors.name = 'Donnez un nom à cette identification.'

	if (values.origin !== 'supplied' || !kind) return errors

	for (const field of visibleCredentialFields(kind.fields, values.data)) {
		const type = fieldType(field)
		const typed = (values.data[field.name] ?? '').trim()
		if (field.secret && storedSecrets.includes(field.name) && typed === '')
			continue
		if (type === 'boolean') continue
		if (field.required && typed === '')
			errors[field.name] = 'Champ obligatoire.'
		else if (type === 'number' && typed !== '' && !/^\d+$/.test(typed))
			errors[field.name] = 'Nombre entier attendu.'
		else if (type === 'json' && typed !== '' && !isJsonObject(typed))
			errors[field.name] = 'JSON invalide : un objet est attendu.'
	}
	return errors
}
