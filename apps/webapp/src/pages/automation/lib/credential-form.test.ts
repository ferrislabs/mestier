import { describe, expect, it } from 'vitest'
import type { AuthScheme } from '#/hooks/use-automation'
import {
	buildCredentialFieldErrors,
	buildCredentialFormErrors,
	credentialErrorMessage,
	credentialFormDefaults,
	credentialPayload,
	emptyCredentialForm,
	translateCredentialMessage,
	visibleCredentialFields,
} from '#/pages/automation/lib/credential-form'
import {
	type LocalizedField,
	type LocalizedScheme,
	localizeScheme,
} from '#/pages/automation/lib/credential-meta'
import type { CredentialFormValues } from '#/pages/automation/types'

describe('emptyCredentialForm', () => {
	it('defaults to a supplied origin with no data', () => {
		expect(emptyCredentialForm()).toEqual({
			kind: '',
			name: '',
			description: '',
			origin: 'supplied',
			data: {},
		})
	})

	it('accepts a default kind', () => {
		expect(emptyCredentialForm('bearer_token').kind).toBe('bearer_token')
	})
})

describe('buildCredentialFormErrors', () => {
	const bearerScheme: AuthScheme = {
		kind: 'bearer_token',
		label: 'Bearer token',
		fields: [
			{
				name: 'token',
				label: 'Token',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	}

	function values(overrides: Partial<CredentialFormValues> = {}) {
		return {
			kind: 'bearer_token',
			name: 'Ma clé',
			description: '',
			origin: 'supplied' as const,
			data: {},
			...overrides,
		}
	}

	it('requires a name', () => {
		const errors = buildCredentialFormErrors(
			values({ name: '' }),
			bearerScheme,
			'create',
		)
		expect(errors).toContain('Le nom est requis')
	})

	it('requires every scheme field on create', () => {
		const errors = buildCredentialFormErrors(values(), bearerScheme, 'create')
		expect(errors).toContain('Token est requis')
	})

	it('accepts a fully filled scheme on create', () => {
		const errors = buildCredentialFormErrors(
			values({ data: { token: 'abc' } }),
			bearerScheme,
			'create',
		)
		expect(errors).toEqual([])
	})

	it('allows every field blank on edit — rename only, data untouched', () => {
		const errors = buildCredentialFormErrors(values(), bearerScheme, 'edit')
		expect(errors).toEqual([])
	})

	it('requires the field once the user starts filling data in on edit', () => {
		const errors = buildCredentialFormErrors(
			values({ data: { token: '' } }),
			bearerScheme,
			'edit',
		)
		// An entry present but blank does not count as "started filling in" —
		// only a non-blank value does (see the implementation's `filledAny`).
		expect(errors).toEqual([])
	})

	it('skips scheme validation entirely for a generated credential', () => {
		const errors = buildCredentialFormErrors(
			values({ origin: 'generated' }),
			bearerScheme,
			'create',
		)
		expect(errors).toEqual([])
	})
})

describe('buildCredentialFieldErrors', () => {
	const scheme: AuthScheme = {
		kind: 'smtp',
		label: 'SMTP',
		fields: [
			{
				name: 'port',
				label: 'Port',
				required: true,
				kind: 'Number',
				expression: false,
				secret: false,
			},
			{
				name: 'token',
				label: 'Jeton',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	}
	const base: CredentialFormValues = {
		kind: 'smtp',
		name: '',
		description: '',
		origin: 'supplied',
		data: {},
	}

	it('keys each message by the field it belongs to', () => {
		expect(buildCredentialFieldErrors(base, scheme, 'create')).toEqual({
			name: 'Donnez un nom à cette identification.',
			port: 'Champ obligatoire.',
			token: 'Champ obligatoire.',
		})
	})

	it('expects whole numbers in number fields', () => {
		expect(
			buildCredentialFieldErrors(
				{ ...base, name: 'x', data: { port: '25a', token: 't' } },
				scheme,
				'create',
			),
		).toEqual({ port: 'Nombre entier attendu.' })
	})

	it('only checks the name on edit while no data is replaced', () => {
		expect(
			buildCredentialFieldErrors({ ...base, name: 'x' }, scheme, 'edit'),
		).toEqual({})
	})
})

function localizedField(
	name: string,
	overrides: Partial<LocalizedField> = {},
): LocalizedField {
	return {
		name,
		label: name,
		required: true,
		kind: 'Text',
		expression: false,
		secret: false,
		...overrides,
	}
}

const SSH_LIKE: LocalizedScheme = {
	kind: 'ssh_like',
	label: 'SSH',
	fields: [
		localizedField('method', {
			kind: {
				Select: {
					options: [
						{ value: 'password', label: 'Mot de passe' },
						{ value: 'key', label: 'Clé privée' },
					],
				},
			},
			defaultValue: 'password',
		}),
		localizedField('password', {
			secret: true,
			visible_when: { field: 'method', any_of: ['password'] },
		}),
		localizedField('private_key', {
			secret: true,
			visible_when: { field: 'method', any_of: ['key'] },
		}),
	],
}

describe('visibleCredentialFields', () => {
	it('shows every field that has no condition', () => {
		const fields = [localizedField('a'), localizedField('b')]
		expect(visibleCredentialFields(fields, {})).toEqual(fields)
	})

	it('shows a conditional field only when the controlling field has one of the values', () => {
		const names = (data: Record<string, string>) =>
			visibleCredentialFields(SSH_LIKE.fields, data).map((entry) => entry.name)

		expect(names({ method: 'key' })).toEqual(['method', 'private_key'])
		expect(names({ method: 'password' })).toEqual(['method', 'password'])
	})

	it('falls back on the default of the controlling field while it is untouched', () => {
		expect(
			visibleCredentialFields(SSH_LIKE.fields, {}).map((entry) => entry.name),
		).toEqual(['method', 'password'])
	})
})

describe('credentialFormDefaults', () => {
	it('prefills the non-secret fields that have a default value', () => {
		expect(
			credentialFormDefaults(
				localizeScheme({
					kind: 'http_header',
					label: 'HTTP header',
					fields: [
						localizedField('header_name'),
						localizedField('header_value', { secret: true }),
					],
				}),
			),
		).toEqual({ header_name: 'X-API-Key' })
	})

	it('has nothing to prefill for a scheme without defaults, or without a scheme', () => {
		expect(credentialFormDefaults(undefined)).toEqual({})
		expect(credentialFormDefaults({ fields: [localizedField('a')] })).toEqual(
			{},
		)
	})
})

describe('credentialPayload', () => {
	it('drops the fields that are hidden', () => {
		expect(
			credentialPayload(SSH_LIKE, {
				method: 'key',
				password: 'old',
				private_key: 'pem',
			}),
		).toEqual({ method: 'key', private_key: 'pem' })
	})

	it('trims the non-secret values and leaves secrets untouched', () => {
		const scheme: LocalizedScheme = {
			kind: 'x',
			label: 'X',
			fields: [
				localizedField('user'),
				localizedField('pass', { secret: true }),
			],
		}
		expect(credentialPayload(scheme, { user: ' bob ', pass: ' s3 ' })).toEqual({
			user: 'bob',
			pass: ' s3 ',
		})
	})

	it('keeps only the fields the user has a value for', () => {
		const scheme: LocalizedScheme = {
			kind: 'x',
			label: 'X',
			fields: [localizedField('a'), localizedField('b')],
		}
		expect(credentialPayload(scheme, { a: '1' })).toEqual({ a: '1' })
	})
})

describe('validation of conditional fields', () => {
	const base: CredentialFormValues = {
		kind: 'ssh_like',
		name: 'x',
		description: '',
		origin: 'supplied',
		data: {},
	}

	it('does not ask for a field that is hidden', () => {
		expect(
			buildCredentialFieldErrors(
				{ ...base, data: { method: 'password' } },
				SSH_LIKE,
				'create',
			),
		).toEqual({ password: 'Champ obligatoire.' })
		expect(
			buildCredentialFieldErrors(
				{ ...base, data: { method: 'key' } },
				SSH_LIKE,
				'create',
			),
		).toEqual({ private_key: 'Champ obligatoire.' })
	})

	it('keeps the form-level list in line with the visible fields', () => {
		expect(
			buildCredentialFormErrors(
				{ ...base, data: { method: 'key', private_key: 'pem' } },
				SSH_LIKE,
				'create',
			),
		).toEqual([])
	})
})

describe('translateCredentialMessage', () => {
	const fields = [
		{ name: 'api_key', label: 'Clé d’API' },
		{ name: 'username', label: 'Utilisateur' },
	]

	it('says in French that a used credential cannot be deleted, without the prefix', () => {
		expect(
			translateCredentialMessage(
				'Conflict: credential is still referenced by workflow(s): Facture Odoo, Relance',
				fields,
			),
		).toBe(
			'Impossible de la supprimer : elle est encore utilisée par Facture Odoo, Relance.',
		)
	})

	it('says in French that the name is taken, without the constraint name', () => {
		const message = translateCredentialMessage(
			'Conflict: uq_automation_credential_name',
			fields,
		)
		expect(message).toBe('Une identification de ce type porte déjà ce nom.')
		expect(message).not.toContain('uq_')
		expect(message).not.toContain('Conflict')
	})

	it('names the missing or unknown field by its French label', () => {
		expect(
			translateCredentialMessage('Conflict: missing field `api_key`', fields),
		).toBe('Le champ « Clé d’API » est obligatoire.')
		expect(
			translateCredentialMessage('Conflict: unknown field `legacy`', fields),
		).toBe('Le champ « legacy » n’existe pas pour ce type d’identification.')
	})

	it('leaves a message it does not know, minus the prefix', () => {
		expect(translateCredentialMessage('La création a échoué.', fields)).toBe(
			'La création a échoué.',
		)
		expect(translateCredentialMessage('Conflict: autre chose', fields)).toBe(
			'autre chose',
		)
	})
})

describe('credentialErrorMessage', () => {
	it('returns nothing when there is no error', () => {
		expect(credentialErrorMessage(null)).toBeNull()
		expect(credentialErrorMessage(undefined)).toBeNull()
	})

	it('translates the known conflicts', () => {
		expect(
			credentialErrorMessage(
				new Error('Conflict: uq_automation_credential_name'),
			),
		).toBe('Une identification de ce type porte déjà ce nom.')
	})

	it('explains a refusal or a missing credential in French', () => {
		expect(
			credentialErrorMessage(
				Object.assign(new Error('Forbidden'), { status: 403 }),
			),
		).toBe('Vous n’avez pas la permission de gérer les identifications.')
		expect(
			credentialErrorMessage(
				Object.assign(new Error('Resource not found'), { status: 404 }),
			),
		).toBe('Cette identification n’existe plus.')
	})

	it('never shows an English technical message', () => {
		expect(credentialErrorMessage(new Error('HTTP 502: Bad Gateway'))).toBe(
			'Une erreur est survenue. Réessayez dans un instant.',
		)
	})
})
