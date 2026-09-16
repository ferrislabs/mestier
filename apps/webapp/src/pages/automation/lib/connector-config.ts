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
