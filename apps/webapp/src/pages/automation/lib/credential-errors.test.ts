import { describe, expect, it } from 'vitest'
import { credentialFailure } from '#/pages/automation/lib/credential-errors'
import {
	BEARER_TOKEN,
	OAUTH2_AUTHORIZATION_CODE,
} from '#/pages/automation/test/credential-fixtures'

function failure(message: string, status?: number) {
	return Object.assign(new Error(message), { status })
}

describe('credentialFailure', () => {
	it('is nothing without an error', () => {
		expect(credentialFailure(null)).toBeNull()
		expect(credentialFailure(undefined)).toBeNull()
	})

	it('shows the French message of a validation refusal without the prefix of the server', () => {
		expect(
			credentialFailure(
				failure('Validation failed: Le champ « Jeton » est obligatoire.', 400),
				BEARER_TOKEN.fields,
			),
		).toEqual({
			message: 'Le champ « Jeton » est obligatoire.',
			field: 'token',
		})
	})

	it('puts the message under the secret to type again when an address moved', () => {
		expect(
			credentialFailure(
				failure(
					'Validation failed: Modifier cette adresse exige de saisir de nouveau « Client secret ».',
					400,
				),
				OAUTH2_AUTHORIZATION_CODE.fields,
			),
		).toEqual({
			message:
				'Modifier cette adresse exige de saisir de nouveau « Client secret ».',
			field: 'client_secret',
		})
	})

	it('keeps a message that names several fields, or none, for the whole form', () => {
		const message =
			'Validation failed: « Jeton » et « Jeton » ne vont pas ensemble.'

		expect(
			credentialFailure(failure(message, 400), [
				...BEARER_TOKEN.fields,
				...BEARER_TOKEN.fields,
			]),
		).toMatchObject({ field: null })
		expect(
			credentialFailure(
				failure(
					'Validation failed: JSON invalide : un objet est attendu.',
					400,
				),
			),
		).toEqual({ message: 'JSON invalide : un objet est attendu.', field: null })
	})

	it('puts a name already taken under the name', () => {
		expect(
			credentialFailure(
				failure(
					'Conflict: Une identification de ce type porte déjà ce nom.',
					409,
				),
			),
		).toEqual({
			message: 'Une identification de ce type porte déjà ce nom.',
			field: 'name',
		})
	})

	it('puts a description that is too long under the description', () => {
		expect(
			credentialFailure(
				failure(
					'Validation failed: La description ne peut pas dépasser 500 caractères.',
					400,
				),
			)?.field,
		).toBe('description')
	})

	it('keeps the message of a server that cannot connect accounts', () => {
		const message =
			'La connexion de comptes n’est pas configurée sur ce serveur : l’administrateur doit renseigner l’adresse publique de l’API.'

		expect(credentialFailure(failure(message, 409))).toEqual({
			message,
			field: null,
		})
	})

	it('translates the English leftovers of the server', () => {
		expect(
			credentialFailure(
				failure(
					'Conflict: no automation secret key configured: credentials cannot be sealed',
					409,
				),
			)?.message,
		).toBe(
			'Le chiffrement des identifications n’est pas configuré sur ce serveur.',
		)
		expect(
			credentialFailure(
				failure('Bad request: the authorization is not valid', 400),
			)?.message,
		).toBe('L’autorisation n’est plus valide. Recommencez la connexion.')
	})

	it('explains a refused permission and a credential that is gone', () => {
		expect(credentialFailure(failure('Forbidden', 403))?.message).toBe(
			'Vous n’avez pas la permission de gérer les identifications.',
		)
		expect(credentialFailure(failure('Resource not found', 404))?.message).toBe(
			'Cette identification n’existe plus.',
		)
	})

	it('falls back to a generic message for anything else', () => {
		const generic = 'Une erreur est survenue. Réessayez dans un instant.'

		expect(
			credentialFailure(failure('HTTP 500: Internal Server Error', 500))
				?.message,
		).toBe(generic)
		expect(credentialFailure(failure('Failed to fetch'))?.message).toBe(generic)
		expect(
			credentialFailure(failure('HTTP 400: Bad Request', 400))?.message,
		).toBe(generic)
	})
})
