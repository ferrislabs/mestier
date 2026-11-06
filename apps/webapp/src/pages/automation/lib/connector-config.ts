import type { Schemas } from '#/api/api.client'
import { isExpression } from '#/pages/automation/lib/parameters'

export const SIGNING_CREDENTIAL_FIELD_NAME = 'signing_credential_id'

export function isSigningCredentialField(
	field: Schemas.FieldResponse,
): boolean {
	return field.name === SIGNING_CREDENTIAL_FIELD_NAME
}

export function isFieldVisible(
	field: Schemas.FieldResponse,
	config: Record<string, unknown>,
): boolean {
	if (!field.visible_when) return true

	const controllingValue = config[field.visible_when.field]
	return (
		typeof controllingValue === 'string' &&
		field.visible_when.any_of.includes(controllingValue)
	)
}

export function visibleFields(
	fields: Schemas.FieldResponse[],
	config: Record<string, unknown>,
): Schemas.FieldResponse[] {
	return fields.filter((field) => isFieldVisible(field, config))
}

export function withField(
	config: Record<string, unknown>,
	name: string,
	value: unknown,
): Record<string, unknown> {
	return { ...config, [name]: value }
}

export function withoutField(
	config: Record<string, unknown>,
	name: string,
): Record<string, unknown> {
	const next = { ...config }
	delete next[name]
	return next
}

export function isSelectKind(
	kind: Schemas.FieldKindResponse,
): kind is { Select: { options: Schemas.SelectOptionResponse[] } } {
	return typeof kind === 'object' && kind !== null && 'Select' in kind
}

export type JsonEntryKind = 'string' | 'number' | 'boolean' | 'json'

export const JSON_ENTRY_KINDS: readonly {
	value: JsonEntryKind
	label: string
}[] = [
	{ value: 'string', label: 'Texte' },
	{ value: 'number', label: 'Nombre' },
	{ value: 'boolean', label: 'Booléen' },
	{ value: 'json', label: 'JSON' },
]

export interface JsonEntry {
	key: string
	value: string
	kind: JsonEntryKind
}

export interface FieldAddOns {
	placeholder?: string
	hint?: string
	entryKinds?: Readonly<Record<string, JsonEntryKind>>
}

export const RAW_JSON_ERROR =
	'JSON invalide : vérifiez les guillemets, les virgules et les accolades. Une expression doit être entre guillemets.'

export function isJsonMapRepresentable(value: unknown): boolean {
	return (
		value === undefined ||
		(typeof value === 'object' && value !== null && !Array.isArray(value))
	)
}

function entryOf(
	key: string,
	value: unknown,
	declared?: JsonEntryKind,
): JsonEntry {
	if (declared !== undefined) {
		if (value === null || value === undefined) {
			return { key, value: '', kind: declared }
		}
		if (typeof value === 'string' && isExpression(value)) {
			return { key, value, kind: declared }
		}
	}
	if (typeof value === 'string') return { key, value, kind: 'string' }
	if (typeof value === 'number')
		return { key, value: String(value), kind: 'number' }
	if (typeof value === 'boolean') {
		return { key, value: String(value), kind: 'boolean' }
	}
	return { key, value: JSON.stringify(value), kind: 'json' }
}

export function jsonEntriesFromValue(
	value: unknown,
	declared: Readonly<Record<string, JsonEntryKind>> = {},
): JsonEntry[] {
	if (!isJsonMapRepresentable(value) || value === undefined) return []
	return Object.entries(value as Record<string, unknown>).map(([key, entry]) =>
		entryOf(
			key,
			entry,
			Object.hasOwn(declared, key) ? declared[key] : undefined,
		),
	)
}

export function jsonEntryValue(entry: JsonEntry): {
	value: unknown
	error: string | null
} {
	if (entry.kind === 'string' || isExpression(entry.value)) {
		return { value: entry.value, error: null }
	}
	const raw = entry.value.trim()
	if (raw === '') return { value: null, error: null }
	if (entry.kind === 'number') {
		const number = Number(raw)
		return Number.isFinite(number)
			? { value: number, error: null }
			: { value: entry.value, error: 'Nombre invalide.' }
	}
	if (entry.kind === 'boolean') {
		return raw === 'true' || raw === 'false'
			? { value: raw === 'true', error: null }
			: { value: entry.value, error: 'Saisissez true ou false.' }
	}
	try {
		return { value: JSON.parse(raw), error: null }
	} catch {
		return { value: entry.value, error: 'JSON invalide.' }
	}
}

export function valueFromJsonEntries(
	entries: readonly JsonEntry[],
): Record<string, unknown> | undefined {
	const populated = entries.filter((entry) => entry.key.trim() !== '')
	if (populated.length === 0) return undefined

	const result: Record<string, unknown> = {}
	for (const entry of populated) {
		result[entry.key.trim()] = jsonEntryValue(entry).value
	}
	return result
}

export type RawJson =
	| { state: 'empty' }
	| { state: 'valid'; value: unknown }
	| { state: 'invalid' }

export function parseRawJson(text: string): RawJson {
	const trimmed = text.trim()
	if (trimmed === '') return { state: 'empty' }
	try {
		return { state: 'valid', value: JSON.parse(trimmed) }
	} catch {
		return trimmed.startsWith('{{') && trimmed.endsWith('}}')
			? { state: 'valid', value: trimmed }
			: { state: 'invalid' }
	}
}

export function numberFieldValue(raw: string): number | string | undefined {
	const trimmed = raw.trim()
	if (trimmed === '') return undefined
	if (isExpression(raw)) return raw
	const number = Number(trimmed)
	return Number.isFinite(number) ? number : raw
}

export function numberFieldError(raw: string): string | null {
	const trimmed = raw.trim()
	if (trimmed === '' || isExpression(raw)) return null
	return Number.isFinite(Number(trimmed)) ? null : 'Nombre invalide.'
}
