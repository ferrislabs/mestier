import type { AuthScheme } from '#/hooks/use-automation'
import type { CredentialFormValues } from '#/pages/automation/types'

export function emptyCredentialForm(defaultKind = ''): CredentialFormValues {
	return { kind: defaultKind, name: '', origin: 'supplied', data: {} }
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

	for (const field of scheme.fields) {
		if (field.required && (values.data[field.name] ?? '').trim() === '') {
			errors.push(`${field.label} est requis`)
		}
	}
	return errors
}
