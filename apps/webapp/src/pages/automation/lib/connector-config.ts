import type { Schemas } from '#/api/api.client'

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

export interface JsonEntry {
	key: string
	value: string
}

export function isJsonMapRepresentable(value: unknown): boolean {
	return (
		value === undefined ||
		(typeof value === 'object' && value !== null && !Array.isArray(value))
	)
}

function stringifyEntryValue(value: unknown): string {
	if (typeof value === 'string') return value
	return JSON.stringify(value)
}

export function jsonEntriesFromValue(value: unknown): JsonEntry[] {
	if (!isJsonMapRepresentable(value) || value === undefined) return []
	return Object.entries(value as Record<string, unknown>).map(
		([key, entryValue]) => ({
			key,
			value: stringifyEntryValue(entryValue),
		}),
	)
}

function parseEntryValue(value: string): unknown {
	try {
		return JSON.parse(value)
	} catch {
		return value
	}
}

export function valueFromJsonEntries(
	entries: JsonEntry[],
): Record<string, unknown> | undefined {
	const populated = entries.filter((entry) => entry.key.trim() !== '')
	if (populated.length === 0) return undefined

	const result: Record<string, unknown> = {}
	for (const entry of populated) {
		result[entry.key] = parseEntryValue(entry.value)
	}
	return result
}

export function nextJsonEntryKey(entries: JsonEntry[]): string {
	const used = new Set(entries.map((entry) => entry.key))
	if (!used.has('key')) return 'key'

	let suffix = 2
	while (used.has(`key_${suffix}`)) suffix += 1
	return `key_${suffix}`
}
