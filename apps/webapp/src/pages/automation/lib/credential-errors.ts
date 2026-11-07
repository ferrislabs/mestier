import type { CredentialField } from '#/hooks/use-automation'

export interface CredentialFailure {
	message: string
	field: string | null
}

type ApiFailure = Error & { status?: number; data?: { code?: string } }

const GENERIC_ERROR = 'Une erreur est survenue. Réessayez dans un instant.'

const SERVER_PREFIX = /^(?:Validation failed|Conflict|Bad request):\s*/

const NAME_MESSAGES: readonly string[] = [
	'Une identification de ce type porte déjà ce nom.',
	'Donnez un nom à cette identification.',
	'Le nom ne peut pas dépasser 120 caractères.',
]

const DESCRIPTION_MESSAGES: readonly string[] = [
	'La description ne peut pas dépasser 500 caractères.',
]

const FRENCH_MESSAGES: Readonly<Record<string, string>> = {
	'no automation secret key configured: credentials cannot be sealed':
		'Le chiffrement des identifications n’est pas configuré sur ce serveur.',
	'the credential changed while the provider was being called':
		'L’identification a changé pendant l’appel au fournisseur. Réessayez.',
	'the authorization is not valid':
		'L’autorisation n’est plus valide. Recommencez la connexion.',
}

function fieldNamed(
	message: string,
	fields: readonly CredentialField[],
): string | null {
	const named = fields.filter((field) => message.includes(`« ${field.label} »`))
	return named.length === 1 ? (named[0]?.name ?? null) : null
}

function fieldOf(
	message: string,
	fields: readonly CredentialField[],
): string | null {
	if (NAME_MESSAGES.includes(message)) return 'name'
	if (DESCRIPTION_MESSAGES.includes(message)) return 'description'
	return fieldNamed(message, fields)
}

export function credentialFailure(
	error: ApiFailure | null | undefined,
	fields: readonly CredentialField[] = [],
): CredentialFailure | null {
	if (!error) return null
	if (error.status === 403)
		return {
			message: 'Vous n’avez pas la permission de gérer les identifications.',
			field: null,
		}
	if (error.status === 404)
		return { message: 'Cette identification n’existe plus.', field: null }
	if (error.status !== 400 && error.status !== 409)
		return { message: GENERIC_ERROR, field: null }

	const text = error.message.replace(SERVER_PREFIX, '')
	const message = FRENCH_MESSAGES[text] ?? text
	if (message.trim() === '' || /^HTTP \d{3}/.test(message))
		return { message: GENERIC_ERROR, field: null }
	return { message, field: fieldOf(message, fields) }
}
