import type { AuthScheme } from '#/hooks/use-automation'
import type { LocalizedField } from '#/pages/automation/lib/credential-meta'
import type { CredentialFormValues } from '#/pages/automation/types'

export function emptyCredentialForm(defaultKind = ''): CredentialFormValues {
	return {
		kind: defaultKind,
		name: '',
		description: '',
		origin: 'supplied',
		data: {},
	}
}

export function visibleCredentialFields<F extends LocalizedField>(
	fields: readonly F[],
	data: Readonly<Record<string, string>>,
): F[] {
	return fields.filter((field) => {
		if (!field.visible_when) return true
		const controller = fields.find(
			(candidate) => candidate.name === field.visible_when?.field,
		)
		const current =
			data[field.visible_when.field] ?? controller?.defaultValue ?? ''
		return field.visible_when.any_of.includes(current)
	})
}

export function credentialFormDefaults(
	scheme: { fields: readonly LocalizedField[] } | undefined,
): Record<string, string> {
	return Object.fromEntries(
		(scheme?.fields ?? [])
			.filter((field) => !field.secret && field.defaultValue !== undefined)
			.map((field) => [field.name, field.defaultValue ?? '']),
	)
}

export function credentialPayload(
	scheme: { fields: readonly LocalizedField[] },
	data: Readonly<Record<string, string>>,
): Record<string, string> {
	return Object.fromEntries(
		visibleCredentialFields(scheme.fields, data)
			.filter((field) => field.name in data)
			.map((field) => {
				const value = data[field.name] ?? ''
				return [field.name, field.secret ? value : value.trim()]
			}),
	)
}

/**
 * Required-field checks against the chosen auth scheme — the same
 * validation `validate_credential_data` runs server-side, done client-side
 * first so a missing field never costs a round trip.
 *
 * In `edit` mode, the data section is optional as a whole: leaving every
 * field blank renames without touching the sealed bytes (`data: undefined`
 * on the wire); filling in any one of them means "replace", which requires
 * every field the scheme needs — a partial replacement is not a thing the
 * backend can validate against the scheme, so it is refused here first.
 */
export function buildCredentialFormErrors(
	values: CredentialFormValues,
	scheme: AuthScheme | undefined,
	mode: 'create' | 'edit',
): string[] {
	const errors: string[] = []
	if (values.name.trim() === '') errors.push('Le nom est requis')
	if (values.kind === '') errors.push('Le type est requis')

	if (values.origin !== 'supplied' || !scheme) return errors

	const filledAny = Object.values(values.data).some(
		(value) => value.trim() !== '',
	)
	if (mode === 'edit' && !filledAny) return errors

	for (const field of visibleCredentialFields(scheme.fields, values.data)) {
		if (field.required && (values.data[field.name] ?? '').trim() === '') {
			errors.push(`${field.label} est requis`)
		}
	}
	return errors
}

export type CredentialFieldErrors = Readonly<Record<string, string>>

export function buildCredentialFieldErrors(
	values: CredentialFormValues,
	scheme: AuthScheme | undefined,
	mode: 'create' | 'edit',
): CredentialFieldErrors {
	const errors: Record<string, string> = {}
	if (values.name.trim() === '')
		errors.name = 'Donnez un nom à cette identification.'

	if (values.origin !== 'supplied' || !scheme) return errors

	const filledAny = Object.values(values.data).some(
		(value) => value.trim() !== '',
	)
	if (mode === 'edit' && !filledAny) return errors

	for (const field of visibleCredentialFields(scheme.fields, values.data)) {
		const value = (values.data[field.name] ?? '').trim()
		if (field.required && value === '')
			errors[field.name] = 'Champ obligatoire.'
		else if (field.kind === 'Number' && value !== '' && !/^\d+$/.test(value))
			errors[field.name] = 'Nombre entier attendu.'
	}
	return errors
}

const GENERIC_ERROR = 'Une erreur est survenue. Réessayez dans un instant.'

const CONFLICT_PREFIX = /^Conflict:\s*/

interface NamedField {
	name: string
	label: string
}

function knownMessage(
	message: string,
	fields: readonly NamedField[],
): string | null {
	const labelOf = (name: string) =>
		fields.find((field) => field.name === name)?.label ?? name

	const referenced = message.match(
		/^credential is still referenced by workflow\(s\): (.+)$/,
	)
	if (referenced)
		return `Impossible de la supprimer : elle est encore utilisée par ${referenced[1]}.`

	if (message === 'uq_automation_credential_name')
		return 'Une identification de ce type porte déjà ce nom.'

	const missing = message.match(/^missing field `(.+)`$/)
	if (missing)
		return `Le champ « ${labelOf(missing[1] ?? '')} » est obligatoire.`

	const unknown = message.match(/^unknown field `(.+)`$/)
	if (unknown)
		return `Le champ « ${labelOf(unknown[1] ?? '')} » n’existe pas pour ce type d’identification.`

	return null
}

export function translateCredentialMessage(
	message: string,
	fields: readonly NamedField[] = [],
): string {
	const text = message.replace(CONFLICT_PREFIX, '')
	return knownMessage(text, fields) ?? text
}

export function credentialErrorMessage(
	error: (Error & { status?: number }) | null | undefined,
	fields: readonly NamedField[] = [],
): string | null {
	if (!error) return null
	const known = knownMessage(error.message.replace(CONFLICT_PREFIX, ''), fields)
	if (known) return known
	if (error.status === 403)
		return 'Vous n’avez pas la permission de gérer les identifications.'
	if (error.status === 404) return 'Cette identification n’existe plus.'
	return GENERIC_ERROR
}
